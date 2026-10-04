use crate::addresses::SCENE_FLAGS;
use crate::game_api::{read_u16, GameApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Title,
    Facility,
    TextEntry,
    ShipScene,
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
        0x0400 => GameMode::ShipScene,
        0x0800 => GameMode::Menu,
        0x2000 => GameMode::Event,
        0x4000 => GameMode::MissionLog,
        other => GameMode::Unknown(other),
    }
}

pub fn read(game: &dyn GameApi) -> GameMode {
    read_u16(game, SCENE_FLAGS).map_or(GameMode::Unknown(0xFFFF), decode)
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

    fn put_u32(game: &mut FakeGame, addr: crate::addr::Arm9Addr, value: u32) {
        let offset = (addr.0 - crate::addr::MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn reads_the_flags_from_main_ram() {
        let mut game = FakeGame::default();
        put_u32(&mut game, SCENE_FLAGS, 0x0400);
        assert_eq!(read(&game), GameMode::ShipScene);
    }
}
