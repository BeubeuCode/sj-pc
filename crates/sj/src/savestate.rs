use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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

pub struct SavedState {
    pub label: String,
    pub path: PathBuf,
    pub age: Duration,
}

// The states that exist on disk, newest first: autosaves and slots.
pub fn saved_states(save_dir: &Path, now: SystemTime) -> Vec<SavedState> {
    let candidates = [
        ("Autosave".to_string(), autosave_path(save_dir)),
        (
            "Previous autosave".to_string(),
            save_dir.join("autosave.previous.state"),
        ),
    ]
    .into_iter()
    .chain((0..SLOT_COUNT).map(|slot| (format!("Slot {slot}"), slot_path(save_dir, slot))));
    let mut states: Vec<SavedState> = candidates
        .filter_map(|(label, path)| {
            let modified = std::fs::metadata(&path)
                .and_then(|meta| meta.modified())
                .ok()?;
            let age = now.duration_since(modified).unwrap_or_default();
            Some(SavedState { label, path, age })
        })
        .collect();
    states.sort_by_key(|state| state.age);
    states
}

pub fn age_text(age: Duration) -> String {
    let minutes = age.as_secs() / 60;
    match minutes {
        0 => "just now".to_string(),
        1..60 => format!("{minutes} min ago"),
        60..1440 => format!("{} h ago", minutes / 60),
        _ => format!("{} days ago", minutes / 1440),
    }
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
    fn lists_existing_states_newest_first() {
        let save_dir = std::env::temp_dir().join(format!("sj-states-{}", std::process::id()));
        std::fs::create_dir_all(&save_dir).unwrap();
        std::fs::write(slot_path(&save_dir, 3), b"old").unwrap();
        std::fs::write(autosave_path(&save_dir), b"new").unwrap();
        let later = SystemTime::now() + Duration::from_secs(600);
        let labels: Vec<String> = saved_states(&save_dir, later)
            .into_iter()
            .map(|state| state.label)
            .collect();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&"Slot 3".to_string()));
        std::fs::remove_dir_all(&save_dir).unwrap();
    }

    #[test]
    fn ages_read_like_a_person_would_say_them() {
        assert_eq!(age_text(Duration::from_secs(20)), "just now");
        assert_eq!(age_text(Duration::from_mins(5)), "5 min ago");
        assert_eq!(age_text(Duration::from_hours(3)), "3 h ago");
        assert_eq!(age_text(Duration::from_hours(48)), "2 days ago");
    }

    #[test]
    fn slot_files_live_in_save_dir() {
        assert_eq!(
            slot_path(Path::new("saves"), 2),
            PathBuf::from("saves/slot2.state")
        );
    }
}
