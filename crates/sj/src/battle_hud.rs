use egui::{pos2, Color32};
use sj_game::battle::{display_names, EnemyStatus};
use sj_game::rect::RectPx;
use sj_game::screen_director::ENEMY_PANEL_DS_PX;

use crate::pixel_canvas::{PixelCanvas, BLACK, CHAMFER_DS_PX, FRAME_DS_PX};
use crate::pixel_font::{DIGITS, LABEL, NAME};

// Colours sampled from the game's party panel in battle.
const STRIPE_LEFT: Color32 = Color32::from_rgb(0x0c, 0x14, 0x2c);
const STRIPE_RIGHT: Color32 = Color32::from_rgb(0x2c, 0x3c, 0x54);
const STRIPE_DARK_LEFT: Color32 = Color32::from_rgb(0x0c, 0x14, 0x24);
const STRIPE_DARK_RIGHT: Color32 = Color32::from_rgb(0x1c, 0x24, 0x3c);
const SEPARATOR: Color32 = Color32::from_rgb(0x0c, 0x1c, 0x2c);
const STATS_FILL: Color32 = Color32::from_rgb(0x00, 0x14, 0x24);
const STATS_EDGE: Color32 = Color32::from_rgb(0x00, 0x1c, 0x2c);
const NAME_TEXT: Color32 = Color32::from_rgb(0xfc, 0xfc, 0xfc);
const NAME_SHADOW: Color32 = Color32::from_rgb(0x0c, 0x0c, 0x0c);
const HP_LABEL: Color32 = Color32::from_rgb(0xfc, 0x8c, 0x7c);
const HP_BAR: [Color32; 2] = [
    Color32::from_rgb(0xfc, 0xcc, 0x3c),
    Color32::from_rgb(0xcc, 0x8c, 0x00),
];
const MP_LABEL: Color32 = Color32::from_rgb(0x54, 0xe4, 0x74);
const MP_BAR: [Color32; 2] = [
    Color32::from_rgb(0x74, 0xdc, 0x2c),
    Color32::from_rgb(0x5c, 0xac, 0x24),
];
const BAR_EMPTY: Color32 = Color32::from_rgb(0x1c, 0x2c, 0x44);
const DOWN_OPACITY: f32 = 0.45;

// Panel geometry in DS pixels, following the party panel: the game's frame, a striped name strip,
// a separator, then HP and MP rows.
const NAME_STRIP_ROWS: (u32, u32) = (3, 15);
const SEPARATOR_ROW: u32 = 15;
const NAME_AT: (u32, u32) = (5, 5);
const HP_ROW_Y: u32 = 18;
const MP_ROW_Y: u32 = 25;
const LABEL_X: u32 = 5;
const BAR_X: u32 = 18;
const BAR_WIDTH_DS_PX: u32 = 27;
const VALUE_RIGHT_X: u32 = 59;

pub fn draw_enemy_panels(root: &egui::Ui, enemies: &[EnemyStatus], rects_px: &[RectPx]) {
    let points_per_pixel = 1.0 / root.ctx().pixels_per_point();
    let names = display_names(enemies);
    for ((enemy, name), rect_px) in enemies.iter().zip(&names).zip(rects_px) {
        let canvas = PixelCanvas {
            painter: root.painter(),
            origin: pos2(rect_px.x as f32, rect_px.y as f32) * points_per_pixel,
            pixel: rect_px.width as f32 / ENEMY_PANEL_DS_PX.0 as f32 * points_per_pixel,
            opacity: if enemy.is_down() { DOWN_OPACITY } else { 1.0 },
        };
        canvas.frame(ENEMY_PANEL_DS_PX.0, ENEMY_PANEL_DS_PX.1);
        draw_name_strip(&canvas, &fit_name(name));
        draw_stat_row(
            &canvas,
            HP_ROW_Y,
            "HP",
            HP_LABEL,
            HP_BAR,
            enemy.hp,
            enemy.max_hp,
        );
        draw_stat_row(
            &canvas,
            MP_ROW_Y,
            "MP",
            MP_LABEL,
            MP_BAR,
            enemy.mp,
            enemy.max_mp,
        );
    }
}

fn draw_name_strip(canvas: &PixelCanvas, name: &str) {
    let inner_width = ENEMY_PANEL_DS_PX.0 - 2 * FRAME_DS_PX;
    for row in NAME_STRIP_ROWS.0..NAME_STRIP_ROWS.1 {
        let (left, right) = if row % 2 == 0 {
            (STRIPE_LEFT, STRIPE_RIGHT)
        } else {
            (STRIPE_DARK_LEFT, STRIPE_DARK_RIGHT)
        };
        let cut = CHAMFER_DS_PX.saturating_sub(row);
        for column in cut..inner_width {
            let t = column as f32 / inner_width as f32;
            canvas.fill(FRAME_DS_PX + column, row, 1, 1, lerp(left, right, t));
        }
    }
    canvas.fill(FRAME_DS_PX, SEPARATOR_ROW, inner_width, 1, SEPARATOR);
    let stats_top = SEPARATOR_ROW + 1;
    let stats_height = ENEMY_PANEL_DS_PX.1 - FRAME_DS_PX - stats_top;
    canvas.fill(
        FRAME_DS_PX,
        stats_top,
        inner_width,
        stats_height,
        STATS_FILL,
    );
    canvas.fill(FRAME_DS_PX, stats_top, 1, stats_height, STATS_EDGE);
    canvas.text(&NAME, NAME_AT.0 + 1, NAME_AT.1 + 1, name, NAME_SHADOW);
    canvas.text(&NAME, NAME_AT.0, NAME_AT.1, name, NAME_TEXT);
}

fn draw_stat_row(
    canvas: &PixelCanvas,
    y: u32,
    label: &str,
    label_colour: Color32,
    bar_colours: [Color32; 2],
    value: u32,
    max: u32,
) {
    canvas.outlined_text(&LABEL, LABEL_X, y, label, label_colour);
    canvas.fill(BAR_X - 1, y, BAR_WIDTH_DS_PX + 2, 4, BLACK);
    canvas.fill(BAR_X, y + 1, BAR_WIDTH_DS_PX, 2, BAR_EMPTY);
    let filled = filled_width(value, max, BAR_WIDTH_DS_PX);
    canvas.fill(BAR_X, y + 1, filled, 1, bar_colours[0]);
    canvas.fill(BAR_X, y + 2, filled, 1, bar_colours[1]);
    let digits = value.to_string();
    let digits_x = VALUE_RIGHT_X.saturating_sub(DIGITS.text_width(&digits));
    canvas.text(&DIGITS, digits_x, y, &digits, NAME_TEXT);
}

fn fit_name(name: &str) -> String {
    let available = VALUE_RIGHT_X - NAME_AT.0;
    let mut fitted = name.to_string();
    while NAME.text_width(&fitted) > available {
        fitted.pop();
    }
    fitted
}

// Rounds up so a unit with any HP left still shows at least one pixel of bar, as the game does.
fn filled_width(value: u32, max: u32, width: u32) -> u32 {
    if max == 0 {
        return 0;
    }
    (value.min(max) * width).div_ceil(max)
}

fn lerp(from: Color32, to: Color32, t: f32) -> Color32 {
    let channel = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgb(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_never_hide_a_living_unit() {
        assert_eq!(filled_width(1, 999, 30), 1);
        assert_eq!(filled_width(0, 29, 30), 0);
        assert_eq!(filled_width(29, 29, 30), 30);
        assert_eq!(filled_width(5, 0, 30), 0);
    }

    #[test]
    fn long_names_are_cut_to_the_panel() {
        assert_eq!(fit_name("Pixie"), "Pixie");
        let cut = fit_name("Mother Harlot Mother Harlot");
        assert!(NAME.text_width(&cut) <= VALUE_RIGHT_X - NAME_AT.0);
        assert!(cut.starts_with("Mother Harlot"));
    }

    #[test]
    fn stats_rows_fit_inside_the_frame() {
        assert!(MP_ROW_Y + 4 <= ENEMY_PANEL_DS_PX.1 - FRAME_DS_PX);
        assert!(VALUE_RIGHT_X < ENEMY_PANEL_DS_PX.0 - FRAME_DS_PX);
        assert!(BAR_X + BAR_WIDTH_DS_PX + 1 < VALUE_RIGHT_X - DIGITS.text_width("999"));
    }
}
