use std::path::PathBuf;

use crate::input::NdsButton;

pub const PRESS_HOLD_FRAMES: u32 = 4;
pub const PRESS_RELEASE_FRAMES: u32 = 8;
const MASH_HOLD_FRAMES: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Wait {
        frames: u32,
    },
    Hold {
        buttons: u16,
        frames: u32,
    },
    Press {
        buttons: u16,
        times: u32,
    },
    Mash {
        buttons: u16,
        frames: u32,
        every: u32,
    },
    Snapshot {
        label: String,
    },
    SaveState {
        path: PathBuf,
    },
    LoadState {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {message}")]
pub struct ScriptError {
    pub line: usize,
    pub message: String,
}

pub fn parse(script: &str) -> Result<Vec<Step>, ScriptError> {
    script
        .lines()
        .enumerate()
        .map(|(index, text)| (index + 1, strip_comment(text)))
        .filter(|(_, text)| !text.is_empty())
        .map(|(line, text)| parse_line(text).map_err(|message| ScriptError { line, message }))
        .collect()
}

pub fn frame_masks(step: &Step) -> Vec<u16> {
    match *step {
        Step::Wait { frames } => vec![0; frames as usize],
        Step::Hold { buttons, frames } => vec![buttons; frames as usize],
        Step::Press { buttons, times } => (0..times).flat_map(|_| press_once(buttons)).collect(),
        Step::Mash {
            buttons,
            frames,
            every,
        } => (0..frames)
            .map(|frame| {
                if frame % every.max(1) < MASH_HOLD_FRAMES {
                    buttons
                } else {
                    0
                }
            })
            .collect(),
        Step::Snapshot { .. } | Step::SaveState { .. } | Step::LoadState { .. } => Vec::new(),
    }
}

fn press_once(buttons: u16) -> Vec<u16> {
    let mut masks = vec![buttons; PRESS_HOLD_FRAMES as usize];
    masks.extend(vec![0; PRESS_RELEASE_FRAMES as usize]);
    masks
}

fn strip_comment(text: &str) -> &str {
    text.split('#').next().unwrap_or_default().trim()
}

fn parse_line(text: &str) -> Result<Step, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    match words.as_slice() {
        ["wait", frames] => Ok(Step::Wait {
            frames: number(frames)?,
        }),
        ["hold", buttons, frames] => Ok(Step::Hold {
            buttons: button_mask(buttons)?,
            frames: number(frames)?,
        }),
        ["press", buttons] => Ok(Step::Press {
            buttons: button_mask(buttons)?,
            times: 1,
        }),
        ["press", buttons, times] => Ok(Step::Press {
            buttons: button_mask(buttons)?,
            times: number(times)?,
        }),
        ["mash", buttons, frames, "every", every] => Ok(Step::Mash {
            buttons: button_mask(buttons)?,
            frames: number(frames)?,
            every: number(every)?,
        }),
        ["snapshot", label] => Ok(Step::Snapshot {
            label: (*label).to_string(),
        }),
        ["save", path] => Ok(Step::SaveState {
            path: PathBuf::from(path),
        }),
        ["load", path] => Ok(Step::LoadState {
            path: PathBuf::from(path),
        }),
        _ => Err(format!("cannot understand `{text}`")),
    }
}

fn number(text: &str) -> Result<u32, String> {
    text.parse()
        .map_err(|_| format!("`{text}` is not a whole number"))
}

fn button_mask(text: &str) -> Result<u16, String> {
    text.split('+').try_fold(0u16, |mask, name| {
        let button =
            NdsButton::from_name(name).ok_or_else(|| format!("unknown button `{name}`"))?;
        Ok(mask | 1 << button as u16)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: u16 = 1 << NdsButton::A as u16;
    const UP: u16 = 1 << NdsButton::Up as u16;

    #[test]
    fn parses_every_command_and_skips_comments() {
        let script = "# boot to title\nwait 60\nhold up 30  # walk\npress a 3\nmash a+up 100 every 20\nsnapshot title\nsave cp/title.state\nload cp/title.state\n";
        assert_eq!(
            parse(script).unwrap(),
            vec![
                Step::Wait { frames: 60 },
                Step::Hold {
                    buttons: UP,
                    frames: 30
                },
                Step::Press {
                    buttons: A,
                    times: 3
                },
                Step::Mash {
                    buttons: A | UP,
                    frames: 100,
                    every: 20
                },
                Step::Snapshot {
                    label: "title".into()
                },
                Step::SaveState {
                    path: "cp/title.state".into()
                },
                Step::LoadState {
                    path: "cp/title.state".into()
                },
            ]
        );
    }

    #[test]
    fn errors_name_the_line() {
        let error = parse("wait 10\npress turbo\n").unwrap_err();
        assert_eq!(
            error,
            ScriptError {
                line: 2,
                message: "unknown button `turbo`".into()
            }
        );
    }

    #[test]
    fn press_holds_then_releases() {
        let masks = frame_masks(&Step::Press {
            buttons: A,
            times: 2,
        });
        assert_eq!(
            masks.len(),
            2 * (PRESS_HOLD_FRAMES + PRESS_RELEASE_FRAMES) as usize
        );
        assert_eq!(masks[0], A);
        assert_eq!(masks[PRESS_HOLD_FRAMES as usize], 0);
    }

    #[test]
    fn mash_taps_on_a_period() {
        let masks = frame_masks(&Step::Mash {
            buttons: A,
            frames: 10,
            every: 5,
        });
        assert_eq!(masks, [A, A, 0, 0, 0, A, A, 0, 0, 0]);
    }
}
