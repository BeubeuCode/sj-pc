use egui::{pos2, Color32};
use sj_game::battle::{display_names, UnitStatus};
use sj_game::demon_data::{Affinity, ELEMENT_NAMES};
use sj_game::rect::RectPx;
use sj_game::screen_director::ENEMY_PANEL_DS_PX;

use crate::pixel_canvas::{PixelCanvas, BLACK, CHAMFER_DS_PX, FRAME_BLUE, FRAME_DS_PX};
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
// The red line the game draws round the selected card.
const SELECTED_LINE: Color32 = Color32::from_rgb(0xf0, 0x30, 0x40);
const LEVEL_LABEL: Color32 = Color32::from_rgb(0x5c, 0x84, 0xfc);
const MP_LABEL: Color32 = Color32::from_rgb(0x54, 0xe4, 0x74);
const MP_BAR: [Color32; 2] = [
    Color32::from_rgb(0x74, 0xdc, 0x2c),
    Color32::from_rgb(0x5c, 0xac, 0x24),
];
const BAR_EMPTY: Color32 = Color32::from_rgb(0x1c, 0x2c, 0x44);
const RACE_TEXT: Color32 = Color32::from_rgb(0xbc, 0xcc, 0xec);
const SKILL_TEXT: Color32 = Color32::from_rgb(0xdc, 0xe4, 0xf4);
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
// Below MP: race, one row per affinity kind ("Wk Fire Expel"), a separator, then the skills.
const DETAILS_TOP_Y: u32 = 32;
const TEXT_LINE_DS_PX: u32 = 8;
const SEPARATOR_LINE_DS_PX: u32 = 3;
const DETAILS_BOTTOM_PAD_DS_PX: u32 = 1;
const ELEMENTS_GAP_DS_PX: u32 = 3;

enum DetailLine {
    Text {
        x: u32,
        text: String,
        colour: Color32,
    },
    Affinity {
        label: Option<(&'static str, Color32)>,
        elements: String,
    },
    Separator,
}

impl DetailLine {
    fn height(&self) -> u32 {
        match self {
            DetailLine::Separator => SEPARATOR_LINE_DS_PX,
            _ => TEXT_LINE_DS_PX,
        }
    }
}

pub fn panel_height_ds_px(unit: &UnitStatus) -> u32 {
    let lines = detail_lines(unit);
    if lines.is_empty() {
        return ENEMY_PANEL_DS_PX.1;
    }
    let details: u32 = lines.iter().map(DetailLine::height).sum();
    DETAILS_TOP_Y + details + DETAILS_BOTTOM_PAD_DS_PX + FRAME_DS_PX
}

// One panel per unit, in the style of the game's party panel: enemies, or the party when the game's
// bottom screen is on its party page.
pub fn draw_unit_panels(
    root: &egui::Ui,
    units: &[UnitStatus],
    rects_px: &[RectPx],
    selected: Option<usize>,
) {
    let points_per_pixel = 1.0 / root.ctx().pixels_per_point();
    let names = display_names(units);
    for (index, ((unit, name), rect_px)) in units.iter().zip(&names).zip(rects_px).enumerate() {
        let canvas = PixelCanvas {
            painter: root.painter(),
            origin: pos2(rect_px.x as f32, rect_px.y as f32) * points_per_pixel,
            pixel: rect_px.width as f32 / ENEMY_PANEL_DS_PX.0 as f32 * points_per_pixel,
            opacity: if unit.is_down() { DOWN_OPACITY } else { 1.0 },
        };
        let line = if selected == Some(index) {
            SELECTED_LINE
        } else {
            FRAME_BLUE
        };
        let height = panel_height_ds_px(unit);
        canvas.frame_with(ENEMY_PANEL_DS_PX.0, height, line);
        draw_name_strip(&canvas, name, &level_text(unit), height);
        draw_stat_row(
            &canvas,
            HP_ROW_Y,
            "HP",
            HP_LABEL,
            HP_BAR,
            stat(unit, unit.hp, unit.max_hp),
        );
        draw_stat_row(
            &canvas,
            MP_ROW_Y,
            "MP",
            MP_LABEL,
            MP_BAR,
            stat(unit, unit.mp, unit.max_mp),
        );
        draw_details(&canvas, &detail_lines(unit));
    }
}

// Like the game's card, an unknown demon shows "??" and "???" instead of its numbers.
fn level_text(unit: &UnitStatus) -> String {
    if unit.unknown {
        return "??".to_string();
    }
    unit.level.to_string()
}

fn stat(unit: &UnitStatus, value: u32, max: u32) -> Option<(u32, u32)> {
    (!unit.unknown).then_some((value, max))
}

fn detail_lines(unit: &UnitStatus) -> Vec<DetailLine> {
    let mut lines = Vec::new();
    if let Some(race) = &unit.race {
        lines.push(DetailLine::Text {
            x: LABEL_X,
            text: race.clone(),
            colour: RACE_TEXT,
        });
    }
    if let Some(affinities) = &unit.affinities {
        for (kind, label) in Affinity::LABELS {
            let elements: Vec<&str> = ELEMENT_NAMES
                .iter()
                .zip(affinities)
                .filter(|(_, affinity)| **affinity == kind)
                .map(|(element, _)| *element)
                .collect();
            lines.extend(affinity_lines(label, affinity_colour(kind), &elements));
        }
    }
    if !unit.skills.is_empty() {
        if !lines.is_empty() {
            lines.push(DetailLine::Separator);
        }
        lines.extend(unit.skills.iter().map(|skill| DetailLine::Text {
            x: LABEL_X,
            text: fit_name(skill, VALUE_RIGHT_X - LABEL_X),
            colour: SKILL_TEXT,
        }));
    }
    lines
}

// The label, then as many elements as fit; the rest wrap to rows under the first element.
fn affinity_lines(label: &'static str, colour: Color32, elements: &[&str]) -> Vec<DetailLine> {
    let available = VALUE_RIGHT_X - elements_x();
    let mut rows: Vec<String> = Vec::new();
    for element in elements {
        match rows.last_mut() {
            Some(row) if NAME.text_width(&format!("{row} {element}")) <= available => {
                row.push(' ');
                row.push_str(element);
            }
            _ => rows.push((*element).to_string()),
        }
    }
    rows.into_iter()
        .enumerate()
        .map(|(index, elements)| DetailLine::Affinity {
            label: (index == 0).then_some((label, colour)),
            elements,
        })
        .collect()
}

fn elements_x() -> u32 {
    LABEL_X + NAME.text_width("Wk") + ELEMENTS_GAP_DS_PX
}

fn affinity_colour(kind: Affinity) -> Color32 {
    match kind {
        Affinity::Weak => HP_LABEL,
        Affinity::Strong | Affinity::Normal => LEVEL_LABEL,
        Affinity::Null | Affinity::Repel | Affinity::Drain => MP_LABEL,
    }
}

fn draw_details(canvas: &PixelCanvas, lines: &[DetailLine]) {
    let mut y = DETAILS_TOP_Y;
    for line in lines {
        match line {
            DetailLine::Text { x, text, colour } => canvas.text(&NAME, *x, y, text, *colour),
            DetailLine::Affinity { label, elements } => {
                if let Some((label, colour)) = label {
                    canvas.text(&NAME, LABEL_X, y, label, *colour);
                }
                canvas.text(&NAME, elements_x(), y, elements, NAME_TEXT);
            }
            DetailLine::Separator => canvas.fill(
                FRAME_DS_PX + 1,
                y + 1,
                ENEMY_PANEL_DS_PX.0 - 2 * FRAME_DS_PX - 2,
                1,
                SEPARATOR,
            ),
        }
        y += line.height();
    }
}

fn draw_name_strip(canvas: &PixelCanvas, name: &str, level: &str, height: u32) {
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
    let stats_height = height - FRAME_DS_PX - stats_top;
    canvas.fill(
        FRAME_DS_PX,
        stats_top,
        inner_width,
        stats_height,
        STATS_FILL,
    );
    canvas.fill(FRAME_DS_PX, stats_top, 1, stats_height, STATS_EDGE);
    let level_x = draw_level(canvas, level);
    let name = fit_name(name, level_x.saturating_sub(NAME_AT.0 + 2));
    canvas.text(&NAME, NAME_AT.0 + 1, NAME_AT.1 + 1, &name, NAME_SHADOW);
    canvas.text(&NAME, NAME_AT.0, NAME_AT.1, &name, NAME_TEXT);
}

// "LV" and the level, right-aligned in the name strip; returns where it starts.
fn draw_level(canvas: &PixelCanvas, level: &str) -> u32 {
    let digits_x = VALUE_RIGHT_X.saturating_sub(DIGITS.text_width(level));
    let label_x = digits_x.saturating_sub(LABEL.text_width("LV") + 2);
    let y = NAME_AT.1 + 1;
    canvas.outlined_text(&LABEL, label_x, y, "LV", LEVEL_LABEL);
    canvas.text(&DIGITS, digits_x, y, level, NAME_TEXT);
    label_x
}

fn draw_stat_row(
    canvas: &PixelCanvas,
    y: u32,
    label: &str,
    label_colour: Color32,
    bar_colours: [Color32; 2],
    value_and_max: Option<(u32, u32)>,
) {
    canvas.outlined_text(&LABEL, LABEL_X, y, label, label_colour);
    canvas.fill(BAR_X - 1, y, BAR_WIDTH_DS_PX + 2, 4, BLACK);
    canvas.fill(BAR_X, y + 1, BAR_WIDTH_DS_PX, 2, BAR_EMPTY);
    let Some((value, max)) = value_and_max else {
        let digits_x = VALUE_RIGHT_X.saturating_sub(DIGITS.text_width("???"));
        canvas.text(&DIGITS, digits_x, y, "???", NAME_TEXT);
        return;
    };
    let filled = filled_width(value, max, BAR_WIDTH_DS_PX);
    canvas.fill(BAR_X, y + 1, filled, 1, bar_colours[0]);
    canvas.fill(BAR_X, y + 2, filled, 1, bar_colours[1]);
    let digits = value.to_string();
    let digits_x = VALUE_RIGHT_X.saturating_sub(DIGITS.text_width(&digits));
    canvas.text(&DIGITS, digits_x, y, &digits, NAME_TEXT);
}

fn fit_name(name: &str, available: u32) -> String {
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
        assert_eq!(fit_name("Pixie", 40), "Pixie");
        let cut = fit_name("Mother Harlot Mother Harlot", 40);
        assert!(NAME.text_width(&cut) <= 40);
        assert!(cut.starts_with("Mother"));
    }

    fn pixie() -> UnitStatus {
        let mut affinities = [Affinity::Normal; 8];
        affinities[2] = Affinity::Strong;
        UnitStatus {
            race: Some("Fairy".to_string()),
            affinities: Some(affinities),
            skills: vec!["Agi".to_string(), "Dia".to_string()],
            ..UnitStatus::default()
        }
    }

    #[test]
    fn a_demon_card_grows_with_its_details_and_the_hero_without_any_stays_compact() {
        let lines = detail_lines(&pixie());
        assert_eq!(lines.len(), 5, "race, St Fire, separator, Agi, Dia");
        assert_eq!(
            panel_height_ds_px(&pixie()),
            DETAILS_TOP_Y + 4 * TEXT_LINE_DS_PX + SEPARATOR_LINE_DS_PX + 1 + FRAME_DS_PX
        );
        assert_eq!(
            panel_height_ds_px(&UnitStatus::default()),
            ENEMY_PANEL_DS_PX.1
        );
    }

    #[test]
    fn an_unknown_demon_shows_question_marks_instead_of_numbers() {
        let unknown = UnitStatus {
            unknown: true,
            level: 2,
            hp: 36,
            max_hp: 36,
            ..UnitStatus::default()
        };
        assert_eq!(level_text(&unknown), "??");
        assert_eq!(stat(&unknown, unknown.hp, unknown.max_hp), None);
        let known = UnitStatus {
            unknown: false,
            ..unknown
        };
        assert_eq!(level_text(&known), "2");
        assert_eq!(stat(&known, known.hp, known.max_hp), Some((36, 36)));
    }

    #[test]
    fn long_affinity_rows_wrap_under_their_label() {
        let elements = ["Phys", "Fire", "Ice", "Force", "Expel", "Curse"];
        let lines = affinity_lines("Nu", MP_LABEL, &elements);
        assert!(lines.len() > 1);
        for (index, line) in lines.iter().enumerate() {
            let DetailLine::Affinity { label, elements } = line else {
                panic!("not an affinity row");
            };
            assert_eq!(label.is_some(), index == 0);
            assert!(elements_x() + NAME.text_width(elements) <= VALUE_RIGHT_X);
        }
    }

    #[test]
    fn stats_rows_fit_inside_the_frame() {
        assert!(MP_ROW_Y + 4 <= ENEMY_PANEL_DS_PX.1 - FRAME_DS_PX);
        const { assert!(MP_ROW_Y + 5 < DETAILS_TOP_Y) };
        assert!(VALUE_RIGHT_X < ENEMY_PANEL_DS_PX.0 - FRAME_DS_PX);
        assert!(BAR_X + BAR_WIDTH_DS_PX + 1 < VALUE_RIGHT_X - DIGITS.text_width("999"));
    }
}
