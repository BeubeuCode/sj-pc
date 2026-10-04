use std::path::{Path, PathBuf};

use crate::image::{split_screens, RgbImage};
use crate::png;

pub struct Snapshot<'a> {
    pub label: &'a str,
    pub savestate: &'a [u8],
    pub main_ram: &'a [u8],
    pub frame: Option<&'a RgbImage>,
}

pub fn write(samples_dir: &Path, snapshot: &Snapshot) -> std::io::Result<PathBuf> {
    let directory = next_free_directory(samples_dir, &clean_label(snapshot.label));
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("savestate.state"), snapshot.savestate)?;
    std::fs::write(directory.join("main_ram.bin"), snapshot.main_ram)?;
    if let Some(frame) = snapshot.frame {
        write_screens(&directory, frame)?;
    }
    Ok(directory)
}

fn write_screens(directory: &Path, frame: &RgbImage) -> std::io::Result<()> {
    let Some((top, bottom)) = split_screens(frame) else {
        return std::fs::write(directory.join("frame.png"), png::encode(frame));
    };
    std::fs::write(directory.join("top.png"), png::encode(&top))?;
    std::fs::write(directory.join("bottom.png"), png::encode(&bottom))
}

fn clean_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        return "snapshot".to_string();
    }
    cleaned
}

fn next_free_directory(samples_dir: &Path, label: &str) -> PathBuf {
    (1..=u32::MAX)
        .map(|number| samples_dir.join(format!("{label}-{number}")))
        .find(|candidate| !candidate.exists())
        .expect("an unused snapshot number always exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_samples_dir(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("sj-snapshot-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn labels_are_made_filesystem_safe() {
        assert_eq!(clean_label("first battle/turn 1"), "first_battle_turn_1");
        assert_eq!(clean_label(""), "snapshot");
    }

    #[test]
    fn writes_state_ram_and_both_screens_in_numbered_folders() {
        let samples_dir = temp_samples_dir("numbered");
        let frame = RgbImage {
            width_px: 256,
            height_px: 384,
            rgb: vec![0; 256 * 384 * 3],
        };
        let snapshot = Snapshot {
            label: "battle",
            savestate: b"state",
            main_ram: b"ram",
            frame: Some(&frame),
        };

        let first = write(&samples_dir, &snapshot).unwrap();
        let second = write(&samples_dir, &snapshot).unwrap();

        assert_eq!(first, samples_dir.join("battle-1"));
        assert_eq!(second, samples_dir.join("battle-2"));
        assert_eq!(std::fs::read(first.join("main_ram.bin")).unwrap(), b"ram");
        assert!(first.join("top.png").exists() && first.join("bottom.png").exists());
        std::fs::remove_dir_all(samples_dir).unwrap();
    }
}
