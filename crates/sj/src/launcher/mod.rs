mod binding_capture;
mod controls_section;
mod display_section;
mod game_section;
mod sound_section;
mod state;

use std::path::{Path, PathBuf};

use sdl2::event::Event;
use sj_game::settings::Settings;

use crate::capture::{encode_ppm, request_from_env};
use crate::host_input::AxisThresholds;
use crate::overlay::Overlay;
use crate::platform::Platform;
use crate::present::read_default_framebuffer;
use crate::savestate::autosave_path;
use binding_capture::{apply_capture, captured_input, Captured};
use state::{LauncherState, Outcome};

const BACKGROUND_RGBA: [f32; 4] = [0.07, 0.07, 0.09, 1.0];

pub fn run(
    mut platform: Platform,
    settings: Settings,
    settings_path: &Path,
    message: Option<String>,
) -> Option<(Platform, Settings, Option<PathBuf>)> {
    let _ = platform.window.set_title("Strange Journey - Launcher");
    let mut overlay = Overlay::new(&platform);
    let mut state = LauncherState::new(settings, settings_path, message);
    let capture = request_from_env();
    let mut frames_drawn: u64 = 0;
    while state.outcome.is_none() {
        let events: Vec<Event> = platform.events.poll_iter().collect();
        for event in &events {
            handle_event(&mut platform, &mut overlay, &mut state, event);
        }
        overlay.clear(BACKGROUND_RGBA);
        overlay.draw(|root| state.show(root));
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
    drop(overlay);
    match state.outcome {
        Some(Outcome::Play) => Some((platform, state.settings, None)),
        Some(Outcome::Resume) => {
            let start_state = autosave_path(&state.settings.save_dir);
            Some((platform, state.settings, Some(start_state)))
        }
        _ => None,
    }
}

fn handle_event(
    platform: &mut Platform,
    overlay: &mut Overlay,
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
    overlay.handle_event(platform, event);
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
