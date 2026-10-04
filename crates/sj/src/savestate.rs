use std::path::{Path, PathBuf};

pub const SLOT_COUNT: u8 = 10;

pub fn slot_path(save_dir: &Path, slot: u8) -> PathBuf {
    save_dir.join(format!("slot{slot}.state"))
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
    fn slot_files_live_in_save_dir() {
        assert_eq!(
            slot_path(Path::new("saves"), 2),
            PathBuf::from("saves/slot2.state")
        );
    }
}
