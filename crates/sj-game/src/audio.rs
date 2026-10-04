use serde::{Deserialize, Serialize};

pub const MAX_VOLUME_PERCENT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub volume_percent: u8,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume_percent: MAX_VOLUME_PERCENT,
            muted: false,
        }
    }
}

pub fn apply_volume(audio: &AudioSettings, stereo_samples: &mut [i16]) {
    let volume_percent = if audio.muted {
        0
    } else {
        audio.volume_percent.min(MAX_VOLUME_PERCENT)
    };
    if volume_percent == MAX_VOLUME_PERCENT {
        return;
    }
    for sample in stereo_samples {
        *sample =
            (i32::from(*sample) * i32::from(volume_percent) / i32::from(MAX_VOLUME_PERCENT)) as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_volume_leaves_samples_alone() {
        let mut samples = [i16::MIN, -1, 0, 1, i16::MAX];
        apply_volume(&AudioSettings::default(), &mut samples);
        assert_eq!(samples, [i16::MIN, -1, 0, 1, i16::MAX]);
    }

    #[test]
    fn half_volume_halves_samples() {
        let mut samples = [1000, -1000];
        apply_volume(
            &AudioSettings {
                volume_percent: 50,
                muted: false,
            },
            &mut samples,
        );
        assert_eq!(samples, [500, -500]);
    }

    #[test]
    fn mute_silences_regardless_of_volume() {
        let mut samples = [1000, -1000];
        apply_volume(
            &AudioSettings {
                volume_percent: 100,
                muted: true,
            },
            &mut samples,
        );
        assert_eq!(samples, [0, 0]);
    }
}
