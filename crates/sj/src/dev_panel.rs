use egui::RichText;
use sj_game::ram_search::{offset_to_addr, value_at, Filter, RamSearch, ValueWidth};

const MAX_LISTED_CANDIDATES: usize = 200;

struct Watch {
    offset: u32,
    width: ValueWidth,
}

pub struct DevPanel {
    pub open: bool,
    width: ValueWidth,
    value_text: String,
    search: Option<RamSearch>,
    watches: Vec<Watch>,
}

impl DevPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            width: ValueWidth::U16,
            value_text: String::new(),
            search: None,
            watches: Vec::new(),
        }
    }

    pub fn show(&mut self, root: &mut egui::Ui, ram: &[u8], status: &str) {
        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("RAM search")
            .open(&mut open)
            .default_width(360.0)
            .show(root.ctx(), |ui| {
                ui.monospace(status);
                ui.separator();
                self.show_controls(ui, ram);
                ui.separator();
                self.show_candidates(ui, ram);
                ui.separator();
                self.show_watches(ui, ram);
            });
        self.open = open;
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, ram: &[u8]) {
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.width, ValueWidth::U8, "u8");
            ui.radio_value(&mut self.width, ValueWidth::U16, "u16");
            ui.radio_value(&mut self.width, ValueWidth::U32, "u32");
            if ui.button("New search").clicked() {
                self.search = Some(RamSearch::start(ram, self.width));
            }
        });
        let Some(search) = &mut self.search else {
            ui.label("Press New search to snapshot main RAM.");
            return;
        };
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.value_text)
                    .desired_width(80.0)
                    .hint_text("value"),
            );
            if ui.button("= value").clicked() {
                if let Ok(wanted) = parse_number(&self.value_text) {
                    search.refine(ram, Filter::Equals(wanted));
                }
            }
        });
        ui.horizontal(|ui| {
            for (label, filter) in [
                ("changed", Filter::Changed),
                ("unchanged", Filter::Unchanged),
                ("increased", Filter::Increased),
                ("decreased", Filter::Decreased),
            ] {
                if ui.button(label).clicked() {
                    search.refine(ram, filter);
                }
            }
        });
        ui.label(format!("{} candidates", search.candidates().len()));
    }

    fn show_candidates(&mut self, ui: &mut egui::Ui, ram: &[u8]) {
        let Some(search) = &self.search else {
            return;
        };
        let width = search.width;
        let mut to_watch = None;
        egui::ScrollArea::vertical()
            .id_salt("candidates")
            .max_height(180.0)
            .show(ui, |ui| {
                for offset in search.candidates().iter().take(MAX_LISTED_CANDIDATES) {
                    ui.horizontal(|ui| {
                        ui.monospace(format!(
                            "{}  {}",
                            offset_to_addr(*offset),
                            value_at(ram, *offset, width)
                        ));
                        if ui.small_button("watch").clicked() {
                            to_watch = Some(*offset);
                        }
                    });
                }
            });
        if let Some(offset) = to_watch {
            self.watches.push(Watch { offset, width });
        }
    }

    fn show_watches(&mut self, ui: &mut egui::Ui, ram: &[u8]) {
        ui.label(RichText::new("Watch list").strong());
        self.watches.retain(|watch| {
            let mut keep = true;
            ui.horizontal(|ui| {
                ui.monospace(format!(
                    "{}  {}",
                    offset_to_addr(watch.offset),
                    value_at(ram, watch.offset, watch.width)
                ));
                keep = !ui.small_button("×").clicked();
            });
            keep
        });
    }
}

fn parse_number(text: &str) -> Result<u32, std::num::ParseIntError> {
    let text = text.trim();
    match text.strip_prefix("0x") {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => text.parse(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_parse_as_decimal_or_hex() {
        assert_eq!(parse_number("56"), Ok(56));
        assert_eq!(parse_number(" 0x38 "), Ok(56));
        assert!(parse_number("hp").is_err());
    }
}
