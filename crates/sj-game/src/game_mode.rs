use crate::addr::Arm9Addr;
use crate::addresses::{
    CURRENT_SCENE, DUNGEON_SCENE_UPDATE_FN, MESSAGE_WINDOW, SCENE_FLAGS, SCENE_UPDATE_FN_OFFSET,
};
use crate::game_api::{read_u16, read_u32, GameApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Title,
    Facility,
    TextEntry,
    ShipScene,
    Dungeon { dialogue: bool },
    Battle,
    Menu,
    Event,
    MissionLog,
    Unknown(u16),
}

pub fn decode(scene_flags: u16) -> GameMode {
    match scene_flags {
        0x0000 => GameMode::Title,
        0x0020 => GameMode::Facility,
        0x0080 => GameMode::TextEntry,
        0x0200 => GameMode::Battle,
        0x0400 => GameMode::ShipScene,
        0x0800 => GameMode::Menu,
        0x2000 => GameMode::Event,
        0x4000 => GameMode::MissionLog,
        other => GameMode::Unknown(other),
    }
}

pub fn read(game: &dyn GameApi) -> GameMode {
    let mode = read_u16(game, SCENE_FLAGS).map_or(GameMode::Unknown(0xFFFF), decode);
    if mode != GameMode::Title || !in_dungeon(game) {
        return mode;
    }
    let dialogue = read_u32(game, MESSAGE_WINDOW).is_some_and(|window| window != 0);
    GameMode::Dungeon { dialogue }
}

fn in_dungeon(game: &dyn GameApi) -> bool {
    read_u32(game, CURRENT_SCENE)
        .and_then(|scene| read_u32(game, Arm9Addr(scene.wrapping_add(SCENE_UPDATE_FN_OFFSET))))
        == Some(DUNGEON_SCENE_UPDATE_FN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_api::FakeGame;

    #[test]
    fn each_known_scene_flag_decodes() {
        assert_eq!(decode(0x0000), GameMode::Title);
        assert_eq!(decode(0x0080), GameMode::TextEntry);
        assert_eq!(decode(0x0800), GameMode::Menu);
        assert_eq!(decode(0x4000), GameMode::MissionLog);
    }

    #[test]
    fn unseen_flags_stay_unknown_with_their_value() {
        assert_eq!(decode(0x0100), GameMode::Unknown(0x0100));
    }

    fn put_u32(game: &mut FakeGame, addr: Arm9Addr, value: u32) {
        let offset = (addr.0 - crate::addr::MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn reads_the_flags_from_main_ram() {
        let mut game = FakeGame::default();
        put_u32(&mut game, SCENE_FLAGS, 0x0400);
        assert_eq!(read(&game), GameMode::ShipScene);
    }

    const DUNGEON_SCENE: u32 = 0x0222_8960;

    fn put_dungeon_scene(game: &mut FakeGame) {
        put_u32(game, CURRENT_SCENE, DUNGEON_SCENE);
        put_u32(
            game,
            Arm9Addr(DUNGEON_SCENE + SCENE_UPDATE_FN_OFFSET),
            DUNGEON_SCENE_UPDATE_FN,
        );
    }

    #[test]
    fn dungeon_shares_the_title_flags_but_runs_the_dungeon_scene() {
        let mut game = FakeGame::default();
        assert_eq!(read(&game), GameMode::Title);
        put_dungeon_scene(&mut game);
        assert_eq!(read(&game), GameMode::Dungeon { dialogue: false });
        put_u32(&mut game, MESSAGE_WINDOW, 0x0229_DF84);
        assert_eq!(read(&game), GameMode::Dungeon { dialogue: true });
    }

    #[test]
    fn battles_in_a_dungeon_stay_battles() {
        let mut game = FakeGame::default();
        put_u32(&mut game, SCENE_FLAGS, 0x0200);
        put_dungeon_scene(&mut game);
        assert_eq!(read(&game), GameMode::Battle);
    }
}
