use serde::{Deserialize, Serialize};

use crate::game_mode::GameMode;
use crate::layout::{corner_rect, fit_centered, Corner, PipSettings, NDS_ASPECT};
use crate::rect::RectPx;
use crate::{NDS_SCREEN_HEIGHT_PX, NDS_SCREEN_WIDTH_PX};

pub const FULL_SCREEN: RectPx = RectPx {
    x: 0,
    y: 0,
    width: NDS_SCREEN_WIDTH_PX,
    height: NDS_SCREEN_HEIGHT_PX,
};
// The automap fills the bottom screen between a 16 px title bar and a 16 px button-hint bar.
pub const AUTOMAP_CROP: RectPx = RectPx {
    x: 0,
    y: 16,
    width: NDS_SCREEN_WIDTH_PX,
    height: 160,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrangement {
    Single(Screen),
    SideBySide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overlay {
    pub source: Screen,
    pub crop: RectPx,
    pub corner: Corner,
    pub height_fraction: f32,
    pub opacity: f32,
    pub accepts_touch: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenPlan {
    pub arrangement: Arrangement,
    pub overlays: Vec<Overlay>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HudSettings {
    pub enemy_panel: bool,
    pub minimap: bool,
    pub minimap_corner: Corner,
    pub minimap_height_fraction: f32,
    pub minimap_opacity: f32,
    pub margin_px: u32,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            enemy_panel: true,
            minimap: true,
            minimap_corner: Corner::TopRight,
            minimap_height_fraction: 0.3,
            minimap_opacity: 0.85,
            margin_px: 16,
        }
    }
}

pub fn plan(mode: GameMode, swapped: bool, hud: &HudSettings, pip: &PipSettings) -> ScreenPlan {
    if swapped {
        return ScreenPlan {
            arrangement: Arrangement::Single(Screen::Bottom),
            overlays: Vec::new(),
        };
    }
    let arrangement = arrangement_for(mode);
    let mut overlays = Vec::new();
    if hud.minimap && shows_automap(mode) {
        overlays.push(minimap_overlay(hud));
    }
    let bottom_menu = mode == (GameMode::Battle { bottom_menu: true });
    if bottom_menu {
        overlays.push(BOTTOM_MENU_OVERLAY);
    }
    if pip.visible && !bottom_menu && arrangement == Arrangement::Single(Screen::Top) {
        overlays.push(pip_overlay(pip));
    }
    ScreenPlan {
        arrangement,
        overlays,
    }
}

fn arrangement_for(mode: GameMode) -> Arrangement {
    match mode {
        GameMode::Facility
        | GameMode::ShipScene
        | GameMode::Dungeon { .. }
        | GameMode::Battle { .. }
        | GameMode::Event => Arrangement::Single(Screen::Top),
        GameMode::Title
        | GameMode::TextEntry
        | GameMode::Menu
        | GameMode::MissionLog
        | GameMode::Unknown(_) => Arrangement::SideBySide,
    }
}

// Only dungeons have an automap worth showing, and it hides while someone talks so it never covers
// a portrait.
fn shows_automap(mode: GameMode) -> bool {
    mode == GameMode::Dungeon { dialogue: false }
}

// Lists the game draws on the bottom screen in battle (Summon) come up over the right of the fight,
// at the size of the game's own panels, and take clicks.
const BOTTOM_MENU_OVERLAY: Overlay = Overlay {
    source: Screen::Bottom,
    crop: FULL_SCREEN,
    corner: Corner::TopRight,
    height_fraction: 0.62,
    opacity: 1.0,
    accepts_touch: true,
};

fn minimap_overlay(hud: &HudSettings) -> Overlay {
    Overlay {
        source: Screen::Bottom,
        crop: AUTOMAP_CROP,
        corner: hud.minimap_corner,
        height_fraction: hud.minimap_height_fraction,
        opacity: hud.minimap_opacity,
        accepts_touch: false,
    }
}

fn pip_overlay(pip: &PipSettings) -> Overlay {
    Overlay {
        source: Screen::Bottom,
        crop: FULL_SCREEN,
        corner: pip.corner,
        height_fraction: pip.height_fraction,
        opacity: 1.0,
        accepts_touch: true,
    }
}

pub fn arrangement_rects(window_px: (u32, u32), arrangement: Arrangement) -> Vec<(Screen, RectPx)> {
    match arrangement {
        Arrangement::Single(screen) => {
            vec![(screen, fit_centered(window_px.0, window_px.1, NDS_ASPECT))]
        }
        Arrangement::SideBySide => {
            let both = fit_centered(window_px.0, window_px.1, NDS_ASPECT * 2.0);
            let half_width = both.width / 2;
            let left = RectPx {
                width: half_width,
                ..both
            };
            let right = RectPx {
                x: both.x + half_width as i32,
                width: half_width,
                ..both
            };
            vec![(Screen::Top, left), (Screen::Bottom, right)]
        }
    }
}

// The game's own status bar across the top of the top screen (moon phase, SEARCH, ANALYZE). HUD
// pieces start below it so they never hide it.
pub const GAME_HEADER_DS_PX: f32 = 24.0;

// Overlays only exist over a single screen, so they anchor to that screen's image, not the
// window: otherwise they hang off the image onto the pillarbox bars.
pub fn overlay_rect(window_px: (u32, u32), overlay: &Overlay, margin_px: u32) -> RectPx {
    let screen = fit_centered(window_px.0, window_px.1, NDS_ASPECT);
    let height = (screen.height as f32 * overlay.height_fraction).round() as u32;
    let width =
        (height as f32 * overlay.crop.width as f32 / overlay.crop.height as f32).round() as u32;
    let in_screen = corner_rect(
        (screen.width, screen.height),
        (width, height),
        overlay.corner,
        margin_px,
    );
    let header_px = match overlay.corner {
        Corner::TopLeft | Corner::TopRight => {
            (GAME_HEADER_DS_PX * screen.height as f32 / NDS_SCREEN_HEIGHT_PX as f32).round() as i32
        }
        Corner::BottomLeft | Corner::BottomRight => 0,
    };
    RectPx {
        x: screen.x + in_screen.x,
        y: screen.y + in_screen.y + header_px,
        ..in_screen
    }
}

// Enemy panels are pixel art drawn at a whole number of window pixels per DS pixel, like the game's
// own party panel. They stack down the right of the top screen below the game's header bar, inside
// the side bar when the window is wide enough to hold them at a readable scale.
pub const ENEMY_PANEL_DS_PX: (u32, u32) = (64, 34);
const ENEMY_PANEL_GAP_DS_PX: u32 = 2;

pub fn enemy_panel_rects(window_px: (u32, u32), count: usize, margin_px: u32) -> Vec<RectPx> {
    let image = fit_centered(window_px.0, window_px.1, NDS_ASPECT);
    let image_scale = image.height as f32 / NDS_SCREEN_HEIGHT_PX as f32;
    let image_right = image.x + image.width as i32;
    let side_bar_px = (window_px.0 as i32 - image_right - 2 * margin_px as i32).max(0) as u32;
    let side_scale = side_bar_px / ENEMY_PANEL_DS_PX.0;
    let readable_scale = ((image_scale * 0.5).round() as u32).max(1);
    let full_scale = (image_scale.floor() as u32).max(1);
    let (scale, x) = if side_scale >= readable_scale {
        let scale = side_scale.min(full_scale);
        (scale, image_right + margin_px as i32)
    } else {
        let width = (ENEMY_PANEL_DS_PX.0 * full_scale) as i32;
        (full_scale, image_right - margin_px as i32 - width)
    };
    let (width, height) = (ENEMY_PANEL_DS_PX.0 * scale, ENEMY_PANEL_DS_PX.1 * scale);
    let top = image.y + (GAME_HEADER_DS_PX * image_scale).round() as i32;
    let step = (height + ENEMY_PANEL_GAP_DS_PX * scale) as i32;
    (0..count as i32)
        .map(|index| RectPx {
            x,
            y: top + index * step,
            width,
            height,
        })
        .collect()
}

pub fn touch_target(window_px: (u32, u32), plan: &ScreenPlan, margin_px: u32) -> Option<RectPx> {
    let in_arrangement = arrangement_rects(window_px, plan.arrangement)
        .into_iter()
        .find(|(screen, _)| *screen == Screen::Bottom)
        .map(|(_, rect)| rect);
    in_arrangement.or_else(|| {
        plan.overlays
            .iter()
            .find(|overlay| overlay.accepts_touch)
            .map(|overlay| overlay_rect(window_px, overlay, margin_px))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_pip() -> PipSettings {
        PipSettings {
            visible: false,
            ..PipSettings::default()
        }
    }

    #[test]
    fn menus_show_both_screens_because_details_live_on_top() {
        let plan = plan(GameMode::Menu, false, &HudSettings::default(), &no_pip());
        assert_eq!(plan.arrangement, Arrangement::SideBySide);
        assert_eq!(plan.overlays, []);
    }

    #[test]
    fn unknown_modes_fall_back_to_showing_everything() {
        assert_eq!(
            plan(
                GameMode::Unknown(0x0100),
                false,
                &HudSettings::default(),
                &no_pip()
            )
            .arrangement,
            Arrangement::SideBySide
        );
    }

    #[test]
    fn swap_shows_the_bottom_screen_alone() {
        let plan = plan(
            GameMode::Dungeon { dialogue: false },
            true,
            &HudSettings::default(),
            &no_pip(),
        );
        assert_eq!(
            plan,
            ScreenPlan {
                arrangement: Arrangement::Single(Screen::Bottom),
                overlays: Vec::new()
            }
        );
    }

    #[test]
    fn side_by_side_letterboxes_two_4_3_screens_in_a_21_9_window() {
        let rects = arrangement_rects((2560, 1080), Arrangement::SideBySide);
        assert_eq!(
            rects,
            [
                (
                    Screen::Top,
                    RectPx {
                        x: 0,
                        y: 60,
                        width: 1280,
                        height: 960
                    }
                ),
                (
                    Screen::Bottom,
                    RectPx {
                        x: 1280,
                        y: 60,
                        width: 1280,
                        height: 960
                    }
                )
            ]
        );
    }

    #[test]
    fn exploring_a_dungeon_shows_the_top_screen_with_a_minimap() {
        let mode = GameMode::Dungeon { dialogue: false };
        let plan = plan(mode, false, &HudSettings::default(), &no_pip());
        assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
        assert_eq!(plan.overlays.len(), 1);
        assert_eq!(plan.overlays[0].crop, AUTOMAP_CROP);
    }

    #[test]
    fn minimap_can_be_turned_off() {
        let hud = HudSettings {
            minimap: false,
            ..HudSettings::default()
        };
        let mode = GameMode::Dungeon { dialogue: false };
        assert_eq!(plan(mode, false, &hud, &no_pip()).overlays, []);
    }

    #[test]
    fn ship_scenes_and_dialogue_show_the_top_screen_without_a_minimap() {
        for mode in [
            GameMode::Facility,
            GameMode::ShipScene,
            GameMode::Dungeon { dialogue: true },
        ] {
            let plan = plan(mode, false, &HudSettings::default(), &no_pip());
            assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
            assert_eq!(plan.overlays, []);
        }
    }

    #[test]
    fn minimap_sits_in_the_image_corner_below_the_game_header() {
        let overlay = minimap_overlay(&HudSettings::default());
        let rect = overlay_rect((1920, 1080), &overlay, 16);
        let image_right_px = 240 + 1440;
        let header_px = 135;
        assert_eq!(
            rect,
            RectPx {
                x: image_right_px - 16 - 518,
                y: 16 + header_px,
                width: 518,
                height: 324
            }
        );
    }

    #[test]
    fn enemy_panels_sit_in_the_side_bar_of_a_16_9_window() {
        let rects = enemy_panel_rects((1920, 1080), 2, 16);
        assert_eq!(
            rects,
            [
                RectPx {
                    x: 1680 + 16,
                    y: 135,
                    width: 192,
                    height: 102
                },
                RectPx {
                    x: 1696,
                    y: 135 + 102 + 6,
                    width: 192,
                    height: 102
                }
            ]
        );
    }

    #[test]
    fn enemy_panels_overlay_the_image_of_a_4_3_window_at_full_scale() {
        let rects = enemy_panel_rects((1440, 1080), 1, 16);
        assert_eq!(
            rects[0],
            RectPx {
                x: 1440 - 16 - 320,
                y: 135,
                width: 320,
                height: 170
            }
        );
    }

    #[test]
    fn summon_list_comes_up_over_the_fight_and_takes_touch() {
        let mode = GameMode::Battle { bottom_menu: true };
        let plan = plan(mode, false, &HudSettings::default(), &no_pip());
        assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
        assert_eq!(plan.overlays, [BOTTOM_MENU_OVERLAY]);
        assert!(touch_target((1920, 1080), &plan, 16).is_some());
        let pip = PipSettings {
            visible: true,
            ..PipSettings::default()
        };
        let with_pip = super::plan(mode, false, &HudSettings::default(), &pip);
        assert_eq!(
            with_pip.overlays,
            [BOTTOM_MENU_OVERLAY],
            "no second copy of the bottom screen"
        );
    }

    #[test]
    fn battles_show_the_top_screen() {
        let mode = GameMode::Battle { bottom_menu: false };
        let plan = plan(mode, false, &HudSettings::default(), &no_pip());
        assert_eq!(plan.overlays, []);
        assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
    }

    #[test]
    fn touch_goes_to_the_visible_bottom_screen_or_the_pip() {
        let menu = plan(GameMode::Menu, false, &HudSettings::default(), &no_pip());
        assert_eq!(
            touch_target((2560, 1080), &menu, 16),
            Some(RectPx {
                x: 1280,
                y: 60,
                width: 1280,
                height: 960
            })
        );
        let field = plan(
            GameMode::Dungeon { dialogue: false },
            false,
            &HudSettings::default(),
            &no_pip(),
        );
        assert_eq!(touch_target((1920, 1080), &field, 16), None);
        let pip = PipSettings {
            visible: true,
            ..PipSettings::default()
        };
        let with_pip = plan(
            GameMode::Dungeon { dialogue: false },
            false,
            &HudSettings::default(),
            &pip,
        );
        assert!(touch_target((1920, 1080), &with_pip, 16).is_some());
    }
}
