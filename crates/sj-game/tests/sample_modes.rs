use std::path::Path;

use sj_game::game_api::GameApi;
use sj_game::game_mode::{read, GameMode};

struct Snapshot {
    ram: Vec<u8>,
}

impl GameApi for Snapshot {
    fn main_ram(&self) -> &[u8] {
        &self.ram
    }

    fn main_ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram
    }
}

const EXPECTED_BY_LABEL_PREFIX: [(&str, GameMode); 15] = [
    ("title", GameMode::Title),
    ("intro", GameMode::Event),
    ("walk-1-", GameMode::Event),
    ("end-name", GameMode::TextEntry),
    ("story", GameMode::Facility),
    ("cr-talk", GameMode::Facility),
    ("ymenu-1", GameMode::Menu),
    ("ymenu-2", GameMode::Menu),
    ("ymenu-sub", GameMode::Menu),
    ("ymenu-closed", GameMode::Facility),
    ("xmenu", GameMode::MissionLog),
    ("dungeon-talk", GameMode::Dungeon { dialogue: true }),
    ("dungeon-1", GameMode::Dungeon { dialogue: false }),
    ("md-deck", GameMode::Facility),
    ("deck-b", GameMode::ShipScene),
];

#[test]
#[ignore = "needs lab snapshots in re/samples"]
fn lab_snapshots_decode_to_the_mode_seen_on_screen() {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples");
    let Ok(entries) = std::fs::read_dir(&samples) else {
        eprintln!("skipped: no re/samples");
        return;
    };
    let mut checked = 0;
    for entry in entries.flatten() {
        let label = entry.file_name().to_string_lossy().into_owned();
        let Some((_, expected)) = EXPECTED_BY_LABEL_PREFIX
            .iter()
            .find(|(prefix, _)| label.starts_with(prefix))
        else {
            continue;
        };
        let ram = std::fs::read(entry.path().join("main_ram.bin")).unwrap();
        assert_eq!(read(&Snapshot { ram }), *expected, "sample {label}");
        checked += 1;
    }
    eprintln!("checked {checked} samples");
}
