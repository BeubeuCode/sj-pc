use std::path::PathBuf;

pub struct CaptureRequest {
    pub after_frames: u64,
    pub path: PathBuf,
}

pub fn request_from_env() -> Option<CaptureRequest> {
    let after_frames = std::env::var("SJ_CAPTURE_AFTER_FRAMES")
        .ok()?
        .parse()
        .ok()?;
    let path = std::env::var("SJ_CAPTURE_PATH")
        .map_or_else(|_| PathBuf::from("capture.ppm"), PathBuf::from);
    Some(CaptureRequest { after_frames, path })
}

pub fn encode_ppm(width_px: u32, height_px: u32, bottom_up_rgba: &[u8]) -> Vec<u8> {
    let mut ppm = format!("P6\n{width_px} {height_px}\n255\n").into_bytes();
    let row_bytes = width_px as usize * 4;
    for row in bottom_up_rgba.chunks_exact(row_bytes).rev() {
        for pixel in row.as_chunks::<4>().0 {
            ppm.extend_from_slice(&pixel[..3]);
        }
    }
    ppm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_header_then_rows_top_first_without_alpha() {
        let bottom_up_rgba = [1, 2, 3, 255, 4, 5, 6, 255];
        let ppm = encode_ppm(1, 2, &bottom_up_rgba);
        assert_eq!(ppm, b"P6\n1 2\n255\n\x04\x05\x06\x01\x02\x03");
    }
}
