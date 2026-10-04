use crate::rect::RectPx;
use crate::{NDS_SCREEN_HEIGHT_PX, NDS_SCREEN_WIDTH_PX};

const POINTER_MIN: i32 = -0x7FFF;
const POINTER_SPAN: i32 = 0xFFFE;

pub fn mouse_to_bottom_screen(pip: RectPx, mouse_x: i32, mouse_y: i32) -> Option<(u32, u32)> {
    if !pip.contains(mouse_x, mouse_y) {
        return None;
    }
    let screen_x = (mouse_x - pip.x) as u32 * NDS_SCREEN_WIDTH_PX / pip.width;
    let screen_y = (mouse_y - pip.y) as u32 * NDS_SCREEN_HEIGHT_PX / pip.height;
    Some((screen_x, screen_y))
}

// The core runs with its "top-bottom" layout and no gap, so the libretro pointer space covers a
// 256x384 surface whose lower half is the touch screen. In widescreen the surface is 4/3 wider
// and the touch screen sits centred in it, 256/6 pixels in. We aim at pixel centres so the core's
// truncating inverse lands on the same pixel.
pub fn bottom_screen_to_pointer(screen_x: u32, screen_y: u32, wide_frame: bool) -> (i16, i16) {
    let surface_height_px = NDS_SCREEN_HEIGHT_PX * 2;
    let pointer_x = if wide_frame {
        let width_px = f64::from(NDS_SCREEN_WIDTH_PX) * 4.0 / 3.0;
        let offset_px = f64::from(NDS_SCREEN_WIDTH_PX) / 6.0;
        let fraction = (offset_px + f64::from(screen_x) + 0.5) / width_px;
        (f64::from(POINTER_MIN) + fraction * f64::from(POINTER_SPAN)) as i16
    } else {
        to_pointer_axis(screen_x, NDS_SCREEN_WIDTH_PX)
    };
    let pointer_y = to_pointer_axis(NDS_SCREEN_HEIGHT_PX + screen_y, surface_height_px);
    (pointer_x, pointer_y)
}

fn to_pointer_axis(pixel: u32, extent_px: u32) -> i16 {
    let pixel_centre = i64::from(2 * pixel + 1);
    let scaled = pixel_centre * i64::from(POINTER_SPAN) / (2 * i64::from(extent_px));
    (i64::from(POINTER_MIN) + scaled) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIP: RectPx = RectPx {
        x: 100,
        y: 50,
        width: 512,
        height: 384,
    };

    fn core_inverse(pointer: i16, extent_px: u32) -> u32 {
        ((i32::from(pointer) - POINTER_MIN) as u32 * extent_px / POINTER_SPAN as u32)
            .min(extent_px - 1)
    }

    #[test]
    fn click_outside_pip_is_not_a_touch() {
        assert_eq!(mouse_to_bottom_screen(PIP, 99, 60), None);
    }

    #[test]
    fn pip_corners_map_to_screen_corners() {
        assert_eq!(mouse_to_bottom_screen(PIP, 100, 50), Some((0, 0)));
        assert_eq!(mouse_to_bottom_screen(PIP, 611, 433), Some((255, 191)));
    }

    #[test]
    fn pip_centre_maps_to_screen_centre() {
        assert_eq!(mouse_to_bottom_screen(PIP, 356, 242), Some((128, 96)));
    }

    #[test]
    fn every_bottom_screen_pixel_round_trips_through_core_pointer_space() {
        for (x, y) in [(0, 0), (255, 191), (128, 96), (17, 140)] {
            let (pointer_x, pointer_y) = bottom_screen_to_pointer(x, y, false);
            assert_eq!(core_inverse(pointer_x, NDS_SCREEN_WIDTH_PX), x);
            assert_eq!(
                core_inverse(pointer_y, NDS_SCREEN_HEIGHT_PX * 2),
                NDS_SCREEN_HEIGHT_PX + y
            );
        }
    }

    // The core maps the pointer over the whole wide surface, then removes the bottom screen's offset.
    #[test]
    fn widescreen_touches_land_on_the_centred_bottom_screen() {
        let scale = 3;
        let width_px = NDS_SCREEN_WIDTH_PX * scale * 4 / 3;
        let offset_px = NDS_SCREEN_WIDTH_PX * scale / 6;
        for x in [0, 17, 128, 255] {
            let (pointer_x, _) = bottom_screen_to_pointer(x, 96, true);
            let surface_x = core_inverse(pointer_x, width_px);
            assert_eq!((surface_x - offset_px) / scale, x);
        }
    }
}
