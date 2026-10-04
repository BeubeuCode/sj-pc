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
    pub minimap: bool,
    pub minimap_corner: Corner,
    pub minimap_height_fraction: f32,
    pub minimap_opacity: f32,
    pub margin_px: u32,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
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
    if pip.visible && arrangement == Arrangement::Single(Screen::Top) {
        overlays.push(pip_overlay(pip));
    }
    ScreenPlan {
        arrangement,
        overlays,
    }
}

fn arrangement_for(mode: GameMode) -> Arrangement {
    match mode {
        GameMode::Facility | GameMode::ShipScene | GameMode::Event => {
            Arrangement::Single(Screen::Top)
        }
        GameMode::Title
        | GameMode::TextEntry
        | GameMode::Menu
        | GameMode::MissionLog
        | GameMode::Unknown(_) => Arrangement::SideBySide,
    }
}

// ponytail: off everywhere for now. Ship rooms are picked from a menu, and every 0x0400 sample so far
// is a ship scene with an empty grid on the bottom screen, so the minimap only covered portraits.
// Turn it back on for the mode a dungeon snapshot shows the automap in.
fn shows_automap(_mode: GameMode) -> bool {
    false
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
    RectPx {
        x: screen.x + in_screen.x,
        y: screen.y + in_screen.y,
        ..in_screen
    }
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
            GameMode::ShipScene,
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
    fn ship_scenes_show_the_top_screen_without_a_minimap() {
        for mode in [GameMode::Facility, GameMode::ShipScene] {
            let plan = plan(mode, false, &HudSettings::default(), &no_pip());
            assert_eq!(plan.arrangement, Arrangement::Single(Screen::Top));
            assert_eq!(plan.overlays, []);
        }
    }

    #[test]
    fn minimap_sits_in_the_corner_of_the_game_image_not_the_window() {
        let overlay = minimap_overlay(&HudSettings::default());
        let rect = overlay_rect((1920, 1080), &overlay, 16);
        let image_right_px = 240 + 1440;
        assert_eq!(
            rect,
            RectPx {
                x: image_right_px - 16 - 518,
                y: 16,
                width: 518,
                height: 324
            }
        );
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
            GameMode::ShipScene,
            false,
            &HudSettings::default(),
            &no_pip(),
        );
        assert_eq!(touch_target((1920, 1080), &field, 16), None);
        let pip = PipSettings {
            visible: true,
            ..PipSettings::default()
        };
        let with_pip = plan(GameMode::ShipScene, false, &HudSettings::default(), &pip);
        assert!(touch_target((1920, 1080), &with_pip, 16).is_some());
    }
}
