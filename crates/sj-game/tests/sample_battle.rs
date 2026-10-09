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

// The bottom-screen page and the demon stock, as the game's own bottom screen showed them.
#[test]
#[ignore = "needs battle snapshots in re/samples"]
fn bottom_page_and_stock_match_the_games_bottom_screen() {
    use sj_game::battle::{bottom_page, BottomPage};
    use sj_game::stock::read_stock;
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples");
    let pages = [
        ("bm-commands-1", BottomPage::EnemyStatus),
        ("bm-partypage-1", BottomPage::PartyStatus),
        ("battle-summon-1", BottomPage::SummonList),
        ("bm-results-1", BottomPage::Other(5)),
    ];
    for (label, page) in pages {
        let Ok(ram) = std::fs::read(samples.join(label).join("main_ram.bin")) else {
            eprintln!("skipped: no {label}");
            continue;
        };
        let snapshot = Snapshot { ram };
        assert_eq!(bottom_page(&snapshot), Some(page), "{label}");
        let stock: Vec<_> = read_stock(&snapshot)
            .iter()
            .map(|demon| (demon.name.clone(), demon.level, demon.max_hp, demon.max_mp))
            .collect();
        assert_eq!(stock, [(Some("Pixie".to_string()), 2, 46, 25)], "{label}");
    }
}

// The second demon (Knocker) and the Summon list cursor moving to it and wrapping back.
#[test]
#[ignore = "needs the two-demon lab snapshots in re/samples"]
fn two_demon_stock_and_summon_cursor() {
    use sj_game::battle::list_cursor;
    use sj_game::stock::read_stock;
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples");
    let Ok(ram) = std::fs::read(samples.join("two-demons-1/main_ram.bin")) else {
        eprintln!("skipped: no two-demons-1");
        return;
    };
    let names: Vec<_> = read_stock(&Snapshot { ram })
        .into_iter()
        .map(|demon| (demon.name, demon.level))
        .collect();
    assert_eq!(
        names,
        [
            (Some("Pixie".to_string()), 2),
            (Some("Knocker".to_string()), 1)
        ]
    );
    for (label, cursor) in [("sl-0-1", 0), ("bl-1a-1", 1), ("bl-2a-1", 0)] {
        let ram = std::fs::read(samples.join(label).join("main_ram.bin")).unwrap();
        assert_eq!(list_cursor(&Snapshot { ram }), Some(cursor), "{label}");
    }
}

// Race, affinities and skills as the game's own cards show them in the two-demon fight: Pixie
// (Fairy, St Fire, Agi and Dia), Knocker (Jirae, Wk Fire, St Ice, Bufu), the hero's Fire Shot.
#[test]
#[ignore = "needs the two-demon lab snapshots in re/samples"]
fn race_affinities_and_skills_match_the_games_cards() {
    use sj_game::battle::{read_enemies, read_party};
    use sj_game::demon_data::Affinity;
    use sj_game::stock::read_stock;
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples");
    let Ok(ram) = std::fs::read(samples.join("bl-1a-1/main_ram.bin")) else {
        eprintln!("skipped: no bl-1a-1");
        return;
    };
    let game = Snapshot { ram };
    let stock = read_stock(&game);
    assert_eq!(stock[0].race.as_deref(), Some("Fairy"));
    assert_eq!(stock[0].skills, ["Agi", "Dia"]);
    assert_eq!(stock[0].affinities.unwrap()[2], Affinity::Strong);
    let knocker = stock[1].affinities.unwrap();
    assert_eq!((knocker[2], knocker[3]), (Affinity::Weak, Affinity::Strong));
    assert_eq!(stock[1].skills, ["Bufu"]);
    let hero = &read_party(&game)[0];
    assert_eq!((hero.race.as_deref(), hero.affinities), (None, None));
    assert_eq!(hero.skills, ["Fire Shot"]);
    let enemies = read_enemies(&game);
    assert_eq!(
        enemies[0].skills,
        ["Agi"],
        "a Lv2 Pixie enemy only knows Agi"
    );
    assert!(enemies[1].unknown, "the game shows ??? UNKNOWN for it");
    assert_eq!(enemies[1].race, None);
}

// What the game's enemy card reveals as the Analyze gauge fills: Pixie at 10 shows its name but
// "??" affinities and "???" skills, Pixie at 70 shows them; a demon never met is "??? UNKNOWN".
#[test]
#[ignore = "needs battle snapshots in re/samples"]
fn enemy_details_follow_the_analyze_gauge() {
    use sj_game::battle::read_enemies;
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../re/samples");
    let cases = [
        ("battle-first-1", [(true, false), (true, false)]),
        ("battle-dungeon-1", [(false, false), (true, false)]),
        ("battle-dungeon-talk-1", [(false, false), (true, false)]),
        ("two-demons-1", [(false, true), (true, false)]),
    ];
    for (label, expected) in cases {
        let Ok(ram) = std::fs::read(samples.join(label).join("main_ram.bin")) else {
            eprintln!("skipped: no {label}");
            continue;
        };
        let shown: Vec<_> = read_enemies(&Snapshot { ram })
            .iter()
            .map(|enemy| (enemy.unknown, enemy.affinities.is_some()))
            .collect();
        assert_eq!(shown, expected, "{label}");
    }
}
