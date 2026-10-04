use crate::image::RgbImage;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
const MAX_STORED_BLOCK_BYTES: usize = 65_535;
const COLOR_TYPE_RGB: u8 = 2;

// Uncompressed ("stored") deflate keeps the encoder dependency-free. Files are larger than a
// compressed PNG, which is fine for research snapshots.
pub fn encode(image: &RgbImage) -> Vec<u8> {
    let mut png = SIGNATURE.to_vec();
    png.extend(chunk(*b"IHDR", &header(image)));
    png.extend(chunk(*b"IDAT", &zlib_stored(&scanlines(image))));
    png.extend(chunk(*b"IEND", &[]));
    png
}

fn header(image: &RgbImage) -> Vec<u8> {
    let mut header = Vec::with_capacity(13);
    header.extend(image.width_px.to_be_bytes());
    header.extend(image.height_px.to_be_bytes());
    header.extend([8, COLOR_TYPE_RGB, 0, 0, 0]);
    header
}

fn scanlines(image: &RgbImage) -> Vec<u8> {
    let row_bytes = image.width_px as usize * 3;
    image
        .rgb
        .chunks_exact(row_bytes)
        .flat_map(|row| std::iter::once(0).chain(row.iter().copied()))
        .collect()
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = data.chunks(MAX_STORED_BLOCK_BYTES).collect();
    for (index, block) in blocks.iter().enumerate() {
        let is_last = index + 1 == blocks.len();
        let length = block.len() as u16;
        zlib.push(u8::from(is_last));
        zlib.extend(length.to_le_bytes());
        zlib.extend((!length).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    if blocks.is_empty() {
        zlib.extend([1, 0, 0, 0xFF, 0xFF]);
    }
    zlib.extend(adler32(data).to_be_bytes());
    zlib
}

fn chunk(kind: [u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
    chunk.extend_from_slice(&kind);
    chunk.extend_from_slice(data);
    let crc = crc32(&chunk[4..]);
    chunk.extend(crc.to_be_bytes());
    chunk
}

fn adler32(data: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let (low, high) = data.iter().fold((1u32, 0u32), |(low, high), byte| {
        let low = (low + u32::from(*byte)) % MODULUS;
        (low, (high + low) % MODULUS)
    });
    high << 16 | low
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_match_known_values() {
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn png_has_signature_header_and_end_chunk() {
        let png = encode(&RgbImage {
            width_px: 2,
            height_px: 1,
            rgb: vec![255, 0, 0, 0, 255, 0],
        });
        assert_eq!(png[..8], SIGNATURE);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(png[16..24], [0, 0, 0, 2, 0, 0, 0, 1]);
        assert_eq!(
            png[png.len() - 12..],
            [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]
        );
    }

    #[test]
    fn large_images_split_into_several_stored_blocks() {
        let data = vec![7u8; MAX_STORED_BLOCK_BYTES + 10];
        let zlib = zlib_stored(&data);
        assert_eq!(zlib[2], 0);
        let second_block_start = 2 + 5 + MAX_STORED_BLOCK_BYTES;
        assert_eq!(zlib[second_block_start], 1);
        assert_eq!(
            zlib[second_block_start + 1..second_block_start + 3],
            [10, 0]
        );
    }
}
