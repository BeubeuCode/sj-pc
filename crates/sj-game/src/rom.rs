use std::io::Read;
use std::path::Path;

const GAME_CODE_OFFSET: usize = 0x0C;
const GAME_CODE_LENGTH: usize = 4;
pub const STRANGE_JOURNEY_USA_CODE: &str = "BMTE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomStatus {
    StrangeJourneyUsa,
    OtherGame { game_code: String },
    NotAnNdsFile,
    Missing,
    Unreadable(String),
}

pub fn identify_header(header: &[u8]) -> RomStatus {
    let Some(code_bytes) = header.get(GAME_CODE_OFFSET..GAME_CODE_OFFSET + GAME_CODE_LENGTH) else {
        return RomStatus::NotAnNdsFile;
    };
    if !code_bytes.iter().all(u8::is_ascii_alphanumeric) {
        return RomStatus::NotAnNdsFile;
    }
    let game_code = String::from_utf8_lossy(code_bytes).into_owned();
    if game_code == STRANGE_JOURNEY_USA_CODE {
        return RomStatus::StrangeJourneyUsa;
    }
    RomStatus::OtherGame { game_code }
}

pub fn identify_file(path: &Path) -> RomStatus {
    let mut header = Vec::with_capacity(GAME_CODE_OFFSET + GAME_CODE_LENGTH);
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return RomStatus::Missing,
        Err(error) => return RomStatus::Unreadable(error.to_string()),
    };
    let header_length = (GAME_CODE_OFFSET + GAME_CODE_LENGTH) as u64;
    if let Err(error) = file.take(header_length).read_to_end(&mut header) {
        return RomStatus::Unreadable(error.to_string());
    }
    identify_header(&header)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with_code(code: [u8; 4]) -> Vec<u8> {
        let mut header = b"MGTNDSNA\0\0\0\0".to_vec();
        header.extend_from_slice(&code);
        header
    }

    #[test]
    fn recognises_strange_journey_usa() {
        assert_eq!(
            identify_header(&header_with_code(*b"BMTE")),
            RomStatus::StrangeJourneyUsa
        );
    }

    #[test]
    fn reports_other_game_code() {
        let status = identify_header(&header_with_code(*b"BMTJ"));
        assert_eq!(
            status,
            RomStatus::OtherGame {
                game_code: "BMTJ".into()
            }
        );
    }

    #[test]
    fn short_or_binary_header_is_not_an_nds_file() {
        assert_eq!(identify_header(b"tiny"), RomStatus::NotAnNdsFile);
        assert_eq!(
            identify_header(&header_with_code([0, 1, 2, 3])),
            RomStatus::NotAnNdsFile
        );
    }

    #[test]
    fn missing_file_is_reported_as_missing() {
        assert_eq!(
            identify_file(Path::new("/definitely/not/here.nds")),
            RomStatus::Missing
        );
    }
}
