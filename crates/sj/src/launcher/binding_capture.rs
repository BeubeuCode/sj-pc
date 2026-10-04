use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sj_game::input::NdsButton;
use sj_game::settings::{HotkeyAction, Settings};

use crate::host_input::{
    key_input_name, pad_axis_input_names, pad_button_input_name, AxisThresholds,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureTarget {
    Button(NdsButton),
    Hotkey(HotkeyAction),
    PadSwapScreens,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Keyboard(String),
    Pad(String),
    Cancelled,
}

pub fn captured_input(event: &Event, thresholds: AxisThresholds) -> Option<Captured> {
    match *event {
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Some(Captured::Cancelled),
        Event::KeyDown {
            keycode: Some(key),
            repeat: false,
            ..
        } => Some(Captured::Keyboard(key_input_name(key))),
        Event::ControllerButtonDown { button, .. } => {
            Some(Captured::Pad(pad_button_input_name(button)))
        }
        Event::ControllerAxisMotion { axis, value, .. } => {
            pad_axis_input_names(axis, value, thresholds)
                .into_iter()
                .find(|(_, pressed)| *pressed)
                .map(|(name, _)| Captured::Pad(name))
        }
        _ => None,
    }
}

pub fn apply_capture(settings: &mut Settings, target: CaptureTarget, captured: Captured) -> bool {
    match (target, captured) {
        (_, Captured::Cancelled) => true,
        (CaptureTarget::Hotkey(_), Captured::Pad(_))
        | (CaptureTarget::PadSwapScreens, Captured::Keyboard(_)) => false,
        (CaptureTarget::PadSwapScreens, Captured::Pad(name)) => {
            settings.controller.swap_screens_button = name;
            true
        }
        (CaptureTarget::Hotkey(action), Captured::Keyboard(key_name)) => {
            settings.hotkeys.set_key(action, key_name);
            true
        }
        (CaptureTarget::Button(button), Captured::Keyboard(name) | Captured::Pad(name)) => {
            let inputs = settings.bindings.entry(button).or_default();
            if !inputs.contains(&name) {
                inputs.push(name);
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::controller::{Axis, Button};

    fn thresholds() -> AxisThresholds {
        AxisThresholds::from_deadzone_percent(50)
    }

    fn key_down(key: Keycode) -> Event {
        Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(key),
            scancode: None,
            keymod: sdl2::keyboard::Mod::NOMOD,
            repeat: false,
        }
    }

    #[test]
    fn escape_cancels() {
        assert_eq!(
            captured_input(&key_down(Keycode::Escape), thresholds()),
            Some(Captured::Cancelled)
        );
    }

    #[test]
    fn key_press_is_captured_by_name() {
        assert_eq!(
            captured_input(&key_down(Keycode::Space), thresholds()),
            Some(Captured::Keyboard("Space".into()))
        );
    }

    #[test]
    fn pad_button_is_captured() {
        let event = Event::ControllerButtonDown {
            timestamp: 0,
            which: 0,
            button: Button::LeftShoulder,
        };
        assert_eq!(
            captured_input(&event, thresholds()),
            Some(Captured::Pad("pad:leftshoulder".into()))
        );
    }

    #[test]
    fn small_stick_wiggle_is_ignored_but_full_push_is_captured() {
        let wiggle = Event::ControllerAxisMotion {
            timestamp: 0,
            which: 0,
            axis: Axis::RightX,
            value: 3_000,
        };
        let push = Event::ControllerAxisMotion {
            timestamp: 0,
            which: 0,
            axis: Axis::RightX,
            value: -30_000,
        };
        assert_eq!(captured_input(&wiggle, thresholds()), None);
        assert_eq!(
            captured_input(&push, thresholds()),
            Some(Captured::Pad("pad:rightx-".into()))
        );
    }

    #[test]
    fn capturing_for_a_button_appends_without_duplicates() {
        let mut settings = Settings::default();
        let target = CaptureTarget::Button(NdsButton::A);
        assert!(apply_capture(
            &mut settings,
            target,
            Captured::Keyboard("K".into())
        ));
        assert!(apply_capture(
            &mut settings,
            target,
            Captured::Keyboard("K".into())
        ));
        assert_eq!(
            settings.bindings[&NdsButton::A],
            ["Return", "Space", "pad:a", "K"]
        );
    }

    #[test]
    fn hotkeys_take_keyboard_and_keep_waiting_on_pad_input() {
        let mut settings = Settings::default();
        let target = CaptureTarget::Hotkey(HotkeyAction::SaveState);
        assert!(!apply_capture(
            &mut settings,
            target,
            Captured::Pad("pad:a".into())
        ));
        assert!(apply_capture(
            &mut settings,
            target,
            Captured::Keyboard("F1".into())
        ));
        assert_eq!(settings.hotkeys.save_state, "F1");
    }

    #[test]
    fn pad_swap_button_takes_pad_input_only() {
        let mut settings = Settings::default();
        let target = CaptureTarget::PadSwapScreens;
        assert!(!apply_capture(
            &mut settings,
            target,
            Captured::Keyboard("M".into())
        ));
        assert!(apply_capture(
            &mut settings,
            target,
            Captured::Pad("pad:leftstick".into())
        ));
        assert_eq!(settings.controller.swap_screens_button, "pad:leftstick");
    }

    #[test]
    fn cancelling_changes_nothing() {
        let mut settings = Settings::default();
        assert!(apply_capture(
            &mut settings,
            CaptureTarget::Button(NdsButton::B),
            Captured::Cancelled
        ));
        assert_eq!(settings, Settings::default());
    }
}
