use sj_game::layout::{Corner, PipSettings};
use sj_game::settings::{ScalingFilter, VideoSettings, MAX_SCALE};
use sj_game::{NDS_SCREEN_HEIGHT_PX, NDS_SCREEN_WIDTH_PX};

pub fn show(ui: &mut egui::Ui, video: &mut VideoSettings, pip: &mut PipSettings) {
    egui::CollapsingHeader::new("Display")
        .default_open(true)
        .show(ui, |ui| {
            show_video(ui, video);
            ui.separator();
            show_touch_screen(ui, pip);
        });
}

fn show_video(ui: &mut egui::Ui, video: &mut VideoSettings) {
    egui::Grid::new("video").num_columns(2).show(ui, |ui| {
        ui.label("3D resolution");
        let scale = video.scale;
        let resolution = format!(
            "{}x  ({} × {} per screen)",
            scale,
            NDS_SCREEN_WIDTH_PX * scale,
            NDS_SCREEN_HEIGHT_PX * scale
        );
        ui.add(
            egui::Slider::new(&mut video.scale, 1..=MAX_SCALE)
                .show_value(false)
                .text(resolution),
        );
        ui.end_row();

        ui.label("Scaling");
        ui.horizontal(|ui| {
            ui.radio_value(&mut video.filter, ScalingFilter::Smooth, "Smooth");
            ui.radio_value(&mut video.filter, ScalingFilter::Sharp, "Sharp pixels");
        });
        ui.end_row();

        ui.label("Window");
        ui.horizontal(|ui| {
            ui.checkbox(&mut video.fullscreen, "Fullscreen");
            ui.add_enabled_ui(!video.fullscreen, |ui| {
                ui.add(
                    egui::DragValue::new(&mut video.window_width_px)
                        .range(640..=7680)
                        .suffix(" px"),
                );
                ui.label("×");
                ui.add(
                    egui::DragValue::new(&mut video.window_height_px)
                        .range(480..=4320)
                        .suffix(" px"),
                );
            });
        });
        ui.end_row();

        ui.label("Vsync");
        ui.checkbox(&mut video.vsync, "Sync to display (prevents tearing)");
        ui.end_row();
    });
}

fn show_touch_screen(ui: &mut egui::Ui, pip: &mut PipSettings) {
    ui.label("Touch screen overlay");
    egui::Grid::new("touch_screen")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Show");
            ui.checkbox(
                &mut pip.visible,
                "Visible at start (toggle in game with its hotkey)",
            );
            ui.end_row();

            ui.label("Corner");
            egui::ComboBox::from_id_salt("pip_corner")
                .selected_text(corner_label(pip.corner))
                .show_ui(ui, |ui| {
                    for corner in [
                        Corner::TopLeft,
                        Corner::TopRight,
                        Corner::BottomLeft,
                        Corner::BottomRight,
                    ] {
                        ui.selectable_value(&mut pip.corner, corner, corner_label(corner));
                    }
                });
            ui.end_row();

            ui.label("Size");
            ui.add(
                egui::Slider::new(&mut pip.height_fraction, 0.1..=1.0).custom_formatter(
                    |fraction, _| format!("{:.0}% of window height", fraction * 100.0),
                ),
            );
            ui.end_row();

            ui.label("Margin");
            ui.add(
                egui::DragValue::new(&mut pip.margin_px)
                    .range(0..=200)
                    .suffix(" px"),
            );
            ui.end_row();
        });
}

fn corner_label(corner: Corner) -> &'static str {
    match corner {
        Corner::TopLeft => "Top left",
        Corner::TopRight => "Top right",
        Corner::BottomLeft => "Bottom left",
        Corner::BottomRight => "Bottom right",
    }
}
