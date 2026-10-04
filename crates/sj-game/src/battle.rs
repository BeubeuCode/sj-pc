use crate::addr::Arm9Addr;
use crate::addresses::{
    BATTLE_ENEMY_SLOT_COUNT, BATTLE_FIRST_ENEMY_SLOT, BATTLE_SCENE_BOTTOM_MENU_OFFSET,
    BATTLE_SCENE_UNIT_TABLE_OFFSET, BATTLE_TABLE_HERO_UNIT_OFFSET, BATTLE_UNIT_HP_OFFSET,
    BATTLE_UNIT_MAX_HP_OFFSET, BATTLE_UNIT_MAX_MP_OFFSET, BATTLE_UNIT_MP_OFFSET,
    BATTLE_UNIT_NAME_OFFSET, CURRENT_SCENE, SCENE_CHILD_OFFSET,
};
use crate::game_api::{read_u32, GameApi};
use crate::text::decode_table_string;

// Above any stat the game can show; anything larger means we are not looking at a battle unit.
const MAX_PLAUSIBLE_STAT: u32 = 9999;
// The battle scene is the current scene on the ship and its child in a dungeon.
const MAX_SCENE_DEPTH: u32 = 4;
const NAME_MAX_BYTES: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyStatus {
    pub slot: u32,
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
}

impl EnemyStatus {
    pub fn is_down(&self) -> bool {
        self.hp == 0
    }
}

pub fn read_enemies(game: &dyn GameApi) -> Vec<EnemyStatus> {
    let Some(table) =
        find_battle_scene(game).map(|scene| scene.wrapping_add(BATTLE_SCENE_UNIT_TABLE_OFFSET))
    else {
        return Vec::new();
    };
    (0..BATTLE_ENEMY_SLOT_COUNT)
        .filter_map(|index| {
            let slot_addr = Arm9Addr(table + 4 * (BATTLE_FIRST_ENEMY_SLOT + index));
            let unit = read_u32(game, slot_addr)?;
            read_unit(game, unit, index)
        })
        .collect()
}

pub fn bottom_menu_open(game: &dyn GameApi) -> bool {
    find_battle_scene(game)
        .and_then(|scene| {
            read_u32(
                game,
                Arm9Addr(scene.wrapping_add(BATTLE_SCENE_BOTTOM_MENU_OFFSET)),
            )
        })
        .is_some_and(|flag| flag != 0)
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

fn read_unit(game: &dyn GameApi, unit: u32, slot: u32) -> Option<EnemyStatus> {
    let field = |offset: u32| read_u32(game, Arm9Addr(unit.wrapping_add(offset)));
    let status = EnemyStatus {
        slot,
        name: field(BATTLE_UNIT_NAME_OFFSET).and_then(|pointer| read_name(game, pointer)),
        hp: field(BATTLE_UNIT_HP_OFFSET)?,
        max_hp: field(BATTLE_UNIT_MAX_HP_OFFSET)?,
        mp: field(BATTLE_UNIT_MP_OFFSET)?,
        max_mp: field(BATTLE_UNIT_MAX_MP_OFFSET)?,
    };
    let plausible = status.max_hp > 0
        && status.max_hp <= MAX_PLAUSIBLE_STAT
        && status.hp <= status.max_hp
        && status.mp <= status.max_mp
        && status.max_mp <= MAX_PLAUSIBLE_STAT;
    plausible.then_some(status)
}

fn read_name(game: &dyn GameApi, pointer: u32) -> Option<String> {
    let offset = Arm9Addr(pointer).main_ram_offset(NAME_MAX_BYTES as u32)?;
    decode_table_string(game.main_ram().get(offset..offset + NAME_MAX_BYTES)?)
}

// Like the game, repeated demons get a letter ("Slime A", "Slime B"); unique ones keep their name.
pub fn display_names(enemies: &[EnemyStatus]) -> Vec<String> {
    let name_of = |enemy: &EnemyStatus| enemy.name.clone().unwrap_or_else(|| "???".to_string());
    enemies
        .iter()
        .map(|enemy| {
            let name = name_of(enemy);
            let same: Vec<&EnemyStatus> = enemies
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
                EnemyStatus {
                    slot: 4,
                    name: None,
                    hp: 29,
                    max_hp: 29,
                    mp: 13,
                    max_mp: 13
                },
                EnemyStatus {
                    slot: 5,
                    name: None,
                    hp: 0,
                    max_hp: 29,
                    mp: 13,
                    max_mp: 13
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
    fn summon_list_flag_lives_in_the_battle_scene() {
        let mut game = FakeGame::default();
        put_u32(&mut game, CURRENT_SCENE.0, SCENE);
        put_battle_scene(&mut game, SCENE);
        assert!(!bottom_menu_open(&game));
        put_u32(&mut game, SCENE + BATTLE_SCENE_BOTTOM_MENU_OFFSET, 1);
        assert!(bottom_menu_open(&game));
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

    fn enemy(slot: u32, name: &str) -> EnemyStatus {
        EnemyStatus {
            slot,
            name: Some(name.to_string()),
            hp: 1,
            max_hp: 1,
            mp: 0,
            max_mp: 0,
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
