use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sj_emu::{Core, CoreConfig, Frame, Pointer};
use sj_game::core_options::{core_variables, Renderer};
use sj_game::game_api::GameApi;
use sj_game::image::RgbImage;
use sj_game::input_script::{frame_masks, parse, Step};
use sj_game::settings::load_or_default;
use sj_game::snapshot::{self, Snapshot};

const SAMPLES_DIR: &str = "re/samples";
const LAB_SAVE_DIR: &str = "re/lab-saves";
const USAGE: &str = "usage: sj-lab <script.txt>";

struct Lab {
    core: Core,
    last_frame: Option<RgbImage>,
    frames_run: u64,
}

fn main() -> ExitCode {
    let Some(script_path) = std::env::args().nth(1) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    match run(Path::new(&script_path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sj-lab: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(script_path: &Path) -> Result<(), String> {
    let script = std::fs::read_to_string(script_path)
        .map_err(|error| format!("{}: {error}", script_path.display()))?;
    let steps = parse(&script).map_err(|error| error.to_string())?;
    let mut lab = Lab::boot()?;
    for step in &steps {
        lab.perform(step)?;
    }
    println!("done after {} frames", lab.frames_run);
    Ok(())
}

impl Lab {
    fn boot() -> Result<Self, String> {
        let settings = load_or_default(Path::new("sj.toml")).map_err(|error| error.to_string())?;
        std::fs::create_dir_all(LAB_SAVE_DIR).map_err(|error| error.to_string())?;
        let config = CoreConfig {
            core_path: settings.core.clone(),
            system_dir: PathBuf::from(LAB_SAVE_DIR),
            save_dir: PathBuf::from(LAB_SAVE_DIR),
            variables: core_variables(&settings, Renderer::Software),
        };
        let mut core = Core::load(&config, Box::new(|_| std::ptr::null()))
            .map_err(|error| error.to_string())?;
        core.load_game(&settings.rom)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            core,
            last_frame: None,
            frames_run: 0,
        })
    }

    fn perform(&mut self, step: &Step) -> Result<(), String> {
        match step {
            Step::Snapshot { label } => self.snapshot(label),
            Step::SaveState { path } => self.save_state(path),
            Step::LoadState { path } => self.load_state(path),
            timed => {
                for mask in frame_masks(timed) {
                    self.run_frame(mask);
                }
                Ok(())
            }
        }
    }

    fn run_frame(&mut self, joypad_mask: u16) {
        self.core.set_input(joypad_mask, Pointer::default());
        let output = self.core.run_frame();
        if let Some(Frame::Software {
            width_px,
            height_px,
            xrgb_pixels,
        }) = output.frame
        {
            self.last_frame = Some(RgbImage::from_xrgb(width_px, height_px, &xrgb_pixels));
        }
        self.frames_run += 1;
    }

    fn snapshot(&self, label: &str) -> Result<(), String> {
        let savestate = self.core.save_state().map_err(|error| error.to_string())?;
        let snapshot = Snapshot {
            label,
            savestate: &savestate,
            main_ram: self.core.main_ram(),
            frame: self.last_frame.as_ref(),
        };
        let directory = snapshot::write(Path::new(SAMPLES_DIR), &snapshot)
            .map_err(|error| error.to_string())?;
        println!(
            "frame {}: snapshot {}",
            self.frames_run,
            directory.display()
        );
        Ok(())
    }

    fn save_state(&self, path: &Path) -> Result<(), String> {
        let state = self.core.save_state().map_err(|error| error.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(path, state).map_err(|error| format!("{}: {error}", path.display()))
    }

    fn load_state(&mut self, path: &Path) -> Result<(), String> {
        let state = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        self.core
            .load_state(&state)
            .map_err(|error| error.to_string())
    }
}
