use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// Discriminants follow RETRO_DEVICE_ID_JOYPAD_* so a mask bit is directly what the core expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NdsButton {
    B = 0,
    Y = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
    A = 8,
    X = 9,
    L = 10,
    R = 11,
}

impl NdsButton {
    pub const ALL: [NdsButton; 12] = [
        NdsButton::A,
        NdsButton::B,
        NdsButton::X,
        NdsButton::Y,
        NdsButton::L,
        NdsButton::R,
        NdsButton::Start,
        NdsButton::Select,
        NdsButton::Up,
        NdsButton::Down,
        NdsButton::Left,
        NdsButton::Right,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NdsButton::A => "A",
            NdsButton::B => "B",
            NdsButton::X => "X",
            NdsButton::Y => "Y",
            NdsButton::L => "L",
            NdsButton::R => "R",
            NdsButton::Start => "Start",
            NdsButton::Select => "Select",
            NdsButton::Up => "Up",
            NdsButton::Down => "Down",
            NdsButton::Left => "Left",
            NdsButton::Right => "Right",
        }
    }
}

pub type Bindings = BTreeMap<NdsButton, Vec<String>>;

pub fn default_bindings() -> Bindings {
    let pairs: [(NdsButton, &[&str]); 12] = [
        (NdsButton::A, &["X", "pad:a"]),
        (NdsButton::B, &["Z", "pad:b"]),
        (NdsButton::X, &["S", "pad:y"]),
        (NdsButton::Y, &["A", "pad:x"]),
        (NdsButton::L, &["Q", "pad:leftshoulder"]),
        (NdsButton::R, &["W", "pad:rightshoulder"]),
        (NdsButton::Start, &["Return", "pad:start"]),
        (NdsButton::Select, &["Right Shift", "pad:back"]),
        (NdsButton::Up, &["Up", "pad:dpup", "pad:lefty-"]),
        (NdsButton::Down, &["Down", "pad:dpdown", "pad:lefty+"]),
        (NdsButton::Left, &["Left", "pad:dpleft", "pad:leftx-"]),
        (NdsButton::Right, &["Right", "pad:dpright", "pad:leftx+"]),
    ];
    pairs
        .into_iter()
        .map(|(button, inputs)| (button, inputs.iter().map(ToString::to_string).collect()))
        .collect()
}

pub fn fill_missing_with_defaults(bindings: &mut Bindings) {
    for (button, inputs) in default_bindings() {
        bindings.entry(button).or_insert(inputs);
    }
}

pub fn button_mask(bindings: &Bindings, is_pressed: impl Fn(&str) -> bool) -> u16 {
    bindings
        .iter()
        .filter(|(_, inputs)| inputs.iter().any(|input| is_pressed(input)))
        .fold(0, |mask, (button, _)| mask | 1 << *button as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_button_has_a_default_binding() {
        let bindings = default_bindings();
        assert!(NdsButton::ALL
            .iter()
            .all(|button| bindings.contains_key(button)));
    }

    #[test]
    fn nothing_pressed_is_empty_mask() {
        assert_eq!(button_mask(&default_bindings(), |_| false), 0);
    }

    #[test]
    fn keyboard_and_pad_bindings_both_trigger_their_button() {
        let bindings = default_bindings();
        assert_eq!(button_mask(&bindings, |input| input == "X"), 1 << 8);
        assert_eq!(button_mask(&bindings, |input| input == "pad:a"), 1 << 8);
    }

    #[test]
    fn simultaneous_buttons_combine() {
        let mask = button_mask(&default_bindings(), |input| input == "Up" || input == "Z");
        assert_eq!(mask, 1 << 4 | 1 << 0);
    }

    #[test]
    fn unbound_input_does_nothing() {
        assert_eq!(button_mask(&default_bindings(), |input| input == "F12"), 0);
    }

    #[test]
    fn rebinding_moves_the_button() {
        let mut bindings = default_bindings();
        bindings.insert(NdsButton::A, vec!["Space".to_string()]);
        assert_eq!(button_mask(&bindings, |input| input == "X"), 0);
        assert_eq!(button_mask(&bindings, |input| input == "Space"), 1 << 8);
    }

    #[test]
    fn missing_buttons_get_default_bindings() {
        let mut bindings = Bindings::from([(NdsButton::A, vec!["Space".to_string()])]);
        fill_missing_with_defaults(&mut bindings);
        assert_eq!(bindings[&NdsButton::A], ["Space"]);
        assert_eq!(bindings[&NdsButton::B], ["Z", "pad:b"]);
    }
}
