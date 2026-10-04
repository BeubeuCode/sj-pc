use serde::{Deserialize, Serialize};

use crate::rect::RectPx;

pub const NDS_ASPECT: f32 = 4.0 / 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipSettings {
    pub visible: bool,
    pub corner: Corner,
    pub height_fraction: f32,
    pub margin_px: u32,
}

impl Default for PipSettings {
    fn default() -> Self {
        Self {
            visible: true,
            corner: Corner::BottomRight,
            height_fraction: 0.35,
            margin_px: 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRects {
    pub top: RectPx,
    pub pip: Option<RectPx>,
}

pub fn screen_rects(
    window_width_px: u32,
    window_height_px: u32,
    top_aspect: f32,
    pip: &PipSettings,
) -> ScreenRects {
    let top = fit_centered(window_width_px, window_height_px, top_aspect);
    if !pip.visible {
        return ScreenRects { top, pip: None };
    }
    let pip_rect = pip_rect(window_width_px, window_height_px, pip);
    ScreenRects {
        top,
        pip: Some(pip_rect),
    }
}

fn fit_centered(window_width_px: u32, window_height_px: u32, aspect: f32) -> RectPx {
    let width_at_full_height = (window_height_px as f32 * aspect).round() as u32;
    if width_at_full_height <= window_width_px {
        let x = (window_width_px - width_at_full_height) / 2;
        return RectPx {
            x: x as i32,
            y: 0,
            width: width_at_full_height,
            height: window_height_px,
        };
    }
    let height_at_full_width = (window_width_px as f32 / aspect).round() as u32;
    let y = (window_height_px - height_at_full_width) / 2;
    RectPx {
        x: 0,
        y: y as i32,
        width: window_width_px,
        height: height_at_full_width,
    }
}

fn pip_rect(window_width_px: u32, window_height_px: u32, pip: &PipSettings) -> RectPx {
    let height = (window_height_px as f32 * pip.height_fraction).round() as u32;
    let width = (height as f32 * NDS_ASPECT).round() as u32;
    let margin = pip.margin_px as i32;
    let left = margin;
    let right = window_width_px as i32 - width as i32 - margin;
    let top = margin;
    let bottom = window_height_px as i32 - height as i32 - margin;
    let (x, y) = match pip.corner {
        Corner::TopLeft => (left, top),
        Corner::TopRight => (right, top),
        Corner::BottomLeft => (left, bottom),
        Corner::BottomRight => (right, bottom),
    };
    RectPx {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hidden_pip() -> PipSettings {
        PipSettings {
            visible: false,
            ..PipSettings::default()
        }
    }

    #[test]
    fn widescreen_window_pillarboxes_4_3_image() {
        let rects = screen_rects(1920, 1080, NDS_ASPECT, &hidden_pip());
        assert_eq!(
            rects.top,
            RectPx {
                x: 240,
                y: 0,
                width: 1440,
                height: 1080
            }
        );
    }

    #[test]
    fn tall_window_letterboxes_4_3_image() {
        let rects = screen_rects(800, 1000, NDS_ASPECT, &hidden_pip());
        assert_eq!(
            rects.top,
            RectPx {
                x: 0,
                y: 200,
                width: 800,
                height: 600
            }
        );
    }

    #[test]
    fn widescreen_aspect_fills_16_9_window() {
        let rects = screen_rects(1920, 1080, 16.0 / 9.0, &hidden_pip());
        assert_eq!(
            rects.top,
            RectPx {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080
            }
        );
    }

    #[test]
    fn hidden_pip_has_no_rect() {
        assert_eq!(
            screen_rects(1920, 1080, NDS_ASPECT, &hidden_pip()).pip,
            None
        );
    }

    #[test]
    fn bottom_right_pip_keeps_margin_and_4_3_aspect() {
        let pip = PipSettings {
            height_fraction: 0.25,
            margin_px: 10,
            ..PipSettings::default()
        };
        let rects = screen_rects(1920, 1080, NDS_ASPECT, &pip);
        assert_eq!(
            rects.pip,
            Some(RectPx {
                x: 1550,
                y: 800,
                width: 360,
                height: 270
            })
        );
    }

    #[test]
    fn top_left_pip_sits_at_margin() {
        let pip = PipSettings {
            corner: Corner::TopLeft,
            height_fraction: 0.25,
            margin_px: 10,
            ..PipSettings::default()
        };
        let rects = screen_rects(1920, 1080, NDS_ASPECT, &pip);
        assert_eq!(rects.pip.map(|r| (r.x, r.y)), Some((10, 10)));
    }
}
