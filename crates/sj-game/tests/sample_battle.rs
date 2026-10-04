use std::path::Path;

use sj_game::battle::read_enemies;
use sj_game::game_api::GameApi;

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

// Two enemies, 29/29 HP and 13/13 MP each, in the first tutorial battle.
#[test]
#[ignore = "needs the battle-first lab snapshot in re/samples"]
fn first_battle_snapshot_shows_two_full_health_enemies() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples/battle-first-1/main_ram.bin");
    let Ok(ram) = std::fs::read(&path) else {
        eprintln!("skipped: no {}", path.display());
        return;
    };
    let enemies = read_enemies(&Snapshot { ram });
    assert_eq!(enemies.len(), 2);
    assert!(enemies
        .iter()
        .all(|enemy| (enemy.hp, enemy.max_hp, enemy.mp) == (29, 29, 13)));
}
