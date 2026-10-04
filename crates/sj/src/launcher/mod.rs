mod binding_capture;
mod controls_section;
mod display_section;
mod game_section;
mod mouse_points;
mod sound_section;
mod state;

use std::path::Path;

use sdl2::event::Event;
use sj_game::settings::Settings;

use crate::capture::{encode_ppm, request_from_env};
use crate::host_input::AxisThresholds;
use crate::platform::Platform;
use crate::present::read_default_framebuffer;
use binding_capture::{apply_capture, captured_input, Captured};
use mouse_points::mouse_event_in_drawable_pixels;
use state::{LauncherState, Outcome};

const BACKGROUND_RGBA: [f32; 4] = [0.07, 0.07, 0.09, 1.0];

pub fn run(
    mut platform: Platform,
    settings: Settings,
    settings_path: &Path,
    message: Option<String>,
) -> Option<(Platform, Settings)> {
    let _ = platform.window.set_title("Strange Journey - Launcher");
    let mut egui = egui_sdl2::EguiGlow::new(&platform.window, platform.gl.clone(), None, false);
    let mut state = LauncherState::new(settings, settings_path, message);
    let capture = request_from_env();
    let mut frames_drawn: u64 = 0;
    while state.outcome.is_none() {
        let events: Vec<Event> = platform.events.poll_iter().collect();
        for event in &events {
            handle_event(&mut platform, &mut egui, &mut state, event);
        }
        egui.run_ui(|root| state.show(root));
        egui.clear(BACKGROUND_RGBA);
        egui.paint();
        if capture
            .as_ref()
            .is_some_and(|request| frames_drawn >= request.after_frames)
        {
            capture_launcher(
                &platform,
                capture.as_ref().map(|request| request.path.as_path()),
            );
            state.outcome = Some(Outcome::Quit);
        }
        platform.window.gl_swap_window();
        frames_drawn += 1;
    }
    egui.destroy();
    match state.outcome {
        Some(Outcome::Play) => Some((platform, state.settings)),
        _ => None,
    }
}

fn handle_event(
    platform: &mut Platform,
    egui: &mut egui_sdl2::EguiGlow,
    state: &mut LauncherState,
    event: &Event,
) {
    match event {
        Event::Quit { .. } => state.outcome = Some(Outcome::Quit),
        Event::DropFile { filename, .. } => state.set_rom_path(filename.clone()),
        Event::ControllerDeviceAdded { which, .. } => platform.open_controller(*which),
        Event::ControllerDeviceRemoved { which, .. } => platform.close_controller(*which),
        _ => {}
    }
    state.controller_names = platform.controller_names();
    let thresholds =
        AxisThresholds::from_deadzone_percent(state.settings.controller.stick_deadzone_percent);
    let captured = captured_input(event, thresholds);
    if let Some(Captured::Pad(name)) = &captured {
        state.last_pad_input = Some(name.clone());
    }
    if let Some(target) = state.capture {
        if let Some(captured) = captured {
            if apply_capture(&mut state.settings, target, captured) {
                state.capture = None;
            }
            return;
        }
    }
    let event = mouse_event_in_drawable_pixels(event, |x, y| platform.to_drawable(x, y));
    let _ = egui.on_event(&platform.window, &event);
}

fn capture_launcher(platform: &Platform, path: Option<&Path>) {
    let Some(path) = path else {
        return;
    };
    let drawable_px = platform.window.drawable_size();
    let rgba = read_default_framebuffer(&platform.gl, drawable_px);
    if let Err(error) = std::fs::write(path, encode_ppm(drawable_px.0, drawable_px.1, &rgba)) {
        eprintln!("sj: capture failed: {error}");
    }
}
