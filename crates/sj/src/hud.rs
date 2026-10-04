use egui::{pos2, Color32, Rect, TextureId};
use sj_game::rect::RectPx;
use sj_game::screen_director::{overlay_rect, source_span, Screen, ScreenPlan};
use sj_game::{NDS_SCREEN_HEIGHT_PX, NDS_SCREEN_WIDTH_PX};

use crate::pixel_canvas::{PixelCanvas, FRAME_DS_PX};
use crate::present::{FrameInfo, RowOrder};

pub fn draw_overlays(
    root: &mut egui::Ui,
    plan: &ScreenPlan,
    frame: Option<FrameInfo>,
    texture: TextureId,
    drawable_px: (u32, u32),
    margin_px: u32,
) {
    let Some(frame) = frame else {
        return;
    };
    let points_per_pixel = 1.0 / root.ctx().pixels_per_point();
    for overlay in &plan.overlays {
        let rect_px = overlay_rect(drawable_px, plan, overlay, margin_px);
        draw_frame_around(root, rect_px, overlay.crop, points_per_pixel);
        let alpha = (overlay.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        root.painter().image(
            texture,
            to_points(rect_px, points_per_pixel),
            crop_uv(
                frame,
                source_span(overlay.source, plan),
                overlay.source,
                overlay.crop,
            ),
            Color32::from_white_alpha(alpha),
        );
    }
}

// The frame sits just outside the crop, at the crop's own DS pixel scale.
fn draw_frame_around(root: &egui::Ui, rect_px: RectPx, crop: RectPx, points_per_pixel: f32) {
    let pixel = rect_px.height as f32 / crop.height as f32 * points_per_pixel;
    let image_min = pos2(rect_px.x as f32, rect_px.y as f32) * points_per_pixel;
    let canvas = PixelCanvas {
        painter: root.painter(),
        origin: image_min - egui::vec2(1.0, 1.0) * FRAME_DS_PX as f32 * pixel,
        pixel,
        opacity: 1.0,
    };
    canvas.frame(crop.width + 2 * FRAME_DS_PX, crop.height + 2 * FRAME_DS_PX);
}

fn to_points(rect: RectPx, points_per_pixel: f32) -> Rect {
    let min = pos2(rect.x as f32, rect.y as f32) * points_per_pixel;
    let max = pos2(
        (rect.x + rect.width as i32) as f32,
        (rect.y + rect.height as i32) as f32,
    ) * points_per_pixel;
    Rect::from_min_max(min, max)
}

// UVs into the presenter's colour texture for a crop given in DS pixels of one screen. The frame
// sits at the texture's origin; bottom-up frames get flipped V coordinates. `span` is the part of
// the frame width holding the screen's 256 columns (its middle 3/4 in widescreen).
fn crop_uv(frame: FrameInfo, span: (f32, f32), screen: Screen, crop: RectPx) -> Rect {
    let span_left_px = span.0 * frame.width_px as f32;
    let scale_x = (span.1 - span.0) * frame.width_px as f32 / NDS_SCREEN_WIDTH_PX as f32;
    let screen_height_px = frame.height_px as f32 / 2.0;
    let scale_y = screen_height_px / NDS_SCREEN_HEIGHT_PX as f32;
    let screen_top_px = match screen {
        Screen::Top => 0.0,
        Screen::Bottom => screen_height_px,
    };
    let left = span_left_px + crop.x as f32 * scale_x;
    let right = span_left_px + (crop.x as f32 + crop.width as f32) * scale_x;
    let top = screen_top_px + crop.y as f32 * scale_y;
    let bottom = screen_top_px + (crop.y as f32 + crop.height as f32) * scale_y;
    let (top_row, bottom_row) = match frame.row_order {
        RowOrder::TopDown => (top, bottom),
        RowOrder::BottomUp => (
            frame.height_px as f32 - top,
            frame.height_px as f32 - bottom,
        ),
    };
    let texture_width = frame.texture_width_px as f32;
    let texture_height = frame.texture_height_px as f32;
    Rect::from_min_max(
        pos2(left / texture_width, top_row / texture_height),
        pos2(right / texture_width, bottom_row / texture_height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sj_game::screen_director::{AUTOMAP_CROP, FULL_SCREEN};

    fn frame(row_order: RowOrder) -> FrameInfo {
        FrameInfo {
            width_px: 512,
            height_px: 768,
            texture_width_px: 1024,
            texture_height_px: 1024,
            row_order,
        }
    }

    #[test]
    fn whole_top_screen_of_a_top_down_frame() {
        let uv = crop_uv(
            frame(RowOrder::TopDown),
            (0.0, 1.0),
            Screen::Top,
            FULL_SCREEN,
        );
        assert_eq!(uv, Rect::from_min_max(pos2(0.0, 0.0), pos2(0.5, 0.375)));
    }

    #[test]
    fn automap_crop_of_a_bottom_up_frame_is_flipped() {
        let uv = crop_uv(
            frame(RowOrder::BottomUp),
            (0.0, 1.0),
            Screen::Bottom,
            AUTOMAP_CROP,
        );
        let top_row = 768.0 - (384.0 + 32.0);
        let bottom_row = 768.0 - (384.0 + 352.0);
        assert_eq!(
            uv,
            Rect::from_min_max(pos2(0.0, top_row / 1024.0), pos2(0.5, bottom_row / 1024.0))
        );
    }

    #[test]
    fn widescreen_crops_come_from_the_middle_of_the_frame() {
        let uv = crop_uv(
            frame(RowOrder::TopDown),
            (0.125, 0.875),
            Screen::Bottom,
            FULL_SCREEN,
        );
        assert_eq!(uv.min.x, 64.0 / 1024.0);
        assert_eq!(uv.max.x, 448.0 / 1024.0);
    }
}
