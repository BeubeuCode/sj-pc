use std::sync::Arc;

use sdl2::controller::GameController;
use sdl2::video::{FullscreenType, GLContext, GLProfile, Window};
use sdl2::{EventPump, GameControllerSubsystem, Sdl, VideoSubsystem};
use sj_game::settings::VideoSettings;

pub struct Platform {
    pub sdl: Sdl,
    pub video: VideoSubsystem,
    pub window: Window,
    pub gl: Arc<glow::Context>,
    pub events: EventPump,
    controller_subsystem: GameControllerSubsystem,
    controllers: Vec<GameController>,
    _gl_context: GLContext,
}

impl Platform {
    pub fn new(window_width_px: u32, window_height_px: u32) -> Result<Self, String> {
        prefer_native_xbox_driver_on_macos();
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        configure_gl_attributes(&video);
        let window = video
            .window("Strange Journey", window_width_px, window_height_px)
            .opengl()
            .resizable()
            .allow_highdpi()
            .position_centered()
            .build()
            .map_err(|error| error.to_string())?;
        let gl_context = window.gl_create_context()?;
        video.gl_set_swap_interval(1)?;
        // SAFETY: the GL context we just created is current on this thread.
        let gl = unsafe {
            glow::Context::from_loader_function(|name| video.gl_get_proc_address(name).cast())
        };
        let controller_subsystem = sdl.game_controller()?;
        let events = sdl.event_pump()?;
        Ok(Self {
            sdl,
            video,
            window,
            gl: Arc::new(gl),
            events,
            controller_subsystem,
            controllers: Vec::new(),
            _gl_context: gl_context,
        })
    }

    pub fn apply_video_settings(&mut self, video_settings: &VideoSettings) -> Result<(), String> {
        self.video
            .gl_set_swap_interval(i32::from(video_settings.vsync))?;
        if video_settings.fullscreen {
            return self.window.set_fullscreen(FullscreenType::Desktop);
        }
        self.window
            .set_size(
                video_settings.window_width_px,
                video_settings.window_height_px,
            )
            .map_err(|error| error.to_string())?;
        self.window.set_position(
            sdl2::video::WindowPos::Centered,
            sdl2::video::WindowPos::Centered,
        );
        Ok(())
    }

    pub fn toggle_fullscreen(&mut self) {
        let next = match self.window.fullscreen_state() {
            FullscreenType::Off => FullscreenType::Desktop,
            _ => FullscreenType::Off,
        };
        let _ = self.window.set_fullscreen(next);
    }

    pub fn open_controller(&mut self, device_index: u32) {
        if let Ok(controller) = self.controller_subsystem.open(device_index) {
            self.controllers.push(controller);
        }
    }

    pub fn close_controller(&mut self, instance_id: u32) {
        self.controllers
            .retain(|controller| controller.instance_id() != instance_id);
    }

    pub fn controller_names(&self) -> Vec<String> {
        self.controllers.iter().map(GameController::name).collect()
    }

    pub fn to_drawable(&self, window_x: i32, window_y: i32) -> (i32, i32) {
        let (window_width, window_height) = self.window.size();
        let (drawable_width, drawable_height) = self.window.drawable_size();
        let x = i64::from(window_x) * i64::from(drawable_width) / i64::from(window_width.max(1));
        let y = i64::from(window_y) * i64::from(drawable_height) / i64::from(window_height.max(1));
        (x as i32, y as i32)
    }
}

// SDL's own HID driver for Bluetooth Xbox controllers lags behind Microsoft firmware updates.
// Apple's GameController framework tracks them and needs no Input Monitoring permission.
fn prefer_native_xbox_driver_on_macos() {
    if cfg!(target_os = "macos") {
        sdl2::hint::set("SDL_JOYSTICK_HIDAPI_XBOX", "0");
    }
}

fn configure_gl_attributes(video: &VideoSubsystem) {
    let attributes = video.gl_attr();
    attributes.set_context_profile(GLProfile::Core);
    attributes.set_context_version(3, 3);
    attributes.set_context_flags().forward_compatible().set();
    attributes.set_depth_size(24);
    attributes.set_stencil_size(8);
}
