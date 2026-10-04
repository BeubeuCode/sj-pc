use crate::addr::Arm9Addr;
use crate::addresses::{
    DISPLAY_MODE_ALTERNATE, DISPLAY_MODE_TOP_ONLY, RENDER_DISPLAY_MODE_OFFSET,
    RENDER_MAIN_LOOP_COUNTER_OFFSET, RENDER_STATE,
};
use crate::game_api::{read_u32, GameApi};

pub fn main_loop_counter(game: &dyn GameApi) -> Option<u32> {
    read_u32(
        game,
        Arm9Addr(RENDER_STATE.0 + RENDER_MAIN_LOOP_COUNTER_OFFSET),
    )
}

// How much of each game frame goes to the top screen: all of it in battles and dungeons, or while
// our top-screen patch runs; half when the 3D alternates between the screens.
pub fn top_screen_share(game: &dyn GameApi, top_screen_patch: bool) -> f32 {
    match read_u32(game, Arm9Addr(RENDER_STATE.0 + RENDER_DISPLAY_MODE_OFFSET)) {
        Some(DISPLAY_MODE_TOP_ONLY) => 1.0,
        Some(DISPLAY_MODE_ALTERNATE) if top_screen_patch => 1.0,
        Some(DISPLAY_MODE_ALTERNATE) => 0.5,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::MAIN_RAM_BASE;
    use crate::game_api::FakeGame;

    fn with_display_mode(mode: u32) -> FakeGame {
        let mut game = FakeGame::default();
        let offset = (RENDER_STATE.0 + RENDER_DISPLAY_MODE_OFFSET - MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + 4].copy_from_slice(&mode.to_le_bytes());
        game
    }

    #[test]
    fn the_top_screen_gets_every_frame_unless_the_3d_alternates() {
        assert_eq!(top_screen_share(&with_display_mode(1), false), 1.0);
        assert_eq!(top_screen_share(&with_display_mode(0), false), 0.5);
        assert_eq!(top_screen_share(&with_display_mode(0), true), 1.0);
        assert_eq!(top_screen_share(&with_display_mode(2), true), 0.0);
    }
}
