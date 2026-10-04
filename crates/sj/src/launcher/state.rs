use std::path::{Path, PathBuf};

use egui::{Color32, RichText};
use sj_game::rom::{identify_file, RomStatus};
use sj_game::settings::{save, Settings};

use super::binding_capture::CaptureTarget;
use super::{controls_section, display_section, game_section, sound_section};
use crate::savestate::autosave_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Play,
    Resume,
    Quit,
}

pub struct LauncherState {
    pub settings: Settings,
    pub settings_path: PathBuf,
    pub rom_text: String,
    pub save_dir_text: String,
    pub core_text: String,
    pub rom_status: RomStatus,
    pub capture: Option<CaptureTarget>,
    pub controller_names: Vec<String>,
    pub last_pad_input: Option<String>,
    pub message: Option<String>,
    pub outcome: Option<Outcome>,
}

impl LauncherState {
    pub fn new(settings: Settings, settings_path: &Path, message: Option<String>) -> Self {
        let rom_status = identify_file(&settings.rom);
        Self {
            rom_text: settings.rom.display().to_string(),
            save_dir_text: settings.save_dir.display().to_string(),
            core_text: settings.core.display().to_string(),
            settings,
            settings_path: settings_path.to_path_buf(),
            rom_status,
            capture: None,
            controller_names: Vec::new(),
            last_pad_input: None,
            message,
            outcome: None,
        }
    }

    pub fn set_rom_path(&mut self, path: String) {
        self.rom_text = path;
        self.apply_path_fields();
    }

    pub fn apply_path_fields(&mut self) {
        self.settings.rom = PathBuf::from(self.rom_text.trim());
        self.settings.save_dir = PathBuf::from(self.save_dir_text.trim());
        self.settings.core = PathBuf::from(self.core_text.trim());
        self.rom_status = identify_file(&self.settings.rom);
    }

    pub fn core_found(&self) -> bool {
        self.settings.core.is_file()
    }

    pub fn ready_to_play(&self) -> bool {
        self.rom_status == RomStatus::StrangeJourneyUsa && self.core_found()
    }

    pub fn show(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("actions").show(root, |ui| self.show_actions(ui));
        egui::CentralPanel::default().show(root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Shin Megami Tensei: Strange Journey");
                ui.add_space(8.0);
                game_section::show(ui, self);
                display_section::show(
                    ui,
                    &mut self.settings.video,
                    &mut self.settings.hud,
                    &mut self.settings.pip,
                );
                sound_section::show(ui, &mut self.settings.audio);
                controls_section::show(
                    ui,
                    &mut self.settings,
                    &mut self.capture,
                    &self.controller_names,
                    self.last_pad_input.as_deref(),
                );
            });
        });
    }

    fn show_actions(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let play = egui::Button::new(RichText::new("  Play  ").size(18.0).strong());
            if ui.add_enabled(self.ready_to_play(), play).clicked() {
                self.finish(Outcome::Play);
            }
            let can_resume =
                self.ready_to_play() && autosave_path(&self.settings.save_dir).is_file();
            if ui
                .add_enabled(can_resume, egui::Button::new("Resume last session"))
                .on_hover_text("Starts from the autosave taken every 5 minutes and on quit")
                .clicked()
            {
                self.finish(Outcome::Resume);
            }
            if ui.button("Save settings").clicked() {
                self.save_settings();
            }
            if ui.button("Reset all to defaults").clicked() {
                let controller_names = std::mem::take(&mut self.controller_names);
                *self = Self::new(
                    Settings::default(),
                    &self.settings_path,
                    Some("Defaults restored, not saved yet.".into()),
                );
                self.controller_names = controller_names;
            }
            if ui.button("Quit").clicked() {
                self.outcome = Some(Outcome::Quit);
            }
        });
        if let Some(message) = &self.message {
            ui.colored_label(Color32::LIGHT_YELLOW, message);
        }
        ui.add_space(6.0);
    }

    fn finish(&mut self, outcome: Outcome) {
        if self.save_settings() {
            self.outcome = Some(outcome);
        }
    }

    fn save_settings(&mut self) -> bool {
        self.apply_path_fields();
        match save(&self.settings_path, &self.settings) {
            Ok(()) => {
                self.message = Some(format!("Saved to {}.", self.settings_path.display()));
                true
            }
            Err(error) => {
                self.message = Some(format!("Could not save: {error}"));
                false
            }
        }
    }
}
