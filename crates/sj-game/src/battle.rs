use crate::addr::Arm9Addr;
use crate::addresses::{
    BATTLE_ENEMY_SLOT_COUNT, BATTLE_FIRST_ENEMY_SLOT, BATTLE_PAGE_ENEMY_STATUS,
    BATTLE_PAGE_PARTY_STATUS, BATTLE_PAGE_SUMMON_LIST, BATTLE_PARTY_SLOT_COUNT,
    BATTLE_SCENE_UNIT_TABLE_OFFSET, BATTLE_TABLE_HERO_UNIT_OFFSET, BATTLE_UI_LIST_CURSOR_OFFSET,
    BATTLE_UI_PAGE_OFFSET, BATTLE_UNIT_DEMON_ID_OFFSET, BATTLE_UNIT_HP_OFFSET,
    BATTLE_UNIT_LEVEL_OFFSET, BATTLE_UNIT_MAX_HP_OFFSET, BATTLE_UNIT_MAX_MP_OFFSET,
    BATTLE_UNIT_MP_OFFSET, BATTLE_UNIT_NAME_OFFSET, BATTLE_UNIT_SKILLS_OFFSET, CURRENT_SCENE,
    SCENE_CHILD_OFFSET,
};
use crate::demon_data::{affinities, race_name, skill_names, Affinity, ELEMENT_COUNT};
use crate::game_api::{read_u16, read_u32, GameApi};
use crate::text::{decode_save_string, decode_table_string};

// Above any stat the game can show; anything larger means we are not looking at a battle unit.
const MAX_PLAUSIBLE_STAT: u32 = 9999;
// The battle scene is the current scene on the ship and its child in a dungeon.
const MAX_SCENE_DEPTH: u32 = 4;
const NAME_MAX_BYTES: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnitStatus {
    pub slot: u32,
    pub name: Option<String>,
    // None for the hero, who has no race and whose affinities come from gear.
    pub race: Option<String>,
    pub level: u16,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub affinities: Option<[Affinity; ELEMENT_COUNT]>,
    pub skills: Vec<String>,
}

impl UnitStatus {
    pub fn is_down(&self) -> bool {
        self.hp == 0
    }
}

// The page the game's bottom screen shows in battle; L and R switch between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPage {
    EnemyStatus,
    SummonList,
    PartyStatus,
    Other(u32),
}

pub fn read_enemies(game: &dyn GameApi) -> Vec<UnitStatus> {
    read_slots(game, BATTLE_FIRST_ENEMY_SLOT, BATTLE_ENEMY_SLOT_COUNT)
}

// The hero and up to three demons out in battle.
pub fn read_party(game: &dyn GameApi) -> Vec<UnitStatus> {
    read_slots(game, 0, BATTLE_PARTY_SLOT_COUNT)
}

pub fn read_hero(game: &dyn GameApi) -> Option<UnitStatus> {
    read_party(game).into_iter().find(|unit| unit.slot == 0)
}

fn read_slots(game: &dyn GameApi, first: u32, count: u32) -> Vec<UnitStatus> {
    let Some(table) =
        find_battle_scene(game).map(|scene| scene.wrapping_add(BATTLE_SCENE_UNIT_TABLE_OFFSET))
    else {
        return Vec::new();
    };
    (0..count)
        .filter_map(|index| {
            let unit = read_u32(game, Arm9Addr(table + 4 * (first + index)))?;
            read_unit(game, unit, index)
        })
        .collect()
}

fn battle_ui(game: &dyn GameApi) -> Option<u32> {
    let scene = find_battle_scene(game)?;
    read_u32(game, Arm9Addr(scene.wrapping_add(SCENE_CHILD_OFFSET)))
}

// Index of the highlighted entry in the bottom page's list (the Summon list's red frame).
pub fn list_cursor(game: &dyn GameApi) -> Option<usize> {
    let ui = battle_ui(game)?;
    let offset = Arm9Addr(ui.wrapping_add(BATTLE_UI_LIST_CURSOR_OFFSET)).main_ram_offset(1)?;
    game.main_ram()
        .get(offset)
        .map(|&cursor| usize::from(cursor))
}

pub fn bottom_page(game: &dyn GameApi) -> Option<BottomPage> {
    let ui = battle_ui(game)?;
    let page = read_u32(game, Arm9Addr(ui.wrapping_add(BATTLE_UI_PAGE_OFFSET)))?;
    Some(match page {
        BATTLE_PAGE_ENEMY_STATUS => BottomPage::EnemyStatus,
        BATTLE_PAGE_SUMMON_LIST => BottomPage::SummonList,
        BATTLE_PAGE_PARTY_STATUS => BottomPage::PartyStatus,
        other => BottomPage::Other(other),
    })
}

fn find_battle_scene(game: &dyn GameApi) -> Option<u32> {
    let mut scene = read_u32(game, CURRENT_SCENE)?;
    for _ in 0..MAX_SCENE_DEPTH {
        let table = scene.wrapping_add(BATTLE_SCENE_UNIT_TABLE_OFFSET);
        let hero_unit = read_u32(game, Arm9Addr(table))?;
        if hero_unit == table.wrapping_add(BATTLE_TABLE_HERO_UNIT_OFFSET) {
            return Some(scene);
        }
        scene = read_u32(game, Arm9Addr(scene.wrapping_add(SCENE_CHILD_OFFSET)))?;
    }
    None
}

fn read_unit(game: &dyn GameApi, unit: u32, slot: u32) -> Option<UnitStatus> {
    let field = |offset: u32| read_u32(game, Arm9Addr(unit.wrapping_add(offset)));
    let demon_id = read_u16(
        game,
        Arm9Addr(unit.wrapping_add(BATTLE_UNIT_DEMON_ID_OFFSET)),
    )
    .filter(|&id| id != 0);
    let status = UnitStatus {
        slot,
        name: field(BATTLE_UNIT_NAME_OFFSET).and_then(|pointer| read_name(game, pointer)),
        race: demon_id.and_then(|id| race_name(game, id)),
        level: read_u16(game, Arm9Addr(unit.wrapping_add(BATTLE_UNIT_LEVEL_OFFSET)))?,
        hp: field(BATTLE_UNIT_HP_OFFSET)?,
        max_hp: field(BATTLE_UNIT_MAX_HP_OFFSET)?,
        mp: field(BATTLE_UNIT_MP_OFFSET)?,
        max_mp: field(BATTLE_UNIT_MAX_MP_OFFSET)?,
        affinities: demon_id.and_then(|id| affinities(game, id)),
        skills: skill_names(game, Arm9Addr(unit.wrapping_add(BATTLE_UNIT_SKILLS_OFFSET))),
    };
    let plausible = status.max_hp > 0
        && status.max_hp <= MAX_PLAUSIBLE_STAT
        && status.hp <= status.max_hp
        && status.mp <= status.max_mp
        && status.max_mp <= MAX_PLAUSIBLE_STAT;
    plausible.then_some(status)
}

// Demon records start with a table string (0xFF 0xFF mark); the hero's starts with 16-bit text.
fn read_name(game: &dyn GameApi, pointer: u32) -> Option<String> {
    let offset = Arm9Addr(pointer).main_ram_offset(NAME_MAX_BYTES as u32)?;
    let bytes = game.main_ram().get(offset..offset + NAME_MAX_BYTES)?;
    if bytes.starts_with(&[0xFF, 0xFF]) {
        decode_table_string(bytes)
    } else {
        decode_save_string(bytes)
    }
}

// Like the game, repeated demons get a letter ("Slime A", "Slime B"); unique ones keep their name.
pub fn display_names(enemies: &[UnitStatus]) -> Vec<String> {
    let name_of = |enemy: &UnitStatus| enemy.name.clone().unwrap_or_else(|| "???".to_string());
    enemies
        .iter()
        .map(|enemy| {
            let name = name_of(enemy);
            let same: Vec<&UnitStatus> = enemies
                .iter()
                .filter(|other| name_of(other) == name)
                .collect();
            if same.len() < 2 {
                return name;
            }
            let position = same
                .iter()
                .position(|other| other.slot == enemy.slot)
                .unwrap_or(0);
            format!("{name} {}", char::from(b'A' + (position % 26) as u8))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::MAIN_RAM_BASE;
    use crate::game_api::FakeGame;

    const SCENE: u32 = 0x0222_BDA0;

    fn put_u32(game: &mut FakeGame, addr: u32, value: u32) {
        let offset = (addr - MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put_battle_scene(game: &mut FakeGame, scene: u32) {
        let table = scene + BATTLE_SCENE_UNIT_TABLE_OFFSET;
        put_u32(game, table, table + BATTLE_TABLE_HERO_UNIT_OFFSET);
    }

    fn put_unit(game: &mut FakeGame, slot: u32, unit: u32, stats: [u32; 4]) {
        let table = SCENE + BATTLE_SCENE_UNIT_TABLE_OFFSET;
        put_u32(game, table + 4 * (BATTLE_FIRST_ENEMY_SLOT + slot), unit);
        put_u32(game, unit + BATTLE_UNIT_HP_OFFSET, stats[0]);
        put_u32(game, unit + BATTLE_UNIT_MAX_HP_OFFSET, stats[1]);
        put_u32(game, unit + BATTLE_UNIT_MP_OFFSET, stats[2]);
        put_u32(game, unit + BATTLE_UNIT_MAX_MP_OFFSET, stats[3]);
    }

    #[test]
    fn reads_the_live_enemies_of_the_first_battle() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_battle_scene(&mut game, SCENE);
        put_unit(&mut game, 0, 0x0222_F21C, [0, 0, 0, 0]);
        put_unit(&mut game, 4, 0x0222_E3BC, [29, 29, 13, 13]);
        put_unit(&mut game, 5, 0x0222_E69C, [0, 29, 13, 13]);
        let enemies = read_enemies(&game);
        assert_eq!(
            enemies,
            [
                UnitStatus {
                    slot: 4,
                    name: None,
                    level: 0,
                    hp: 29,
                    max_hp: 29,
                    mp: 13,
                    max_mp: 13,
                    ..UnitStatus::default()
                },
                UnitStatus {
                    slot: 5,
                    name: None,
                    level: 0,
                    hp: 0,
                    max_hp: 29,
                    mp: 13,
                    max_mp: 13,
                    ..UnitStatus::default()
                }
            ]
        );
        assert!(enemies[1].is_down());
    }

    #[test]
    fn garbage_outside_battle_reads_as_no_enemies() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, 0x314C_4254);
        assert_eq!(read_enemies(&game), []);
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_unit(&mut game, 0, 0x0222_E3BC, [29, 29, 13, 13]);
        assert_eq!(
            read_enemies(&game),
            [],
            "no hero marker, not a battle scene"
        );
        put_battle_scene(&mut game, SCENE);
        put_unit(&mut game, 0, 0x0222_E3BC, [50_000, 60_000, 0, 0]);
        assert_eq!(read_enemies(&game), []);
    }

    #[test]
    fn the_party_is_read_from_the_first_slots_with_the_heros_save_name() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_battle_scene(&mut game, SCENE);
        let table = SCENE + BATTLE_SCENE_UNIT_TABLE_OFFSET;
        let hero = table + BATTLE_TABLE_HERO_UNIT_OFFSET;
        put_u32(&mut game, hero + BATTLE_UNIT_HP_OFFSET, 37);
        put_u32(&mut game, hero + BATTLE_UNIT_MAX_HP_OFFSET, 64);
        put_u32(&mut game, hero + BATTLE_UNIT_MAX_MP_OFFSET, 42);
        put_u32(&mut game, hero + BATTLE_UNIT_LEVEL_OFFSET, 2);
        let record = 0x0221_4324;
        put_u32(&mut game, hero + BATTLE_UNIT_NAME_OFFSET, record);
        put_u32(&mut game, record, 0x0023_0023);
        put_u32(&mut game, record + 4, 0x0000_0023);
        let party = read_party(&game);
        assert_eq!(party.len(), 1);
        assert_eq!(
            (party[0].name.as_deref(), party[0].level, party[0].hp),
            (Some("BBB"), 2, 37)
        );
    }

    #[test]
    fn bottom_page_follows_the_games_panel_value() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_battle_scene(&mut game, SCENE);
        let ui = 0x022B_5CC0;
        put_u32(&mut game, SCENE + SCENE_CHILD_OFFSET, ui);
        let page_at = ui + BATTLE_UI_PAGE_OFFSET;
        put_u32(&mut game, page_at, 1);
        assert_eq!(bottom_page(&game), Some(BottomPage::EnemyStatus));
        put_u32(&mut game, page_at, 2);
        assert_eq!(bottom_page(&game), Some(BottomPage::PartyStatus));
        put_u32(&mut game, page_at, 3);
        assert_eq!(bottom_page(&game), Some(BottomPage::SummonList));
        put_u32(&mut game, page_at, 5);
        assert_eq!(
            bottom_page(&game),
            Some(BottomPage::Other(5)),
            "battle results"
        );
    }

    #[test]
    fn dungeon_battles_live_in_the_child_scene() {
        let mut game = FakeGame::default();
        let dungeon_scene = 0x0222_8960;
        put_u32(&mut game, CURRENT_SCENE.0, dungeon_scene);
        put_u32(&mut game, dungeon_scene + SCENE_CHILD_OFFSET, SCENE);
        put_battle_scene(&mut game, SCENE);
        put_unit(&mut game, 4, 0x0222_BA9C, [36, 36, 16, 16]);
        assert_eq!(read_enemies(&game).len(), 1);
    }

    fn enemy(slot: u32, name: &str) -> UnitStatus {
        UnitStatus {
            slot,
            name: Some(name.to_string()),
            level: 1,
            hp: 1,
            max_hp: 1,
            ..UnitStatus::default()
        }
    }

    #[test]
    fn repeated_demons_get_letters_and_unique_ones_do_not() {
        let enemies = [enemy(4, "Slime"), enemy(5, "Pixie"), enemy(6, "Slime")];
        assert_eq!(display_names(&enemies), ["Slime A", "Pixie", "Slime B"]);
    }

    #[test]
    fn reads_the_name_through_the_unit_pointer() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_battle_scene(&mut game, SCENE);
        put_unit(&mut game, 4, 0x0222_BA9C, [36, 36, 16, 16]);
        let name_at = 0x021E_7EA0;
        put_u32(&mut game, 0x0222_BA9C + BATTLE_UNIT_NAME_OFFSET, name_at);
        let offset = (name_at - MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + 9]
            .copy_from_slice(&[0xFF, 0xFF, 0x31, 0x4A, 0x59, 0x4A, 0x46, 0x00, 0xFE]);
        assert_eq!(read_enemies(&game)[0].name.as_deref(), Some("Pixie"));
    }
}
