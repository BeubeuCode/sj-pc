use egui::{vec2, Color32, Pos2, Rect};

use crate::pixel_font::Glyphs;

// Frame colours sampled from the game's battle panels.
pub const FRAME_OUTER: Color32 = Color32::from_rgb(0x0c, 0x14, 0x24);
pub const FRAME_BLUE: Color32 = Color32::from_rgb(0x24, 0x4c, 0x84);
pub const BLACK: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const FRAME_DS_PX: u32 = 3;
pub const CHAMFER_DS_PX: u32 = 6;

// Draws in DS pixels: one DS pixel is `pixel` points, starting at `origin`.
pub struct PixelCanvas<'a> {
    pub painter: &'a egui::Painter,
    pub origin: Pos2,
    pub pixel: f32,
    pub opacity: f32,
}

impl PixelCanvas<'_> {
    pub fn fill(&self, x: u32, y: u32, width: u32, height: u32, colour: Color32) {
        let min = self.origin + vec2(x as f32, y as f32) * self.pixel;
        let size = vec2(width as f32, height as f32) * self.pixel;
        self.painter.rect_filled(
            Rect::from_min_size(min, size),
            0.0,
            colour.gamma_multiply(self.opacity),
        );
    }

    pub fn text(&self, glyphs: &Glyphs, x: u32, y: u32, text: &str, colour: Color32) {
        let mut glyph_x = x;
        for ch in text.chars() {
            for (row, bits) in glyphs.rows(ch).unwrap_or_default().iter().enumerate() {
                for column in 0..glyphs.cell_width {
                    if glyphs.is_set(*bits, column) {
                        self.fill(glyph_x + column, y + row as u32, 1, 1, colour);
                    }
                }
            }
            glyph_x += glyphs.glyph_width(ch) + 1;
        }
    }

    pub fn outlined_text(&self, glyphs: &Glyphs, x: u32, y: u32, text: &str, colour: Color32) {
        for (dx, dy) in [
            (0, 1),
            (2, 1),
            (1, 0),
            (1, 2),
            (0, 0),
            (2, 2),
            (0, 2),
            (2, 0),
        ] {
            self.text(glyphs, x + dx - 1, y + dy - 1, text, BLACK);
        }
        self.text(glyphs, x, y, text, colour);
    }

    // The game's panel border: navy edge, blue line, black line, top-left corner cut at 45°.
    pub fn frame(&self, width: u32, height: u32) {
        for (layer, colour) in [FRAME_OUTER, FRAME_BLUE, BLACK].into_iter().enumerate() {
            let layer = layer as u32;
            let (inner_width, inner_height) = (width - 2 * layer, height - 2 * layer);
            for row in 0..inner_height {
                let cut = CHAMFER_DS_PX.saturating_sub(row);
                self.fill(layer + cut, layer + row, inner_width - cut, 1, colour);
            }
        }
    }
}
