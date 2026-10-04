// The game's text is ASCII shifted down by 0x1F ("1" is 0x12, "B" is 0x23). Strings in loaded
// tables are one byte per character between a 0xFF 0xFF start mark and a 0x00 or 0xFE end.
const CHAR_OFFSET: u8 = 0x1F;
const START_MARK: [u8; 2] = [0xFF, 0xFF];
const LAST_PRINTABLE_CODE: u8 = b'~' - CHAR_OFFSET;

pub fn decode_table_string(bytes: &[u8]) -> Option<String> {
    let body = bytes.strip_prefix(&START_MARK).unwrap_or(bytes);
    let text: String = body
        .iter()
        .take_while(|&&code| code != 0x00 && code != 0xFE)
        .map(|&code| decode_char(code))
        .collect();
    (!text.is_empty()).then_some(text)
}

fn decode_char(code: u8) -> char {
    if (1..=LAST_PRINTABLE_CODE).contains(&code) {
        char::from(code + CHAR_OFFSET)
    } else {
        '?'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_demon_names_from_the_loaded_name_table() {
        let pixie = [0xFF, 0xFF, 0x31, 0x4A, 0x59, 0x4A, 0x46, 0x00, 0xFE, 0xFF];
        assert_eq!(decode_table_string(&pixie).as_deref(), Some("Pixie"));
        let pyro_jack = [
            0xFF, 0xFF, 0x31, 0x5A, 0x53, 0x50, 0x01, 0x2B, 0x42, 0x44, 0x4C, 0xFE,
        ];
        assert_eq!(
            decode_table_string(&pyro_jack).as_deref(),
            Some("Pyro Jack")
        );
    }

    #[test]
    fn empty_or_unprintable_strings() {
        assert_eq!(decode_table_string(&[0xFF, 0xFF, 0xFE, 0xFF]), None);
        assert_eq!(decode_table_string(&[0xF0]).as_deref(), Some("?"));
    }
}
