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
            visible: false,
            corner: Corner::BottomRight,
            height_fraction: 0.35,
            margin_px: 16,
        }
    }
}

pub fn fit_centered(window_width_px: u32, window_height_px: u32, aspect: f32) -> RectPx {
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

pub fn corner_rect(
    window_px: (u32, u32),
    size_px: (u32, u32),
    corner: Corner,
    margin_px: u32,
) -> RectPx {
    let margin = margin_px as i32;
    let left = margin;
    let right = window_px.0 as i32 - size_px.0 as i32 - margin;
    let top = margin;
    let bottom = window_px.1 as i32 - size_px.1 as i32 - margin;
    let (x, y) = match corner {
        Corner::TopLeft => (left, top),
        Corner::TopRight => (right, top),
        Corner::BottomLeft => (left, bottom),
        Corner::BottomRight => (right, bottom),
    };
    RectPx {
        x,
        y,
        width: size_px.0,
        height: size_px.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widescreen_window_pillarboxes_4_3_image() {
        assert_eq!(
            fit_centered(1920, 1080, NDS_ASPECT),
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
        assert_eq!(
            fit_centered(800, 1000, NDS_ASPECT),
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
        assert_eq!(
            fit_centered(1920, 1080, 16.0 / 9.0),
            RectPx {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080
            }
        );
    }

    #[test]
    fn corner_rects_keep_their_margin() {
        let size = (360, 270);
        assert_eq!(
            corner_rect((1920, 1080), size, Corner::BottomRight, 10),
            RectPx {
                x: 1550,
                y: 800,
                width: 360,
                height: 270
            }
        );
        assert_eq!(
            corner_rect((1920, 1080), size, Corner::TopLeft, 10),
            RectPx {
                x: 10,
                y: 10,
                width: 360,
                height: 270
            }
        );
    }
}
