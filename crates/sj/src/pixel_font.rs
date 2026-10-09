// Our own glyphs, drawn to match the shapes of the game's battle UI fonts. Each row is a bit mask
// `cell_width` bits wide, left-aligned; a glyph is as wide as its rightmost lit column, so text is
// proportional like the game's.

const SPACE_WIDTH: u32 = 2;

pub struct Glyphs {
    pub cell_width: u32,
    lookup: fn(char) -> Option<&'static [u8]>,
}

impl Glyphs {
    pub fn rows(&self, ch: char) -> Option<&'static [u8]> {
        (self.lookup)(ch)
    }

    pub fn is_set(&self, row_bits: u8, column: u32) -> bool {
        row_bits & (1 << (self.cell_width - 1 - column)) != 0
    }

    pub fn glyph_width(&self, ch: char) -> u32 {
        let lit = self
            .rows(ch)
            .map_or(0, |rows| rows.iter().fold(0, |all, row| all | row));
        if lit == 0 {
            return SPACE_WIDTH;
        }
        self.cell_width - u32::from(lit).trailing_zeros()
    }

    pub fn text_width(&self, text: &str) -> u32 {
        let total: u32 = text.chars().map(|ch| self.glyph_width(ch) + 1).sum();
        total.saturating_sub(1)
    }
}

// Names: 7 rows, up to 5 wide, like the hero's name in the party panel.
pub const NAME: Glyphs = Glyphs {
    cell_width: 5,
    lookup: name_glyph,
};

// Stat labels: 5x5, like "HP" and "MP".
pub const LABEL: Glyphs = Glyphs {
    cell_width: 5,
    lookup: label_glyph,
};

// Stat values: 3x5, like the HP and MP numbers.
pub const DIGITS: Glyphs = Glyphs {
    cell_width: 3,
    lookup: digit_glyph,
};

const NAME_GLYPHS: [(char, [u8; 7]); 57] = [
    (
        'A',
        [
            0b01100, 0b10010, 0b10010, 0b11110, 0b10010, 0b10010, 0b10010,
        ],
    ),
    (
        'B',
        [
            0b11100, 0b10010, 0b10010, 0b11100, 0b10010, 0b10010, 0b11100,
        ],
    ),
    (
        'C',
        [
            0b01110, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01110,
        ],
    ),
    (
        'D',
        [
            0b11100, 0b10010, 0b10010, 0b10010, 0b10010, 0b10010, 0b11100,
        ],
    ),
    (
        'E',
        [
            0b11110, 0b10000, 0b10000, 0b11100, 0b10000, 0b10000, 0b11110,
        ],
    ),
    (
        'F',
        [
            0b11110, 0b10000, 0b10000, 0b11100, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'G',
        [
            0b01110, 0b10000, 0b10000, 0b10110, 0b10010, 0b10010, 0b01110,
        ],
    ),
    (
        'H',
        [
            0b10010, 0b10010, 0b10010, 0b11110, 0b10010, 0b10010, 0b10010,
        ],
    ),
    (
        'I',
        [
            0b11100, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b11100,
        ],
    ),
    (
        'J',
        [
            0b00110, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
    ),
    (
        'K',
        [
            0b10010, 0b10100, 0b11000, 0b11000, 0b10100, 0b10010, 0b10010,
        ],
    ),
    (
        'L',
        [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11110,
        ],
    ),
    (
        'M',
        [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
    ),
    (
        'N',
        [
            0b10010, 0b11010, 0b11010, 0b10110, 0b10110, 0b10010, 0b10010,
        ],
    ),
    (
        'O',
        [
            0b01100, 0b10010, 0b10010, 0b10010, 0b10010, 0b10010, 0b01100,
        ],
    ),
    (
        'P',
        [
            0b11100, 0b10010, 0b10010, 0b11100, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'Q',
        [
            0b01100, 0b10010, 0b10010, 0b10010, 0b10110, 0b10010, 0b01110,
        ],
    ),
    (
        'R',
        [
            0b11100, 0b10010, 0b10010, 0b11100, 0b10100, 0b10010, 0b10010,
        ],
    ),
    (
        'S',
        [
            0b01110, 0b10000, 0b10000, 0b01100, 0b00010, 0b00010, 0b11100,
        ],
    ),
    (
        'T',
        [
            0b11100, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000,
        ],
    ),
    (
        'U',
        [
            0b10010, 0b10010, 0b10010, 0b10010, 0b10010, 0b10010, 0b01100,
        ],
    ),
    (
        'V',
        [
            0b10010, 0b10010, 0b10010, 0b10010, 0b10010, 0b01100, 0b01100,
        ],
    ),
    (
        'W',
        [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
    ),
    (
        'X',
        [
            0b10010, 0b10010, 0b01100, 0b01100, 0b01100, 0b10010, 0b10010,
        ],
    ),
    (
        'Y',
        [
            0b10100, 0b10100, 0b10100, 0b01000, 0b01000, 0b01000, 0b01000,
        ],
    ),
    (
        'Z',
        [
            0b11110, 0b00010, 0b00100, 0b01100, 0b01000, 0b10000, 0b11110,
        ],
    ),
    (
        'a',
        [
            0b00000, 0b00000, 0b01100, 0b00100, 0b11100, 0b10100, 0b11100,
        ],
    ),
    (
        'b',
        [
            0b10000, 0b10000, 0b11100, 0b10100, 0b10100, 0b10100, 0b11100,
        ],
    ),
    (
        'c',
        [
            0b00000, 0b00000, 0b11100, 0b10000, 0b10000, 0b10000, 0b11100,
        ],
    ),
    (
        'd',
        [
            0b00100, 0b00100, 0b11100, 0b10100, 0b10100, 0b10100, 0b11100,
        ],
    ),
    (
        'e',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b11100, 0b10000, 0b11100,
        ],
    ),
    (
        'f',
        [
            0b01100, 0b10000, 0b11100, 0b10000, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'g',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b11100, 0b00100, 0b11100,
        ],
    ),
    (
        'h',
        [
            0b10000, 0b10000, 0b11100, 0b10100, 0b10100, 0b10100, 0b10100,
        ],
    ),
    (
        'i',
        [
            0b10000, 0b00000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'j',
        [
            0b01000, 0b00000, 0b01000, 0b01000, 0b01000, 0b01000, 0b11000,
        ],
    ),
    (
        'k',
        [
            0b10000, 0b10000, 0b10100, 0b10100, 0b11000, 0b10100, 0b10100,
        ],
    ),
    (
        'l',
        [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'm',
        [
            0b00000, 0b00000, 0b11110, 0b10101, 0b10101, 0b10101, 0b10101,
        ],
    ),
    (
        'n',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b10100, 0b10100, 0b10100,
        ],
    ),
    (
        'o',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b10100, 0b10100, 0b11100,
        ],
    ),
    (
        'p',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b11100, 0b10000, 0b10000,
        ],
    ),
    (
        'q',
        [
            0b00000, 0b00000, 0b11100, 0b10100, 0b11100, 0b00100, 0b00100,
        ],
    ),
    (
        'r',
        [
            0b00000, 0b00000, 0b10100, 0b11000, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        's',
        [
            0b00000, 0b00000, 0b11100, 0b10000, 0b11100, 0b00100, 0b11100,
        ],
    ),
    (
        't',
        [
            0b10000, 0b10000, 0b11100, 0b10000, 0b10000, 0b10000, 0b01100,
        ],
    ),
    (
        'u',
        [
            0b00000, 0b00000, 0b10100, 0b10100, 0b10100, 0b10100, 0b11100,
        ],
    ),
    (
        'v',
        [
            0b00000, 0b00000, 0b10100, 0b10100, 0b10100, 0b10100, 0b01000,
        ],
    ),
    (
        'w',
        [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010,
        ],
    ),
    (
        'x',
        [
            0b00000, 0b00000, 0b10100, 0b10100, 0b01000, 0b10100, 0b10100,
        ],
    ),
    (
        'y',
        [
            0b00000, 0b00000, 0b10100, 0b10100, 0b11100, 0b00100, 0b11100,
        ],
    ),
    (
        'z',
        [
            0b00000, 0b00000, 0b11100, 0b00100, 0b01000, 0b10000, 0b11100,
        ],
    ),
    (
        '-',
        [
            0b00000, 0b00000, 0b00000, 0b11100, 0b00000, 0b00000, 0b00000,
        ],
    ),
    (
        '.',
        [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b10000,
        ],
    ),
    (
        '\'',
        [
            0b10000, 0b10000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
    ),
    (
        '?',
        [
            0b01100, 0b10010, 0b00010, 0b00100, 0b01000, 0b00000, 0b01000,
        ],
    ),
    (
        ' ',
        [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
    ),
];

fn name_glyph(ch: char) -> Option<&'static [u8]> {
    NAME_GLYPHS
        .iter()
        .find(|(glyph, _)| *glyph == ch)
        .map(|(_, rows)| rows.as_slice())
}

fn label_glyph(ch: char) -> Option<&'static [u8]> {
    let rows: &'static [u8; 5] = match ch {
        'H' => &[0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        'L' => &[0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'V' => &[0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'M' => &[0b10001, 0b11011, 0b11111, 0b10101, 0b10001],
        'P' => &[0b11110, 0b10010, 0b11110, 0b10000, 0b10000],
        _ => return None,
    };
    Some(rows)
}

fn digit_glyph(ch: char) -> Option<&'static [u8]> {
    let rows: &'static [u8; 5] = match ch {
        '0' => &[0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => &[0b110, 0b010, 0b010, 0b010, 0b111],
        '2' => &[0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => &[0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => &[0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => &[0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => &[0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => &[0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => &[0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => &[0b111, 0b101, 0b111, 0b001, 0b111],
        '?' => &[0b111, 0b001, 0b010, 0b000, 0b010],
        _ => return None,
    };
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_fits_its_cell() {
        for (glyphs, height, chars) in [
            (
                &NAME,
                7,
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-.'? ",
            ),
            (&LABEL, 5, "HLMPV"),
            (&DIGITS, 5, "0123456789?"),
        ] {
            for ch in chars.chars() {
                let rows = glyphs.rows(ch).unwrap();
                assert_eq!(rows.len(), height, "{ch}");
                assert!(
                    rows.iter()
                        .all(|row| u32::from(*row) < 1 << glyphs.cell_width),
                    "{ch}"
                );
            }
        }
    }

    #[test]
    fn text_width_counts_one_pixel_between_glyphs() {
        assert_eq!(DIGITS.text_width("58"), 7);
        assert_eq!(NAME.text_width(""), 0);
    }

    #[test]
    fn narrow_letters_take_less_room() {
        assert_eq!(NAME.glyph_width('i'), 1);
        assert_eq!(NAME.glyph_width('M'), 5);
        assert_eq!(NAME.text_width("Pixie"), 4 + 1 + 1 + 1 + 3 + 1 + 1 + 1 + 3);
    }
}
