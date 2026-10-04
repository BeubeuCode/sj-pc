use crate::{NDS_SCREEN_HEIGHT_PX, NDS_SCREEN_WIDTH_PX};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbImage {
    pub width_px: u32,
    pub height_px: u32,
    pub rgb: Vec<u8>,
}

impl RgbImage {
    pub fn from_xrgb(width_px: u32, height_px: u32, xrgb_pixels: &[u32]) -> Self {
        let rgb = xrgb_pixels
            .iter()
            .flat_map(|pixel| {
                let [blue, green, red, _] = pixel.to_le_bytes();
                [red, green, blue]
            })
            .collect();
        Self {
            width_px,
            height_px,
            rgb,
        }
    }

    #[must_use]
    pub fn rows(&self, first_row: u32, row_count: u32) -> Self {
        let row_bytes = self.width_px as usize * 3;
        let start = first_row as usize * row_bytes;
        let end = start + row_count as usize * row_bytes;
        Self {
            width_px: self.width_px,
            height_px: row_count,
            rgb: self.rgb[start..end].to_vec(),
        }
    }
}

// The core is pinned to the gapless top-bottom layout, so a frame is the top screen stacked on the
// touch screen, each a native 256x192 or a scaled multiple of it.
pub fn split_screens(frame: &RgbImage) -> Option<(RgbImage, RgbImage)> {
    let is_stacked =
        frame.width_px * NDS_SCREEN_HEIGHT_PX * 2 == frame.height_px * NDS_SCREEN_WIDTH_PX;
    if !is_stacked || frame.rgb.len() != (frame.width_px * frame.height_px * 3) as usize {
        return None;
    }
    let screen_height_px = frame.height_px / 2;
    Some((
        frame.rows(0, screen_height_px),
        frame.rows(screen_height_px, screen_height_px),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xrgb_becomes_rgb_bytes() {
        let image = RgbImage::from_xrgb(1, 1, &[0x00AA_BBCC]);
        assert_eq!(image.rgb, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn stacked_frame_splits_into_top_and_bottom() {
        let mut xrgb = vec![0x00FF_0000; 256 * 192];
        xrgb.extend(vec![0x0000_00FF; 256 * 192]);
        let (top, bottom) = split_screens(&RgbImage::from_xrgb(256, 384, &xrgb)).unwrap();
        assert_eq!((top.width_px, top.height_px), (256, 192));
        assert_eq!(top.rgb[..3], [0xFF, 0, 0]);
        assert_eq!(bottom.rgb[..3], [0, 0, 0xFF]);
    }

    #[test]
    fn frame_that_is_not_two_stacked_screens_is_rejected() {
        let image = RgbImage::from_xrgb(256, 192, &vec![0; 256 * 192]);
        assert_eq!(split_screens(&image), None);
    }
}
