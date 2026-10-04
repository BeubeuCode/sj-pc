use std::path::PathBuf;

use sj_emu::CoreConfig;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn core_path() -> PathBuf {
    let extension = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    repo_root().join(format!("cores/melondsds_libretro.{extension}"))
}

pub fn software_config(save_dir: PathBuf) -> CoreConfig {
    let variables = [
        ("melonds_render_mode", "software"),
        ("melonds_boot_mode", "direct"),
        ("melonds_number_of_screen_layouts", "1"),
        ("melonds_screen_layout1", "top-bottom"),
        ("melonds_screen_gap", "0"),
    ];
    CoreConfig {
        core_path: core_path(),
        system_dir: save_dir.clone(),
        save_dir,
        variables: variables
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}
