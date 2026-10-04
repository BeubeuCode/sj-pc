use std::path::{Path, PathBuf};
use std::time::Duration;

use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sj_emu::{AvInfo, Core, CoreConfig, Frame, Pointer};
use sj_game::audio::apply_volume;
use sj_game::core_options::{core_variables, Renderer};
use sj_game::game_api::GameApi;
use sj_game::game_mode::{self, GameMode};
use sj_game::input::button_mask;
use sj_game::screen_director::{self, arrangement_rects, touch_target, ScreenPlan};
use sj_game::settings::{HotkeyAction, Settings};
use sj_game::snapshot::{self, Snapshot};
use sj_game::touch::{bottom_screen_to_pointer, mouse_to_bottom_screen};

use crate::capture::{encode_ppm, request_from_env, CaptureRequest};
use crate::dev_panel::DevPanel;
use crate::host_input::{key_input_name, pad_button_input_name, AxisThresholds, HostInput};
use crate::hud::draw_overlays;
use crate::overlay::Overlay;
use crate::platform::Platform;
use crate::present::{read_default_framebuffer, Presenter};
use crate::savestate::{next_slot, previous_slot, slot_path};

const FAST_FORWARD_FRAMES_PER_PRESENT: u32 = 4;
const SAMPLES_DIR: &str = "re/samples";
const DEV_PANEL_KEY: &str = "F12";
const AUDIO_BUFFER_FRAMES: f64 = 4.0;

pub struct App {
    settings: Settings,
    core: Core,
    overlay: Overlay,
    hud_texture: egui::TextureId,
    dev_panel: DevPanel,
    platform: Platform,
    presenter: Presenter,
    audio: AudioQueue<i16>,
    audio_target_bytes: u32,
    input: HostInput,
    mouse_drawable_px: Option<(i32, i32)>,
    capture: Option<CaptureRequest>,
    frames_presented: u64,
    pending_state: Option<Vec<u8>>,
    slot: u8,
    swapped: bool,
    fast_forward: bool,
    running: bool,
}

impl App {
    pub fn new(mut platform: Platform, settings: Settings) -> Result<Self, String> {
        platform.apply_video_settings(&settings.video)?;
        std::fs::create_dir_all(&settings.save_dir)
            .map_err(|error| format!("cannot create save dir: {error}"))?;
        let mut core = load_core(&platform, &settings)?;
        let av_info = core
            .load_game(&settings.rom)
            .map_err(|error| error.to_string())?;
        let presenter = create_presenter(&platform.gl, &core, av_info)?;
        let mut overlay = Overlay::new(&platform);
        let hud_texture = overlay.register_native_texture(presenter.color_texture());
        let audio = open_audio(&platform, av_info)?;
        let thresholds =
            AxisThresholds::from_deadzone_percent(settings.controller.stick_deadzone_percent);
        Ok(Self {
            settings,
            core,
            overlay,
            hud_texture,
            dev_panel: DevPanel::new(),
            platform,
            presenter,
            audio,
            audio_target_bytes: audio_target_bytes(av_info),
            input: HostInput::new(thresholds),
            mouse_drawable_px: None,
            capture: request_from_env(),
            frames_presented: 0,
            pending_state: None,
            slot: 0,
            swapped: false,
            fast_forward: false,
            running: true,
        })
    }

    pub fn run(&mut self) {
        self.update_title("");
        self.audio.resume();
        while self.running {
            self.handle_events();
            self.emulate();
            self.present();
            self.throttle();
        }
    }

    fn emulate(&mut self) {
        let frames = if self.fast_forward {
            FAST_FORWARD_FRAMES_PER_PRESENT
        } else {
            1
        };
        for _ in 0..frames {
            self.core.set_input(
                button_mask(&self.settings.bindings, |name| self.input.is_pressed(name)),
                self.pointer(),
            );
            let mut output = self.core.run_frame();
            self.apply_pending_state();
            if let Some(av_info) = output.new_av_info {
                self.presenter.ensure_capacity(
                    &self.platform.gl,
                    av_info.max_width_px,
                    av_info.max_height_px,
                );
            }
            if let Some(frame) = output.frame {
                self.accept_frame(frame);
            }
            if !self.fast_forward {
                apply_volume(&self.settings.audio, &mut output.stereo_samples);
                let _ = self.audio.queue_audio(&output.stereo_samples);
            }
        }
    }

    fn accept_frame(&mut self, frame: Frame) {
        match frame {
            Frame::Hardware {
                width_px,
                height_px,
            } => {
                self.presenter.note_hardware_frame(width_px, height_px);
            }
            Frame::Software {
                width_px,
                height_px,
                xrgb_pixels,
            } => {
                self.presenter.upload_software_frame(
                    &self.platform.gl,
                    width_px,
                    height_px,
                    &xrgb_pixels,
                );
            }
        }
    }

    fn present(&mut self) {
        let drawable_px = self.platform.window.drawable_size();
        let mode = game_mode::read(&self.core);
        let plan = self.plan(mode);
        let screens = arrangement_rects(drawable_px, plan.arrangement);
        self.presenter.draw(
            &self.platform.gl,
            drawable_px,
            &screens,
            self.settings.video.filter,
        );
        let frame = self.presenter.frame_info();
        let (texture, margin_px) = (self.hud_texture, self.settings.hud.margin_px);
        let status = format!("mode: {mode:?}  swapped: {}", self.swapped);
        let ram = self.core.main_ram();
        let dev_panel = &mut self.dev_panel;
        self.overlay.draw(|root| {
            draw_overlays(root, &plan, frame, texture, drawable_px, margin_px);
            dev_panel.show(root, ram, &status);
        });
        self.capture_if_requested();
        self.platform.window.gl_swap_window();
        self.frames_presented += 1;
    }

    fn capture_if_requested(&mut self) {
        let Some(request) = &self.capture else {
            return;
        };
        if self.frames_presented < request.after_frames {
            return;
        }
        let drawable_px = self.platform.window.drawable_size();
        let rgba = read_default_framebuffer(&self.platform.gl, drawable_px);
        if let Err(error) = std::fs::write(
            &request.path,
            encode_ppm(drawable_px.0, drawable_px.1, &rgba),
        ) {
            eprintln!("sj: capture failed: {error}");
        }
        self.running = false;
    }

    fn throttle(&self) {
        if self.fast_forward {
            return;
        }
        while self.audio.size() > self.audio_target_bytes {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn plan(&self, mode: GameMode) -> ScreenPlan {
        screen_director::plan(mode, self.swapped, &self.settings.hud, &self.settings.pip)
    }

    fn pointer(&self) -> Pointer {
        let Some((mouse_x, mouse_y)) = self.mouse_drawable_px else {
            return Pointer::default();
        };
        let drawable_px = self.platform.window.drawable_size();
        let plan = self.plan(game_mode::read(&self.core));
        let Some(touch) = touch_target(drawable_px, &plan, self.settings.hud.margin_px)
            .and_then(|target| mouse_to_bottom_screen(target, mouse_x, mouse_y))
        else {
            return Pointer::default();
        };
        let (x, y) = bottom_screen_to_pointer(touch.0, touch.1);
        Pointer {
            x,
            y,
            pressed: true,
        }
    }

    fn handle_events(&mut self) {
        let events: Vec<Event> = self.platform.events.poll_iter().collect();
        for event in events {
            self.handle_event(&event);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        self.overlay.handle_event(&self.platform, event);
        if self.overlay.wants_keyboard() && is_keyboard(event) {
            return;
        }
        if self.overlay.wants_pointer() && is_mouse(event) {
            self.mouse_drawable_px = None;
            return;
        }
        match *event {
            Event::Quit { .. } => self.running = false,
            Event::KeyDown {
                keycode: Some(key),
                repeat,
                ..
            } => {
                if !repeat {
                    self.handle_hotkey(key);
                }
                self.input.set_key(key, true);
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                if self.settings.hotkeys.action_for(&key_input_name(key))
                    == Some(HotkeyAction::FastForward)
                {
                    self.set_fast_forward(false);
                }
                self.input.set_key(key, false);
            }
            Event::ControllerDeviceAdded { which, .. } => self.platform.open_controller(which),
            Event::ControllerDeviceRemoved { which, .. } => {
                self.platform.close_controller(which);
                self.input.release_all_pad_inputs();
            }
            Event::ControllerButtonDown { button, .. } => {
                if pad_button_input_name(button) == self.settings.controller.swap_screens_button {
                    self.swapped = !self.swapped;
                }
                self.input.set_pad_button(button, true);
            }
            Event::ControllerButtonUp { button, .. } => self.input.set_pad_button(button, false),
            Event::ControllerAxisMotion { axis, value, .. } => self.input.set_pad_axis(axis, value),
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.mouse_drawable_px = Some(self.platform.to_drawable(x, y));
            }
            Event::MouseMotion {
                mousestate, x, y, ..
            } if mousestate.left() => {
                self.mouse_drawable_px = Some(self.platform.to_drawable(x, y));
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.mouse_drawable_px = None;
            }
            _ => {}
        }
    }

    fn handle_hotkey(&mut self, key: Keycode) {
        if key_input_name(key) == DEV_PANEL_KEY {
            self.dev_panel.open = !self.dev_panel.open;
            return;
        }
        let Some(action) = self.settings.hotkeys.action_for(&key_input_name(key)) else {
            return;
        };
        match action {
            HotkeyAction::FastForward => self.set_fast_forward(true),
            HotkeyAction::TogglePip => self.settings.pip.visible = !self.settings.pip.visible,
            HotkeyAction::ToggleFullscreen => self.platform.toggle_fullscreen(),
            HotkeyAction::SaveState => self.save_state(),
            HotkeyAction::LoadState => self.load_state(),
            HotkeyAction::PreviousSlot => self.select_slot(previous_slot(self.slot)),
            HotkeyAction::NextSlot => self.select_slot(next_slot(self.slot)),
            HotkeyAction::Snapshot => self.take_snapshot(),
            HotkeyAction::SwapScreens => self.swapped = !self.swapped,
        }
    }

    fn set_fast_forward(&mut self, enabled: bool) {
        self.fast_forward = enabled;
        if enabled {
            self.audio.clear();
        }
        self.update_title("");
    }

    fn select_slot(&mut self, slot: u8) {
        self.slot = slot;
        self.update_title("");
    }

    fn save_state(&mut self) {
        let path = self.current_slot_path();
        let result = self
            .core
            .save_state()
            .map_err(|error| error.to_string())
            .and_then(|state| std::fs::write(&path, state).map_err(|error| error.to_string()));
        self.update_title(&status_text("saved", &result));
    }

    fn load_state(&mut self) {
        let path = self.current_slot_path();
        let result = std::fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|state| {
                self.core
                    .load_state(&state)
                    .map_err(|error| error.to_string())
            });
        self.update_title(&status_text("loaded", &result));
    }

    fn take_snapshot(&mut self) {
        let result = self.write_snapshot();
        let status = match result {
            Ok(directory) => format!("(snapshot {})", directory.display()),
            Err(error) => format!("(snapshot failed: {error})"),
        };
        self.update_title(&status);
    }

    fn write_snapshot(&self) -> Result<PathBuf, String> {
        let savestate = self.core.save_state().map_err(|error| error.to_string())?;
        let frame = self.presenter.read_frame(&self.platform.gl);
        let snapshot = Snapshot {
            label: "play",
            savestate: &savestate,
            main_ram: self.core.main_ram(),
            frame: frame.as_ref(),
        };
        snapshot::write(Path::new(SAMPLES_DIR), &snapshot).map_err(|error| error.to_string())
    }

    // The core finishes booting the console during its first frame, which would wipe a state
    // loaded earlier, so the state waits until one frame has run.
    pub fn load_state_file(&mut self, path: &Path) -> Result<(), String> {
        let state = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        self.pending_state = Some(state);
        Ok(())
    }

    fn apply_pending_state(&mut self) {
        let Some(state) = self.pending_state.take() else {
            return;
        };
        if let Err(error) = self.core.load_state(&state) {
            self.update_title(&format!("(start state failed: {error})"));
        }
    }

    fn current_slot_path(&self) -> PathBuf {
        slot_path(&self.settings.save_dir, self.slot)
    }

    fn update_title(&mut self, status: &str) {
        let fast_forward = if self.fast_forward {
            " - fast forward"
        } else {
            ""
        };
        let title = format!(
            "Strange Journey - slot {}{fast_forward} {status}",
            self.slot
        );
        let _ = self.platform.window.set_title(title.trim_end());
    }
}

fn is_keyboard(event: &Event) -> bool {
    matches!(
        event,
        Event::KeyDown { .. } | Event::KeyUp { .. } | Event::TextInput { .. }
    )
}

fn is_mouse(event: &Event) -> bool {
    matches!(
        event,
        Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. } | Event::MouseMotion { .. }
    )
}

fn status_text(action: &str, result: &Result<(), String>) -> String {
    match result {
        Ok(()) => format!("({action})"),
        Err(error) => format!("(failed: {error})"),
    }
}

fn load_core(platform: &Platform, settings: &Settings) -> Result<Core, String> {
    let core_config = CoreConfig {
        core_path: settings.core.clone(),
        system_dir: settings.save_dir.clone(),
        save_dir: settings.save_dir.clone(),
        variables: core_variables(settings, Renderer::OpenGl),
    };
    let lookup_video = platform.video.clone();
    Core::load(
        &core_config,
        Box::new(move |name| lookup_video.gl_get_proc_address(name).cast()),
    )
    .map_err(|error| error.to_string())
}

fn create_presenter(gl: &glow::Context, core: &Core, av_info: AvInfo) -> Result<Presenter, String> {
    let presenter = Presenter::new(gl, av_info.max_width_px, av_info.max_height_px)?;
    if core.wants_opengl() {
        core.set_framebuffer(presenter.framebuffer.0.get());
        core.reset_gl_context();
    }
    Ok(presenter)
}

fn open_audio(platform: &Platform, av_info: AvInfo) -> Result<AudioQueue<i16>, String> {
    let desired = AudioSpecDesired {
        freq: Some(av_info.sample_rate_hz.round() as i32),
        channels: Some(2),
        samples: None,
    };
    platform.sdl.audio()?.open_queue::<i16, _>(None, &desired)
}

fn audio_target_bytes(av_info: AvInfo) -> u32 {
    let bytes_per_frame = av_info.sample_rate_hz / av_info.fps * 2.0 * 2.0;
    (bytes_per_frame * AUDIO_BUFFER_FRAMES) as u32
}
