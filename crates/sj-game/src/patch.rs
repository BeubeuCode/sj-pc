use crate::addr::Arm9Addr;
use crate::game_api::GameApi;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub name: &'static str,
    pub addr: Arm9Addr,
    pub original: &'static [u8],
    pub replacement: &'static [u8],
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PatchError {
    #[error("patch {name}: original and replacement lengths differ")]
    LengthMismatch { name: &'static str },
    #[error("patch {name}: {addr} is outside main RAM")]
    OutOfRange { name: &'static str, addr: Arm9Addr },
    #[error("patch {name}: bytes at {addr} are not the expected original, wrong ROM revision?")]
    UnexpectedBytes { name: &'static str, addr: Arm9Addr },
}

pub fn apply(game: &mut dyn GameApi, patch: &Patch) -> Result<(), PatchError> {
    let range = checked_range(game, patch)?;
    game.main_ram_mut()[range].copy_from_slice(patch.replacement);
    Ok(())
}

fn checked_range(game: &dyn GameApi, patch: &Patch) -> Result<std::ops::Range<usize>, PatchError> {
    let name = patch.name;
    let addr = patch.addr;
    if patch.original.len() != patch.replacement.len() {
        return Err(PatchError::LengthMismatch { name });
    }
    let length = patch.original.len();
    let offset = addr
        .main_ram_offset(length as u32)
        .ok_or(PatchError::OutOfRange { name, addr })?;
    let range = offset..offset + length;
    if game.main_ram()[range.clone()] != *patch.original {
        return Err(PatchError::UnexpectedBytes { name, addr });
    }
    Ok(range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::MAIN_RAM_BASE;
    use crate::game_api::FakeGame;

    const AT_0X100: Arm9Addr = Arm9Addr(MAIN_RAM_BASE.0 + 0x100);

    fn patch(original: &'static [u8], replacement: &'static [u8]) -> Patch {
        Patch {
            name: "test",
            addr: AT_0X100,
            original,
            replacement,
        }
    }

    #[test]
    fn writes_replacement_when_original_matches() {
        let mut game = FakeGame::default();
        game.ram[0x100..0x102].copy_from_slice(&[0xAA, 0xBB]);
        assert_eq!(
            apply(&mut game, &patch(&[0xAA, 0xBB], &[0x11, 0x22])),
            Ok(())
        );
        assert_eq!(game.ram[0x100..0x102], [0x11, 0x22]);
    }

    #[test]
    fn refuses_and_leaves_ram_untouched_when_original_differs() {
        let mut game = FakeGame::default();
        let result = apply(&mut game, &patch(&[0xAA, 0xBB], &[0x11, 0x22]));
        assert_eq!(
            result,
            Err(PatchError::UnexpectedBytes {
                name: "test",
                addr: AT_0X100
            })
        );
        assert_eq!(game.ram[0x100..0x102], [0, 0]);
    }

    #[test]
    fn refuses_length_mismatch() {
        let result = apply(&mut FakeGame::default(), &patch(&[0], &[1, 2]));
        assert_eq!(result, Err(PatchError::LengthMismatch { name: "test" }));
    }

    #[test]
    fn refuses_address_outside_main_ram() {
        let outside = Patch {
            addr: Arm9Addr(0x0100_0000),
            ..patch(&[0], &[1])
        };
        assert!(matches!(
            apply(&mut FakeGame::default(), &outside),
            Err(PatchError::OutOfRange { .. })
        ));
    }
}
