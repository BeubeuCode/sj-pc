// In menus, facilities and the ship the game draws 3D on both screens with one 3D engine: every
// frame it sends one screen's geometry (picked by the parity at 0x021437DC) and its VBlank code
// (0x020523B8) flips the parity and points display capture at the other screen. Each screen gets
// 30fps although the game logic runs at 60. When we only show the top screen, two instructions
// keep the parity on the top screen, which then draws every frame; the bottom one stops updating.
// Battles and dungeons already draw 3D on the top screen only and never reach this code.

struct CodeWord {
    addr: u32,
    original: u32,
    patched: u32,
}

const PATCH: [CodeWord; 2] = [
    // beq to the top-screen setup (0x02051FB8) -> always branch there
    CodeWord {
        addr: 0x0205_23FC,
        original: 0x0A00_0001,
        patched: 0xEA00_0001,
    },
    // moveq r2, #1 (next parity: bottom) -> nop, so the parity stays 0 (top)
    CodeWord {
        addr: 0x0205_2428,
        original: 0x03A0_2001,
        patched: 0xE1A0_0000,
    },
];

// Code type 5 is "if the word equals"; type 0, a 32-bit write, is the bare address.
const AR_IF_EQUAL: u32 = 0x5000_0000;
const AR_END_IF: &str = "D0000000 00000000";

// The patch as an Action Replay code, or the code that puts the original instructions back.
// Code goes through the core's cheat engine because the JIT ignores host writes to code. Each
// write only happens over the exact instruction it replaces, so another ROM revision is left
// alone and an applied patch costs nothing per frame.
pub fn cheat_code(top_only: bool) -> String {
    PATCH
        .iter()
        .map(|word| {
            let (from, to) = if top_only {
                (word.original, word.patched)
            } else {
                (word.patched, word.original)
            };
            format!(
                "{:08X} {from:08X} {:08X} {to:08X} {AR_END_IF}",
                AR_IF_EQUAL | word.addr,
                word.addr
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_each_word_only_over_the_instruction_it_expects() {
        assert_eq!(
            cheat_code(true),
            "520523FC 0A000001 020523FC EA000001 D0000000 00000000 \
             52052428 03A02001 02052428 E1A00000 D0000000 00000000"
        );
        assert!(cheat_code(false).starts_with("520523FC EA000001 020523FC 0A000001"));
    }
}
