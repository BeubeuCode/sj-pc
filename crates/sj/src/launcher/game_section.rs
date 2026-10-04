use egui::Color32;
use sj_game::rom::{RomStatus, STRANGE_JOURNEY_USA_CODE};

use super::state::LauncherState;

pub fn show(ui: &mut egui::Ui, state: &mut LauncherState) {
    egui::CollapsingHeader::new("Game")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("game_paths").num_columns(2).show(ui, |ui| {
                ui.label("ROM");
                if ui
                    .add(egui::TextEdit::singleline(&mut state.rom_text).desired_width(420.0))
                    .changed()
                {
                    state.apply_path_fields();
                }
                ui.end_row();
                ui.label("");
                show_rom_status(ui, &state.rom_status);
                ui.end_row();
                ui.label("Save folder");
                if ui
                    .add(egui::TextEdit::singleline(&mut state.save_dir_text).desired_width(420.0))
                    .changed()
                {
                    state.apply_path_fields();
                }
                ui.end_row();
            });
            ui.label("Tip: drag and drop your .nds file onto this window.");
            ui.collapsing("Advanced", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Emulator core");
                    if ui
                        .add(egui::TextEdit::singleline(&mut state.core_text).desired_width(380.0))
                        .changed()
                    {
                        state.apply_path_fields();
                    }
                });
                if !state.core_found() {
                    ui.colored_label(
                        Color32::LIGHT_RED,
                        "Core not found. Build it with scripts/build-core.sh.",
                    );
                }
            });
        });
}

fn show_rom_status(ui: &mut egui::Ui, status: &RomStatus) {
    let (color, text) = match status {
        RomStatus::StrangeJourneyUsa => (
            Color32::LIGHT_GREEN,
            "Strange Journey (USA) found.".to_string(),
        ),
        RomStatus::OtherGame { game_code } => (
            Color32::LIGHT_RED,
            format!(
                "This is game {game_code}, not Strange Journey USA ({STRANGE_JOURNEY_USA_CODE})."
            ),
        ),
        RomStatus::NotAnNdsFile => (Color32::LIGHT_RED, "This is not a DS ROM.".to_string()),
        RomStatus::Missing => (Color32::LIGHT_RED, "No file at this path.".to_string()),
        RomStatus::Unreadable(error) => {
            (Color32::LIGHT_RED, format!("Cannot read the file: {error}"))
        }
    };
    ui.colored_label(color, text);
}
