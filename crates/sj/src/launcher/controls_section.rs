use egui::{Color32, RichText};
use sj_game::input::{default_bindings, NdsButton};
use sj_game::settings::{
    ControllerSettings, HotkeyAction, Hotkeys, Settings, CURRENT_BINDINGS_VERSION,
    MAX_DEADZONE_PERCENT, MIN_DEADZONE_PERCENT,
};

use super::binding_capture::CaptureTarget;

const WAITING_TEXT: &str = "Press a key or controller button… (Esc cancels)";

pub fn show(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    capture: &mut Option<CaptureTarget>,
    controller_names: &[String],
    last_pad_input: Option<&str>,
) {
    egui::CollapsingHeader::new("Controls")
        .default_open(true)
        .show(ui, |ui| {
            show_controllers(ui, controller_names, last_pad_input);
            show_legacy_notice(ui, settings, capture);
            ui.add(
                egui::Slider::new(
                    &mut settings.controller.stick_deadzone_percent,
                    MIN_DEADZONE_PERCENT..=MAX_DEADZONE_PERCENT,
                )
                .suffix("%")
                .text("Stick deadzone"),
            );
            ui.add_space(6.0);
            show_button_bindings(ui, settings, capture);
            ui.add_space(6.0);
            show_hotkeys(ui, &mut settings.hotkeys, capture);
            ui.horizontal(|ui| {
                ui.label("Swap screens (controller)");
                let current = display_name(&settings.controller.swap_screens_button);
                show_capture_button(ui, capture, CaptureTarget::PadSwapScreens, &current);
            });
            ui.add_space(6.0);
            if ui.button("Reset controls to defaults").clicked() {
                reset_controls(settings, capture);
            }
        });
}

fn show_controllers(ui: &mut egui::Ui, controller_names: &[String], last_pad_input: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Controllers").strong());
        if controller_names.is_empty() {
            ui.colored_label(Color32::LIGHT_RED, "none connected");
            return;
        }
        ui.colored_label(Color32::LIGHT_GREEN, controller_names.join(", "));
        let last_input =
            last_pad_input.map_or_else(|| "press any button to test".to_string(), display_name);
        ui.label(format!("· last input: {last_input}"));
    });
}

fn show_legacy_notice(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    capture: &mut Option<CaptureTarget>,
) {
    if settings.bindings_version >= CURRENT_BINDINGS_VERSION {
        return;
    }
    ui.colored_label(
        Color32::LIGHT_YELLOW,
        "New default controls: the pad maps by label (Y opens the menu) and the keyboard uses WASD, Enter, Escape and Tab.",
    );
    ui.horizontal(|ui| {
        if ui.button("Use new default controls").clicked() {
            reset_controls(settings, capture);
        }
        if ui.button("Keep mine").clicked() {
            settings.bindings_version = CURRENT_BINDINGS_VERSION;
        }
    });
}

fn reset_controls(settings: &mut Settings, capture: &mut Option<CaptureTarget>) {
    settings.bindings = default_bindings();
    settings.hotkeys = Hotkeys::default();
    settings.controller = ControllerSettings::default();
    settings.bindings_version = CURRENT_BINDINGS_VERSION;
    *capture = None;
}

fn show_button_bindings(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    capture: &mut Option<CaptureTarget>,
) {
    ui.label(RichText::new("DS buttons").strong());
    egui::Grid::new("button_bindings")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            for button in NdsButton::ALL {
                ui.label(button.label());
                let inputs = settings.bindings.entry(button).or_default();
                ui.horizontal(|ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    inputs.retain(|input| {
                        !ui.button(format!("{}  ×", display_name(input)))
                            .on_hover_text("Click to remove")
                            .clicked()
                    });
                });
                show_capture_button(ui, capture, CaptureTarget::Button(button), "+ Add");
                ui.end_row();
            }
        });
}

fn show_hotkeys(ui: &mut egui::Ui, hotkeys: &mut Hotkeys, capture: &mut Option<CaptureTarget>) {
    ui.label(RichText::new("Hotkeys (keyboard)").strong());
    egui::Grid::new("hotkeys")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for action in HotkeyAction::ALL {
                ui.label(action.label());
                show_capture_button(
                    ui,
                    capture,
                    CaptureTarget::Hotkey(action),
                    hotkeys.key_for(action),
                );
                ui.end_row();
            }
        });
}

fn show_capture_button(
    ui: &mut egui::Ui,
    capture: &mut Option<CaptureTarget>,
    target: CaptureTarget,
    idle_text: &str,
) {
    if *capture == Some(target) {
        ui.colored_label(Color32::LIGHT_YELLOW, WAITING_TEXT);
        return;
    }
    if ui.button(idle_text).clicked() {
        *capture = Some(target);
    }
}

fn display_name(input: &str) -> String {
    match input.strip_prefix("pad:") {
        Some(pad_input) => format!("Pad {pad_input}"),
        None => input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_inputs_read_naturally() {
        assert_eq!(display_name("pad:leftshoulder"), "Pad leftshoulder");
        assert_eq!(display_name("Right Shift"), "Right Shift");
    }
}
