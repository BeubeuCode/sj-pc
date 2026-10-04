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
    // The core drew 4/3 wider screens (widescreen 3D): the 2D of each screen sits in the middle 3/4.
    pub wide_frame: bool,
}

pub const WIDE_ASPECT: f32 = 16.0 / 9.0;
// The middle 3/4 of a wide frame holds the original 256 DS columns.
const WIDE_CENTRE_SPAN: (f32, f32) = (0.125, 0.875);
const FULL_SPAN: (f32, f32) = (0.0, 1.0);

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

pub fn plan(
    mode: GameMode,
    swapped: bool,
    hud: &HudSettings,
    pip: &PipSettings,
    wide_frame: bool,
) -> ScreenPlan {
    if swapped {
        return ScreenPlan {
            arrangement: Arrangement::Single(Screen::Bottom),
            overlays: Vec::new(),
            wide_frame,
        };
    }
    let arrangement = arrangement_for(mode);
    let mut overlays = Vec::new();
    if hud.minimap && shows_automap(mode) {
        overlays.push(minimap_overlay(hud));
    }
    if pip.visible && arrangement == Arrangement::Single(Screen::Top) {
        overlays.push(pip_overlay(pip));
    }
    ScreenPlan {
        arrangement,
        overlays,
        wide_frame,
    }
}

// Widescreen frames are 4/3 wider than two stacked 256x192 screens.
pub fn is_wide_frame(width_px: u32, height_px: u32) -> bool {
    width_px * 3 > height_px * 2
}

// The top screen is shown 16:9 only when it is alone; next to the bottom screen, as in menus,
// it shows its 4:3 middle like everything else.
pub fn top_is_wide(plan: &ScreenPlan) -> bool {
    plan.wide_frame && plan.arrangement == Arrangement::Single(Screen::Top)
}

// Which horizontal part of a screen's frame rows to show, as fractions of the frame width.
pub fn source_span(screen: Screen, plan: &ScreenPlan) -> (f32, f32) {
    if !plan.wide_frame || (screen == Screen::Top && top_is_wide(plan)) {
        return FULL_SPAN;
    }
    WIDE_CENTRE_SPAN
}

fn top_image(window_px: (u32, u32), plan: &ScreenPlan) -> RectPx {
    let aspect = if top_is_wide(plan) {
        WIDE_ASPECT
    } else {
        NDS_ASPECT
    };
    fit_centered(window_px.0, window_px.1, aspect)
}

fn arrangement_for(mode: GameMode) -> Arrangement {
    match mode {
        GameMode::Facility
        | GameMode::ShipScene
        | GameMode::Dungeon { .. }
        | GameMode::Battle
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

pub fn arrangement_rects(window_px: (u32, u32), plan: &ScreenPlan) -> Vec<(Screen, RectPx)> {
    match plan.arrangement {
        Arrangement::Single(screen) => vec![(screen, top_image(window_px, plan))],
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
pub fn overlay_rect(
    window_px: (u32, u32),
    plan: &ScreenPlan,
    overlay: &Overlay,
    margin_px: u32,
) -> RectPx {
    let screen = top_image(window_px, plan);
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
// Panel width, and the height of a panel with only the name, HP and MP.
pub const ENEMY_PANEL_DS_PX: (u32, u32) = (64, 34);
const ENEMY_PANEL_GAP_DS_PX: u32 = 2;

// One rect per panel, stacked under the game's header; `heights_ds_px` are the panels' heights in
// DS pixels. The scale shrinks when a tall stack would run off the window.
pub fn enemy_panel_rects(
    window_px: (u32, u32),
    plan: &ScreenPlan,
    heights_ds_px: &[u32],
    margin_px: u32,
) -> Vec<RectPx> {
    let image = top_image(window_px, plan);
    // The game draws everything in the 4:3 middle; with widescreen 3D the extra strips on each side
    // are free, so the panels go there and never cover the game's own boxes.
    let content = if top_is_wide(plan) {
        RectPx {
            x: image.x + (image.width / 8) as i32,
            width: image.width * 3 / 4,
            ..image
        }
    } else {
        image
    };
    let image_scale = content.height as f32 / NDS_SCREEN_HEIGHT_PX as f32;
    let image_right = content.x + content.width as i32;
    let side_bar_px = (window_px.0 as i32 - image_right - 2 * margin_px as i32).max(0) as u32;
    let side_scale = side_bar_px / ENEMY_PANEL_DS_PX.0;
    let readable_scale = ((image_scale * 0.5).round() as u32).max(1);
    let full_scale = (image_scale.floor() as u32).max(1);
    let top = image.y + (GAME_HEADER_DS_PX * image_scale).round() as i32;
    let stack_ds_px: u32 = heights_ds_px
        .iter()
        .map(|height| height + ENEMY_PANEL_GAP_DS_PX)
        .sum();
    let free_height_px = (window_px.1 as i32 - top - margin_px as i32).max(0) as u32;
    let fit_scale = (free_height_px / stack_ds_px.max(1)).max(1);
    let in_side_bar = side_scale >= readable_scale;
    let scale = if in_side_bar {
        side_scale.min(full_scale)
    } else {
        full_scale
    }
    .min(fit_scale);
    let width = ENEMY_PANEL_DS_PX.0 * scale;
    let x = if in_side_bar {
        image_right + margin_px as i32
    } else {
        image_right - margin_px as i32 - width as i32
    };
    let mut y = top;
    heights_ds_px
        .iter()
        .map(|height_ds_px| {
            let rect = RectPx {
                x,
                y,
                width,
                height: height_ds_px * scale,
            };
            y += ((height_ds_px + ENEMY_PANEL_GAP_DS_PX) * scale) as i32;
            rect
        })
        .collect()
}

pub fn touch_target(window_px: (u32, u32), plan: &ScreenPlan, margin_px: u32) -> Option<RectPx> {
    let in_arrangement = arrangement_rects(window_px, plan)
        .into_iter()
        .find(|(screen, _)| *screen == Screen::Bottom)
        .map(|(_, rect)| rect);
    in_arrangement.or_else(|| {
        plan.overlays
            .iter()
            .find(|overlay| overlay.accepts_touch)
            .map(|overlay| overlay_rect(window_px, plan, overlay, margin_px))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top_only(wide_frame: bool) -> ScreenPlan {
        ScreenPlan {
            arrangement: Arrangement::Single(Screen::Top),
            overlays: Vec::new(),
            wide_frame,
        }
    }

    fn side_by_side() -> ScreenPlan {
        ScreenPlan {
            arrangement: Arrangement::SideBySide,
            overlays: Vec::new(),
            wide_frame: true,
        }
    }

    #[test]
    fn widescreen_frames_are_detected_from_their_size() {
        assert!(!is_wide_frame(1024, 1536));
        assert!(is_wide_frame(1365, 1536));
    }

    #[test]
    fn a_lone_wide_top_screen_fills_a_16_9_window_and_shows_its_whole_width() {
        let plan = top_only(true);
        assert_eq!(
            arrangement_rects((1920, 1080), &plan),
            [(
                Screen::Top,
                RectPx {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080
                }
            )]
        );
        assert_eq!(source_span(Screen::Top, &plan), (0.0, 1.0));
        assert_eq!(source_span(Screen::Bottom, &plan), (0.125, 0.875));
    }

    #[test]
    fn next_to_each_other_both_screens_show_their_4_3_middle() {
        let plan = side_by_side();
        assert_eq!(source_span(Screen::Top, &plan), (0.125, 0.875));
        assert_eq!(arrangement_rects((2560, 1080), &plan)[0].1.width, 1280);
    }

    #[test]
    fn enemy_panels_sit_in_the_widescreen_strip_clear_of_the_games_4_3_middle() {
        let rects = enemy_panel_rects((1920, 1080), &top_only(true), &[34], 16);
        let middle_right_px = 240 + 1440;
        assert_eq!(rects[0].x, middle_right_px + 16);
        assert!(rects[0].x + rects[0].width as i32 <= 1920);
    }

    fn no_pip() -> PipSettings {
        PipSettings {
            visible: false,
            ..PipSettings::default()
        }
    }

    #[test]
    fn menus_show_both_screens_because_details_live_on_top() {
        let plan = plan(
            GameMode::Menu,
            false,
            &HudSettings::default(),
            &no_pip(),
            false,
        );
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
                &no_pip(),
                false,
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
            false,
        );
        assert_eq!(
            plan,
            ScreenPlan {
                arrangement: Arrangement::Single(Screen::Bottom),
                overlays: Vec::new(),
                wide_frame: false
            }
        );
    }

    #[test]
    fn side_by_side_letterboxes_two_4_3_screens_in_a_21_9_window() {
        let rects = arrangement_rects((2560, 1080), &side_by_side());
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
        let plan = plan(mode, false, &HudSettings::default(), &no_pip(), false);
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
        assert_eq!(plan(mode, false, &hud, &no_pip(), false).overlays, []);
    }

    #[test]
    fn ship_scenes_and_dialogue_show_the_top_screen_without_a_minimap() {
        for mode in [
            GameMode::Facility,
            GameMode::ShipScene,
            GameMode::Dungeon { dialogue: true },
        ] {
            let plan = plan(mode, false, &HudSettings::default(), &no_pip(), false);
            assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
            assert_eq!(plan.overlays, []);
        }
    }

    #[test]
    fn minimap_sits_in_the_image_corner_below_the_game_header() {
        let overlay = minimap_overlay(&HudSettings::default());
        let rect = overlay_rect((1920, 1080), &top_only(false), &overlay, 16);
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
        let rects = enemy_panel_rects((1920, 1080), &top_only(false), &[34, 34], 16);
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
    fn a_tall_stack_shrinks_to_stay_inside_the_window() {
        let rects = enemy_panel_rects((1920, 1080), &top_only(true), &[100; 4], 16);
        let last = rects.last().unwrap();
        assert!(last.y + last.height as i32 <= 1080 - 16);
        assert_eq!(rects[0].width, 64 * 2);
    }

    #[test]
    fn enemy_panels_overlay_the_image_of_a_4_3_window_at_full_scale() {
        let rects = enemy_panel_rects((1440, 1080), &top_only(false), &[34], 16);
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
    fn battles_show_the_top_screen() {
        let mode = GameMode::Battle;
        let plan = plan(mode, false, &HudSettings::default(), &no_pip(), false);
        assert_eq!(plan.overlays, []);
        assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
    }

    #[test]
    fn touch_goes_to_the_visible_bottom_screen_or_the_pip() {
        let menu = plan(
            GameMode::Menu,
            false,
            &HudSettings::default(),
            &no_pip(),
            false,
        );
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
            false,
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
            false,
        );
        assert!(touch_target((1920, 1080), &with_pip, 16).is_some());
    }
}
