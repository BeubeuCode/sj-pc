use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audio::{AudioSettings, MAX_VOLUME_PERCENT};
use crate::input::{default_bindings, fill_missing_with_defaults, Bindings};
use crate::layout::PipSettings;

pub const MAX_SCALE: u32 = 8;
pub const MIN_DEADZONE_PERCENT: u8 = 10;
pub const MAX_DEADZONE_PERCENT: u8 = 90;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub rom: PathBuf,
    pub core: PathBuf,
    pub save_dir: PathBuf,
    pub video: VideoSettings,
    pub pip: PipSettings,
    pub audio: AudioSettings,
    pub controller: ControllerSettings,
    pub hotkeys: Hotkeys,
    pub bindings: Bindings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalingFilter {
    Smooth,
    Sharp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ControllerSettings {
    pub stick_deadzone_percent: u8,
}

impl Default for ControllerSettings {
    fn default() -> Self {
        Self {
            stick_deadzone_percent: 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    FastForward,
    TogglePip,
    ToggleFullscreen,
    SaveState,
    LoadState,
    PreviousSlot,
    NextSlot,
    Snapshot,
}

impl HotkeyAction {
    pub const ALL: [HotkeyAction; 8] = [
        HotkeyAction::FastForward,
        HotkeyAction::TogglePip,
        HotkeyAction::ToggleFullscreen,
        HotkeyAction::SaveState,
        HotkeyAction::LoadState,
        HotkeyAction::PreviousSlot,
        HotkeyAction::NextSlot,
        HotkeyAction::Snapshot,
    ];

    pub fn label(self) -> &'static str {
        match self {
            HotkeyAction::FastForward => "Fast forward (hold)",
            HotkeyAction::TogglePip => "Show/hide touch screen",
            HotkeyAction::ToggleFullscreen => "Fullscreen",
            HotkeyAction::SaveState => "Save state",
            HotkeyAction::LoadState => "Load state",
            HotkeyAction::PreviousSlot => "Previous slot",
            HotkeyAction::NextSlot => "Next slot",
            HotkeyAction::Snapshot => "Research snapshot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoSettings {
    pub scale: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub window_width_px: u32,
    pub window_height_px: u32,
    pub filter: ScalingFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub fast_forward: String,
    pub toggle_pip: String,
    pub toggle_fullscreen: String,
    pub save_state: String,
    pub load_state: String,
    pub next_slot: String,
    pub previous_slot: String,
    pub snapshot: String,
}

impl Hotkeys {
    pub fn key_for(&self, action: HotkeyAction) -> &str {
        match action {
            HotkeyAction::FastForward => &self.fast_forward,
            HotkeyAction::TogglePip => &self.toggle_pip,
            HotkeyAction::ToggleFullscreen => &self.toggle_fullscreen,
            HotkeyAction::SaveState => &self.save_state,
            HotkeyAction::LoadState => &self.load_state,
            HotkeyAction::PreviousSlot => &self.previous_slot,
            HotkeyAction::NextSlot => &self.next_slot,
            HotkeyAction::Snapshot => &self.snapshot,
        }
    }

    pub fn set_key(&mut self, action: HotkeyAction, key_name: String) {
        let slot = match action {
            HotkeyAction::FastForward => &mut self.fast_forward,
            HotkeyAction::TogglePip => &mut self.toggle_pip,
            HotkeyAction::ToggleFullscreen => &mut self.toggle_fullscreen,
            HotkeyAction::SaveState => &mut self.save_state,
            HotkeyAction::LoadState => &mut self.load_state,
            HotkeyAction::PreviousSlot => &mut self.previous_slot,
            HotkeyAction::NextSlot => &mut self.next_slot,
            HotkeyAction::Snapshot => &mut self.snapshot,
        };
        *slot = key_name;
    }

    pub fn action_for(&self, key_name: &str) -> Option<HotkeyAction> {
        HotkeyAction::ALL
            .into_iter()
            .find(|action| self.key_for(*action) == key_name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read or write {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid settings file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("video.scale must be between 1 and {MAX_SCALE}, got {0}")]
    InvalidScale(u32),
    #[error("pip.height_fraction must be between 0.1 and 1.0, got {0}")]
    InvalidPipHeight(f32),
    #[error("audio.volume_percent must be at most {MAX_VOLUME_PERCENT}, got {0}")]
    InvalidVolume(u8),
    #[error("controller.stick_deadzone_percent must be between {MIN_DEADZONE_PERCENT} and {MAX_DEADZONE_PERCENT}, got {0}")]
    InvalidDeadzone(u8),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rom: PathBuf::from("roms/sj_usa.nds"),
            core: default_core_path(),
            save_dir: PathBuf::from("saves"),
            video: VideoSettings::default(),
            pip: PipSettings::default(),
            audio: AudioSettings::default(),
            controller: ControllerSettings::default(),
            hotkeys: Hotkeys::default(),
            bindings: default_bindings(),
        }
    }
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            scale: 4,
            fullscreen: false,
            vsync: true,
            window_width_px: 1280,
            window_height_px: 960,
            filter: ScalingFilter::Smooth,
        }
    }
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            fast_forward: "Tab".into(),
            toggle_pip: "P".into(),
            toggle_fullscreen: "F11".into(),
            save_state: "F5".into(),
            load_state: "F8".into(),
            next_slot: "F7".into(),
            previous_slot: "F6".into(),
            snapshot: "F9".into(),
        }
    }
}

fn default_core_path() -> PathBuf {
    let extension = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    PathBuf::from(format!("cores/melondsds_libretro.{extension}"))
}

pub fn parse(text: &str) -> Result<Settings, SettingsError> {
    let mut settings: Settings = toml::from_str(text)?;
    fill_missing_with_defaults(&mut settings.bindings);
    validate(&settings)?;
    Ok(settings)
}

pub fn to_toml(settings: &Settings) -> String {
    toml::to_string_pretty(settings).expect("settings always serialize")
}

pub fn load_or_default(path: &Path) -> Result<Settings, SettingsError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(source) => Err(SettingsError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    std::fs::write(path, to_toml(settings)).map_err(|source| SettingsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate(settings: &Settings) -> Result<(), SettingsError> {
    let scale = settings.video.scale;
    if !(1..=MAX_SCALE).contains(&scale) {
        return Err(SettingsError::InvalidScale(scale));
    }
    let pip_height = settings.pip.height_fraction;
    if !(0.1..=1.0).contains(&pip_height) {
        return Err(SettingsError::InvalidPipHeight(pip_height));
    }
    let volume = settings.audio.volume_percent;
    if volume > MAX_VOLUME_PERCENT {
        return Err(SettingsError::InvalidVolume(volume));
    }
    let deadzone = settings.controller.stick_deadzone_percent;
    if !(MIN_DEADZONE_PERCENT..=MAX_DEADZONE_PERCENT).contains(&deadzone) {
        return Err(SettingsError::InvalidDeadzone(deadzone));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::NdsButton;
    use crate::layout::Corner;

    #[test]
    fn defaults_round_trip_through_toml() {
        let defaults = Settings::default();
        assert_eq!(parse(&to_toml(&defaults)).unwrap(), defaults);
    }

    #[test]
    fn empty_file_gives_defaults() {
        assert_eq!(parse("").unwrap(), Settings::default());
    }

    #[test]
    fn partial_file_overrides_only_what_it_sets() {
        let settings = parse("[video]\nscale = 2\n[pip]\ncorner = \"top-left\"\n").unwrap();
        assert_eq!(settings.video.scale, 2);
        assert!(settings.video.vsync);
        assert_eq!(settings.pip.corner, Corner::TopLeft);
    }

    #[test]
    fn partial_bindings_keep_defaults_for_other_buttons() {
        let settings = parse("[bindings]\na = [\"Space\"]\n").unwrap();
        assert_eq!(settings.bindings[&NdsButton::A], ["Space"]);
        assert_eq!(settings.bindings[&NdsButton::B], ["Z", "pad:b"]);
    }

    #[test]
    fn scale_out_of_range_is_rejected() {
        assert!(matches!(
            parse("[video]\nscale = 0\n"),
            Err(SettingsError::InvalidScale(0))
        ));
        assert!(matches!(
            parse("[video]\nscale = 9\n"),
            Err(SettingsError::InvalidScale(9))
        ));
    }

    #[test]
    fn tiny_pip_is_rejected() {
        assert!(matches!(
            parse("[pip]\nheight_fraction = 0.01\n"),
            Err(SettingsError::InvalidPipHeight(_))
        ));
    }

    #[test]
    fn unknown_corner_is_a_parse_error() {
        assert!(matches!(
            parse("[pip]\ncorner = \"middle\"\n"),
            Err(SettingsError::Parse(_))
        ));
    }

    #[test]
    fn loud_volume_and_tiny_deadzone_are_rejected() {
        assert!(matches!(
            parse("[audio]\nvolume_percent = 101\n"),
            Err(SettingsError::InvalidVolume(101))
        ));
        assert!(matches!(
            parse("[controller]\nstick_deadzone_percent = 5\n"),
            Err(SettingsError::InvalidDeadzone(5))
        ));
    }

    #[test]
    fn sharp_filter_parses() {
        assert_eq!(
            parse("[video]\nfilter = \"sharp\"\n").unwrap().video.filter,
            ScalingFilter::Sharp
        );
    }

    #[test]
    fn hotkeys_round_trip_through_actions() {
        let mut hotkeys = Hotkeys::default();
        hotkeys.set_key(HotkeyAction::SaveState, "F1".into());
        assert_eq!(hotkeys.key_for(HotkeyAction::SaveState), "F1");
        assert_eq!(hotkeys.action_for("F1"), Some(HotkeyAction::SaveState));
        assert_eq!(hotkeys.action_for("Tab"), Some(HotkeyAction::FastForward));
        assert_eq!(hotkeys.action_for("F12"), None);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let settings = load_or_default(Path::new("/definitely/not/here/sj.toml")).unwrap();
        assert_eq!(settings, Settings::default());
    }
}
