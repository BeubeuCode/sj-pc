mod app;
mod capture;
mod dev_panel;
mod host_input;
mod hud;
mod launcher;
mod mouse_points;
mod overlay;
mod platform;
mod present;
mod savestate;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::platform::Platform;
use sj_game::settings::{load_or_default, Settings};

const SETTINGS_PATH: &str = "sj.toml";
const SKIP_LAUNCHER_FLAG: &str = "--play";
const START_STATE_FLAG: &str = "--state";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sj: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let settings_path = Path::new(SETTINGS_PATH);
    let skip_launcher = std::env::args().any(|argument| argument == SKIP_LAUNCHER_FLAG);
    let (settings, load_problem) = match load_or_default(settings_path) {
        Ok(settings) => (settings, None),
        Err(error) if !skip_launcher => (
            Settings::default(),
            Some(format!(
                "{SETTINGS_PATH} could not be used ({error}). Showing defaults."
            )),
        ),
        Err(error) => return Err(error.to_string()),
    };
    let platform = Platform::new(
        settings.video.window_width_px,
        settings.video.window_height_px,
    )?;
    let (platform, settings, resume_state) = if skip_launcher {
        (platform, settings, None)
    } else {
        match launcher::run(platform, settings, settings_path, load_problem) {
            Some(chosen) => chosen,
            None => return Ok(()),
        }
    };
    let mut app = app::App::new(platform, settings)?;
    let flag_state = flag_value(START_STATE_FLAG).map(PathBuf::from);
    if let Some(state_path) = resume_state.or(flag_state) {
        app.load_state_file(&state_path)?;
    }
    app.run();
    Ok(())
}

fn flag_value(flag: &str) -> Option<String> {
    let arguments: Vec<String> = std::env::args().collect();
    let position = arguments.iter().position(|argument| argument == flag)?;
    arguments.get(position + 1).cloned()
}
