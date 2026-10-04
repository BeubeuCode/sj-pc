use crate::addr::{Arm9Addr, MAIN_RAM_BASE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueWidth {
    U8,
    U16,
    U32,
}

impl ValueWidth {
    pub fn bytes(self) -> usize {
        match self {
            ValueWidth::U8 => 1,
            ValueWidth::U16 => 2,
            ValueWidth::U32 => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    Equals(u32),
    Changed,
    Unchanged,
    Increased,
    Decreased,
}

pub struct RamSearch {
    pub width: ValueWidth,
    candidates: Vec<u32>,
    previous_ram: Vec<u8>,
}

impl RamSearch {
    pub fn start(ram: &[u8], width: ValueWidth) -> Self {
        let step = width.bytes();
        let last_offset = ram.len().saturating_sub(step);
        let candidates = (0..=last_offset)
            .step_by(step)
            .map(|offset| offset as u32)
            .collect();
        Self {
            width,
            candidates,
            previous_ram: ram.to_vec(),
        }
    }

    pub fn refine(&mut self, ram: &[u8], filter: Filter) {
        let width = self.width;
        let previous_ram = &self.previous_ram;
        self.candidates.retain(|offset| {
            let now = value_at(ram, *offset, width);
            let before = value_at(previous_ram, *offset, width);
            keeps(filter, before, now)
        });
        self.previous_ram = ram.to_vec();
    }

    pub fn candidates(&self) -> &[u32] {
        &self.candidates
    }
}

pub fn value_at(ram: &[u8], offset: u32, width: ValueWidth) -> u32 {
    let start = offset as usize;
    let Some(bytes) = ram.get(start..start + width.bytes()) else {
        return 0;
    };
    bytes
        .iter()
        .rev()
        .fold(0, |value, byte| value << 8 | u32::from(*byte))
}

pub fn offset_to_addr(offset: u32) -> Arm9Addr {
    Arm9Addr(MAIN_RAM_BASE.0 + offset)
}

fn keeps(filter: Filter, before: u32, now: u32) -> bool {
    match filter {
        Filter::Equals(wanted) => now == wanted,
        Filter::Changed => now != before,
        Filter::Unchanged => now == before,
        Filter::Increased => now > before,
        Filter::Decreased => now < before,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_little_endian_values_of_each_width() {
        let ram = [0x34, 0x12, 0x78, 0x56];
        assert_eq!(value_at(&ram, 0, ValueWidth::U8), 0x34);
        assert_eq!(value_at(&ram, 0, ValueWidth::U16), 0x1234);
        assert_eq!(value_at(&ram, 0, ValueWidth::U32), 0x5678_1234);
        assert_eq!(value_at(&ram, 3, ValueWidth::U16), 0);
    }

    #[test]
    fn candidates_are_aligned_to_the_width() {
        let search = RamSearch::start(&[0; 8], ValueWidth::U16);
        assert_eq!(search.candidates(), [0, 2, 4, 6]);
    }

    #[test]
    fn hp_hunt_narrows_to_the_one_value_that_dropped() {
        let mut ram = vec![0u8; 16];
        ram[4] = 56;
        ram[10] = 56;
        let mut search = RamSearch::start(&ram, ValueWidth::U16);
        search.refine(&ram, Filter::Equals(56));
        assert_eq!(search.candidates(), [4, 10]);

        ram[4] = 41;
        search.refine(&ram, Filter::Decreased);
        assert_eq!(search.candidates(), [4]);
        assert_eq!(offset_to_addr(4).to_string(), "0x02000004");
    }

    #[test]
    fn changed_and_unchanged_compare_with_the_previous_refine() {
        let mut ram = vec![1u8, 2, 3, 4];
        let mut search = RamSearch::start(&ram, ValueWidth::U8);
        ram[1] = 9;
        search.refine(&ram, Filter::Unchanged);
        assert_eq!(search.candidates(), [0, 2, 3]);
        ram[2] = 0;
        search.refine(&ram, Filter::Changed);
        assert_eq!(search.candidates(), [2]);
    }
}
