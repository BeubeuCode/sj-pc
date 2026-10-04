use crate::addr::Arm9Addr;

pub trait GameApi {
    fn main_ram(&self) -> &[u8];
    fn main_ram_mut(&mut self) -> &mut [u8];
}

pub fn read_u16(game: &dyn GameApi, addr: Arm9Addr) -> Option<u16> {
    let offset = addr.main_ram_offset(2)?;
    let bytes = game.main_ram().get(offset..offset + 2)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

pub fn read_u32(game: &dyn GameApi, addr: Arm9Addr) -> Option<u32> {
    let offset = addr.main_ram_offset(4)?;
    let bytes = game.main_ram().get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
pub struct FakeGame {
    pub ram: Vec<u8>,
}

#[cfg(test)]
impl Default for FakeGame {
    fn default() -> Self {
        Self {
            ram: vec![0; crate::addr::MAIN_RAM_SIZE_BYTES as usize],
        }
    }
}

#[cfg(test)]
impl GameApi for FakeGame {
    fn main_ram(&self) -> &[u8] {
        &self.ram
    }

    fn main_ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::MAIN_RAM_BASE;

    #[test]
    fn reads_little_endian_word() {
        let mut game = FakeGame::default();
        game.ram[0x10..0x14].copy_from_slice(&[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(
            read_u32(&game, Arm9Addr(MAIN_RAM_BASE.0 + 0x10)),
            Some(0x1234_5678)
        );
    }

    #[test]
    fn reading_outside_main_ram_gives_none() {
        assert_eq!(read_u32(&FakeGame::default(), Arm9Addr(0x0100_0000)), None);
    }
}
