use sj_game::audio::{AudioSettings, MAX_VOLUME_PERCENT};

pub fn show(ui: &mut egui::Ui, audio: &mut AudioSettings) {
    egui::CollapsingHeader::new("Sound")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled(
                    !audio.muted,
                    egui::Slider::new(&mut audio.volume_percent, 0..=MAX_VOLUME_PERCENT)
                        .suffix("%")
                        .text("Volume"),
                );
                ui.checkbox(&mut audio.muted, "Mute");
            });
        });
}
