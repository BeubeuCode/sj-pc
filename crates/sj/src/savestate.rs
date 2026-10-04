use std::path::{Path, PathBuf};
use std::time::Duration;

pub const SLOT_COUNT: u8 = 10;
pub const AUTOSAVE_INTERVAL: Duration = Duration::from_mins(5);

pub fn slot_path(save_dir: &Path, slot: u8) -> PathBuf {
    save_dir.join(format!("slot{slot}.state"))
}

pub fn autosave_path(save_dir: &Path) -> PathBuf {
    save_dir.join("autosave.state")
}

// Writes through a temp file and keeps the previous autosave, so a crash mid-write or an autosave
// taken at a bad moment never leaves the player without a usable state.
pub fn write_autosave(save_dir: &Path, state: &[u8]) -> std::io::Result<()> {
    let path = autosave_path(save_dir);
    let temp_path = save_dir.join("autosave.state.tmp");
    std::fs::write(&temp_path, state)?;
    if path.exists() {
        std::fs::rename(&path, save_dir.join("autosave.previous.state"))?;
    }
    std::fs::rename(&temp_path, &path)
}

pub fn next_slot(slot: u8) -> u8 {
    (slot + 1) % SLOT_COUNT
}

pub fn previous_slot(slot: u8) -> u8 {
    (slot + SLOT_COUNT - 1) % SLOT_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_wrap_both_ways() {
        assert_eq!(next_slot(9), 0);
        assert_eq!(previous_slot(0), 9);
        assert_eq!(next_slot(3), 4);
    }

    #[test]
    fn autosave_keeps_the_previous_one() {
        let save_dir = std::env::temp_dir().join(format!("sj-autosave-{}", std::process::id()));
        std::fs::create_dir_all(&save_dir).unwrap();
        write_autosave(&save_dir, b"first").unwrap();
        write_autosave(&save_dir, b"second").unwrap();
        assert_eq!(std::fs::read(autosave_path(&save_dir)).unwrap(), b"second");
        assert_eq!(
            std::fs::read(save_dir.join("autosave.previous.state")).unwrap(),
            b"first"
        );
        std::fs::remove_dir_all(&save_dir).unwrap();
    }

    #[test]
    fn slot_files_live_in_save_dir() {
        assert_eq!(
            slot_path(Path::new("saves"), 2),
            PathBuf::from("saves/slot2.state")
        );
    }
}
