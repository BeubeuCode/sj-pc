#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvInfo {
    pub base_width_px: u32,
    pub base_height_px: u32,
    pub max_width_px: u32,
    pub max_height_px: u32,
    pub fps: f64,
    pub sample_rate_hz: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Software {
        width_px: u32,
        height_px: u32,
        xrgb_pixels: Vec<u32>,
    },
    Hardware {
        width_px: u32,
        height_px: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pointer {
    pub x: i16,
    pub y: i16,
    pub pressed: bool,
}

#[derive(Debug, Default)]
pub struct FrameOutput {
    pub frame: Option<Frame>,
    pub stereo_samples: Vec<i16>,
    pub new_av_info: Option<AvInfo>,
}
