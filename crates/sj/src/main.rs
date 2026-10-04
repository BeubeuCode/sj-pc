mod app;
mod capture;
mod core_options;
mod host_input;
mod launcher;
mod platform;
mod present;
mod savestate;

use std::path::Path;
use std::process::ExitCode;

use crate::platform::Platform;
use sj_game::settings::{load_or_default, Settings};

const SETTINGS_PATH: &str = "sj.toml";
const SKIP_LAUNCHER_FLAG: &str = "--play";

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
    let (platform, settings) = if skip_launcher {
        (platform, settings)
    } else {
        match launcher::run(platform, settings, settings_path, load_problem) {
            Some(chosen) => chosen,
            None => return Ok(()),
        }
    };
    app::App::new(platform, settings)?.run();
    Ok(())
}
