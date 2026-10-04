use std::fmt;

pub const MAIN_RAM_BASE: Arm9Addr = Arm9Addr(0x0200_0000);
pub const MAIN_RAM_SIZE_BYTES: u32 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arm9Addr(pub u32);

impl Arm9Addr {
    pub fn main_ram_offset(self, length_bytes: u32) -> Option<usize> {
        let offset = self.0.checked_sub(MAIN_RAM_BASE.0)?;
        let end = offset.checked_add(length_bytes)?;
        if end > MAIN_RAM_SIZE_BYTES {
            return None;
        }
        Some(offset as usize)
    }
}

impl fmt::Display for Arm9Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_byte_of_main_ram_is_offset_zero() {
        assert_eq!(MAIN_RAM_BASE.main_ram_offset(1), Some(0));
    }

    #[test]
    fn last_word_of_main_ram_fits() {
        let last_word = Arm9Addr(MAIN_RAM_BASE.0 + MAIN_RAM_SIZE_BYTES - 4);
        assert_eq!(
            last_word.main_ram_offset(4),
            Some(MAIN_RAM_SIZE_BYTES as usize - 4)
        );
    }

    #[test]
    fn range_crossing_end_of_main_ram_is_rejected() {
        let last_byte = Arm9Addr(MAIN_RAM_BASE.0 + MAIN_RAM_SIZE_BYTES - 1);
        assert_eq!(last_byte.main_ram_offset(2), None);
    }

    #[test]
    fn address_below_main_ram_is_rejected() {
        assert_eq!(Arm9Addr(0x01FF_FFFF).main_ram_offset(1), None);
    }

    #[test]
    fn displays_as_padded_hex() {
        assert_eq!(Arm9Addr(0x0200_0800).to_string(), "0x02000800");
    }
}
