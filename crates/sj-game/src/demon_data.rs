use crate::addr::Arm9Addr;
use crate::addresses::{
    ANALYZE_DETAILS_THRESHOLD_OFFSET, ANALYZE_GAUGE_MASK, BASE_AFFINITIES_OFFSET, BASE_RACE_OFFSET,
    DEMON_ANALYZE_RECORD_SIZE, DEMON_ANALYZE_TABLE, DEMON_ANALYZE_TABLE_ENTRIES,
    DEMON_BASE_RECORD_SIZE, DEMON_BASE_TABLE, DEMON_BASE_TABLE_ENTRIES, DEMON_NAME_TABLE,
    DEMON_SAVE_RECORDS, DEMON_SAVE_RECORD_SIZE, RACE_NAME_TABLE, SAVE_ANALYZE_GAUGE_OFFSET,
    SKILL_NAME_BLOCK_OFFSET, SKILL_SLOTS, SKILL_STRINGS_FILE, TABLE_ID_OFFSET,
};
use crate::game_api::{read_u16, read_u32, read_u8, GameApi};
use crate::text::decode_table_string;

const MTBL_MAGIC: u32 = u32::from_le_bytes(*b"MTBL");
const MTBL_ENTRY_COUNT_OFFSET: u32 = 12;
const MTBL_HEADER_BYTES: u32 = 16;
const STRING_MAX_BYTES: u32 = 24;
const AFFINITY_KIND_SHIFT: u16 = 10;

pub const ELEMENT_COUNT: usize = 8;
pub const ELEMENT_NAMES: [&str; ELEMENT_COUNT] = [
    "Phys", "Gun", "Fire", "Ice", "Elec", "Force", "Expel", "Curse",
];

// In the order of the game's labels ("-", "Nu", "Wk", "Rf", "Dr", "St"), which is also the
// order of the kind stored in the affinity's top bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Affinity {
    #[default]
    Normal,
    Null,
    Weak,
    Repel,
    Drain,
    Strong,
}

impl Affinity {
    pub const LABELS: [(Affinity, &str); 5] = [
        (Affinity::Weak, "Wk"),
        (Affinity::Strong, "St"),
        (Affinity::Null, "Nu"),
        (Affinity::Repel, "Rf"),
        (Affinity::Drain, "Dr"),
    ];

    pub fn from_raw(raw: u16) -> Option<Affinity> {
        match raw >> AFFINITY_KIND_SHIFT {
            0 => Some(Affinity::Normal),
            1 => Some(Affinity::Null),
            2 => Some(Affinity::Weak),
            3 => Some(Affinity::Repel),
            4 => Some(Affinity::Drain),
            5 => Some(Affinity::Strong),
            _ => None,
        }
    }
}

pub fn demon_name(game: &dyn GameApi, id: u16) -> Option<String> {
    table_string(game, DEMON_NAME_TABLE, u32::from(id))
}

pub fn race_name(game: &dyn GameApi, id: u16) -> Option<String> {
    let race = game
        .main_ram()
        .get(Arm9Addr(base_record(id)? + BASE_RACE_OFFSET).main_ram_offset(1)?)?;
    table_string(game, RACE_NAME_TABLE, u32::from(*race))
}

pub fn affinities(game: &dyn GameApi, id: u16) -> Option<[Affinity; ELEMENT_COUNT]> {
    let first = base_record(id)? + BASE_AFFINITIES_OFFSET;
    let mut affinities = [Affinity::Normal; ELEMENT_COUNT];
    for (element, affinity) in (0u32..).zip(affinities.iter_mut()) {
        *affinity = Affinity::from_raw(read_u16(game, Arm9Addr(first + 2 * element))?)?;
    }
    Some(affinities)
}

// The names of the non-empty skill IDs in the six u16 slots at `slots`.
pub fn skill_names(game: &dyn GameApi, slots: Arm9Addr) -> Vec<String> {
    let Some(table) = read_u32(game, SKILL_STRINGS_FILE) else {
        return Vec::new();
    };
    let names = Arm9Addr(table.wrapping_add(SKILL_NAME_BLOCK_OFFSET));
    (0..SKILL_SLOTS)
        .filter_map(|slot| read_u16(game, Arm9Addr(slots.0 + 2 * slot)))
        .filter(|&skill| skill != 0)
        .filter_map(|skill| table_string(game, names, u32::from(skill)))
        .collect()
}

// Whether the player has analyzed this demon far enough for the game to show its affinities and
// skills on the enemy status card.
pub fn details_analyzed(game: &dyn GameApi, id: u16) -> bool {
    let id = u32::from(id);
    if id == 0 || id >= DEMON_ANALYZE_TABLE_ENTRIES {
        return false;
    }
    let gauge =
        DEMON_SAVE_RECORDS.0 + (id - 1) * DEMON_SAVE_RECORD_SIZE + SAVE_ANALYZE_GAUGE_OFFSET;
    let threshold =
        DEMON_ANALYZE_TABLE.0 + id * DEMON_ANALYZE_RECORD_SIZE + ANALYZE_DETAILS_THRESHOLD_OFFSET;
    match (
        read_u8(game, Arm9Addr(gauge)),
        read_u8(game, Arm9Addr(threshold)),
    ) {
        (Some(gauge), Some(threshold)) => gauge & ANALYZE_GAUGE_MASK >= threshold,
        _ => false,
    }
}

fn base_record(id: u16) -> Option<u32> {
    let id = u32::from(id);
    (id < DEMON_BASE_TABLE_ENTRIES).then(|| DEMON_BASE_TABLE.0 + id * DEMON_BASE_RECORD_SIZE)
}

fn table_string(game: &dyn GameApi, table: Arm9Addr, id: u32) -> Option<String> {
    if read_u32(game, table)? != MTBL_MAGIC {
        return None;
    }
    let entry = id + TABLE_ID_OFFSET;
    if entry >= read_u32(game, Arm9Addr(table.0 + MTBL_ENTRY_COUNT_OFFSET))? {
        return None;
    }
    let offset = read_u32(game, Arm9Addr(table.0 + MTBL_HEADER_BYTES + 4 * entry))?;
    let start = Arm9Addr(table.0.wrapping_add(offset)).main_ram_offset(STRING_MAX_BYTES)?;
    decode_table_string(
        game.main_ram()
            .get(start..start + STRING_MAX_BYTES as usize)?,
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::addr::MAIN_RAM_BASE;
    use crate::game_api::FakeGame;

    pub fn put(game: &mut FakeGame, addr: u32, bytes: &[u8]) {
        let offset = (addr - MAIN_RAM_BASE.0) as usize;
        game.ram[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    // An MTBL block at `table` holding `text` (plain ASCII) as the string of `id`.
    pub fn put_table_string(game: &mut FakeGame, table: u32, id: u32, text: &str) {
        let entry = id + TABLE_ID_OFFSET;
        put(game, table, b"MTBL");
        put(
            game,
            table + MTBL_ENTRY_COUNT_OFFSET,
            &0x200u32.to_le_bytes(),
        );
        let string_offset = 0x400 + 0x20 * id;
        put(
            game,
            table + MTBL_HEADER_BYTES + 4 * entry,
            &string_offset.to_le_bytes(),
        );
        let mut encoded = vec![0xFF, 0xFF];
        encoded.extend(text.bytes().map(|byte| byte - 0x1F));
        encoded.push(0xFE);
        put(game, table + string_offset, &encoded);
    }

    #[test]
    fn affinity_kind_is_the_top_bits() {
        assert_eq!(Affinity::from_raw(0x0064), Some(Affinity::Normal));
        assert_eq!(Affinity::from_raw(0x1432), Some(Affinity::Strong));
        assert_eq!(Affinity::from_raw(0x087D), Some(Affinity::Weak));
        assert_eq!(Affinity::from_raw(0x0401), Some(Affinity::Null));
        assert_eq!(Affinity::from_raw(0x0C7D), Some(Affinity::Repel));
        assert_eq!(Affinity::from_raw(0x1064), Some(Affinity::Drain));
        assert_eq!(Affinity::from_raw(0xFFFF), None);
    }

    #[test]
    fn reads_race_and_affinities_from_the_base_record() {
        let mut game = FakeGame::default();
        let knocker = DEMON_BASE_TABLE.0 + 167 * DEMON_BASE_RECORD_SIZE;
        put(&mut game, knocker + BASE_RACE_OFFSET, &[18]);
        put_table_string(&mut game, RACE_NAME_TABLE.0, 18, "Jirae");
        let raw: [u16; 8] = [0x64, 0x64, 0x086E, 0x1432, 0x64, 0x64, 0x64, 0x64];
        let bytes: Vec<u8> = raw.iter().flat_map(|value| value.to_le_bytes()).collect();
        put(&mut game, knocker + BASE_AFFINITIES_OFFSET, &bytes);
        assert_eq!(race_name(&game, 167).as_deref(), Some("Jirae"));
        let affinities = affinities(&game, 167).unwrap();
        assert_eq!(affinities[2], Affinity::Weak);
        assert_eq!(affinities[3], Affinity::Strong);
        assert_eq!(affinities[0], Affinity::Normal);
        assert_eq!(race_name(&game, 999), None);
    }

    #[test]
    fn details_need_the_gauge_to_reach_the_demons_threshold() {
        let mut game = FakeGame::default();
        let pixie = 146;
        let gauge =
            DEMON_SAVE_RECORDS.0 + (pixie - 1) * DEMON_SAVE_RECORD_SIZE + SAVE_ANALYZE_GAUGE_OFFSET;
        let threshold = DEMON_ANALYZE_TABLE.0
            + pixie * DEMON_ANALYZE_RECORD_SIZE
            + ANALYZE_DETAILS_THRESHOLD_OFFSET;
        put(&mut game, threshold, &[44]);
        put(&mut game, gauge, &[10]);
        assert!(!details_analyzed(&game, 146));
        put(&mut game, gauge, &[0x80 + 70]);
        assert!(
            details_analyzed(&game, 146),
            "the top bit is not part of the gauge"
        );
        assert!(!details_analyzed(&game, 0));
    }

    #[test]
    fn skill_names_follow_the_loaded_skill_file() {
        let mut game = FakeGame::default();
        let slots = 0x0221_4378;
        put(&mut game, slots, &[1, 0, 0x65, 0, 0, 0]);
        assert_eq!(skill_names(&game, Arm9Addr(slots)), Vec::<String>::new());
        let file: u32 = 0x0223_4040;
        put(&mut game, SKILL_STRINGS_FILE.0, &file.to_le_bytes());
        put_table_string(&mut game, file + SKILL_NAME_BLOCK_OFFSET, 1, "Agi");
        put_table_string(&mut game, file + SKILL_NAME_BLOCK_OFFSET, 0x65, "Dia");
        assert_eq!(skill_names(&game, Arm9Addr(slots)), ["Agi", "Dia"]);
    }
}
