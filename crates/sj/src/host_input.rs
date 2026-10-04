use std::collections::HashSet;

use sdl2::controller::{Axis, Button};
use sdl2::keyboard::Keycode;

const AXIS_MAX: i32 = i16::MAX as i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisThresholds {
    pub stick: i16,
    pub trigger: i16,
}

impl AxisThresholds {
    pub fn from_deadzone_percent(stick_deadzone_percent: u8) -> Self {
        let stick = AXIS_MAX * i32::from(stick_deadzone_percent) / 100;
        Self {
            stick: stick as i16,
            trigger: (AXIS_MAX / 4) as i16,
        }
    }
}

pub fn key_input_name(key: Keycode) -> String {
    key.name()
}

pub fn pad_button_input_name(button: Button) -> String {
    format!("pad:{}", button.string())
}

pub fn pad_axis_input_names(
    axis: Axis,
    value: i16,
    thresholds: AxisThresholds,
) -> Vec<(String, bool)> {
    let name = format!("pad:{}", axis.string());
    if matches!(axis, Axis::TriggerLeft | Axis::TriggerRight) {
        return vec![(name, value > thresholds.trigger)];
    }
    vec![
        (format!("{name}-"), value < -thresholds.stick),
        (format!("{name}+"), value > thresholds.stick),
    ]
}

pub struct HostInput {
    pressed: HashSet<String>,
    thresholds: AxisThresholds,
}

impl HostInput {
    pub fn new(thresholds: AxisThresholds) -> Self {
        Self {
            pressed: HashSet::new(),
            thresholds,
        }
    }

    pub fn is_pressed(&self, input_name: &str) -> bool {
        self.pressed.contains(input_name)
    }

    pub fn set_key(&mut self, key: Keycode, down: bool) {
        self.set(key_input_name(key), down);
    }

    pub fn set_pad_button(&mut self, button: Button, down: bool) {
        self.set(pad_button_input_name(button), down);
    }

    pub fn set_pad_axis(&mut self, axis: Axis, value: i16) {
        for (name, down) in pad_axis_input_names(axis, value, self.thresholds) {
            self.set(name, down);
        }
    }

    pub fn release_all_pad_inputs(&mut self) {
        self.pressed.retain(|name| !name.starts_with("pad:"));
    }

    fn set(&mut self, name: String, down: bool) {
        if down {
            self.pressed.insert(name);
        } else {
            self.pressed.remove(&name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> HostInput {
        HostInput::new(AxisThresholds::from_deadzone_percent(50))
    }

    #[test]
    fn deadzone_percent_scales_stick_threshold() {
        assert_eq!(AxisThresholds::from_deadzone_percent(50).stick, 16_383);
        assert_eq!(AxisThresholds::from_deadzone_percent(20).stick, 6_553);
    }

    #[test]
    fn stick_past_threshold_presses_direction() {
        let mut input = input();
        input.set_pad_axis(Axis::LeftY, -20_000);
        assert!(input.is_pressed("pad:lefty-"));
        assert!(!input.is_pressed("pad:lefty+"));
    }

    #[test]
    fn smaller_deadzone_reacts_to_lighter_push() {
        let mut input = HostInput::new(AxisThresholds::from_deadzone_percent(20));
        input.set_pad_axis(Axis::LeftX, 10_000);
        assert!(input.is_pressed("pad:leftx+"));
    }

    #[test]
    fn stick_back_in_deadzone_releases_direction() {
        let mut input = input();
        input.set_pad_axis(Axis::LeftX, 20_000);
        input.set_pad_axis(Axis::LeftX, 1_000);
        assert!(!input.is_pressed("pad:leftx+"));
    }

    #[test]
    fn trigger_acts_as_button() {
        let mut input = input();
        input.set_pad_axis(Axis::TriggerLeft, 30_000);
        assert!(input.is_pressed("pad:lefttrigger"));
    }

    #[test]
    fn unplugging_releases_pad_but_not_keyboard() {
        let mut input = input();
        input.set_key(Keycode::X, true);
        input.set_pad_button(Button::A, true);
        input.release_all_pad_inputs();
        assert!(input.is_pressed("X"));
        assert!(!input.is_pressed("pad:a"));
    }
}
