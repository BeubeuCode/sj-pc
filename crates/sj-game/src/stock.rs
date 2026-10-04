use crate::addr::Arm9Addr;
use crate::addresses::{
    DEMON_STOCK, DEMON_STOCK_RECORD_SIZE, DEMON_STOCK_SLOTS, STOCK_DEMON_ID_OFFSET,
    STOCK_HP_OFFSET, STOCK_LEVEL_OFFSET, STOCK_MAX_HP_OFFSET, STOCK_MAX_MP_OFFSET, STOCK_MP_OFFSET,
    STOCK_SKILLS_OFFSET,
};
use crate::battle::UnitStatus;
use crate::demon_data::{affinities, demon_name, race_name, skill_names};
use crate::game_api::{read_u16, GameApi};

// The demons in stock, in slot order, as battle-panel rows.
pub fn read_stock(game: &dyn GameApi) -> Vec<UnitStatus> {
    (0..DEMON_STOCK_SLOTS)
        .filter_map(|slot| read_record(game, slot))
        .collect()
}

fn read_record(game: &dyn GameApi, slot: u32) -> Option<UnitStatus> {
    let record = DEMON_STOCK.0 + slot * DEMON_STOCK_RECORD_SIZE;
    let field = |offset: u32| read_u16(game, Arm9Addr(record + offset));
    let id = field(STOCK_DEMON_ID_OFFSET)?;
    if id == 0 {
        return None;
    }
    Some(UnitStatus {
        slot,
        name: demon_name(game, id),
        race: race_name(game, id),
        level: field(STOCK_LEVEL_OFFSET)?,
        hp: u32::from(field(STOCK_HP_OFFSET)?),
        max_hp: u32::from(field(STOCK_MAX_HP_OFFSET)?),
        mp: u32::from(field(STOCK_MP_OFFSET)?),
        max_mp: u32::from(field(STOCK_MAX_MP_OFFSET)?),
        affinities: affinities(game, id),
        skills: skill_names(game, Arm9Addr(record + STOCK_SKILLS_OFFSET)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addresses::DEMON_NAME_TABLE;
    use crate::demon_data::tests::{put, put_table_string};
    use crate::game_api::FakeGame;

    fn put_u16(game: &mut FakeGame, addr: u32, value: u16) {
        put(game, addr, &value.to_le_bytes());
    }

    #[test]
    fn reads_pixie_from_the_first_stock_record() {
        let mut game = FakeGame::default();
        put_table_string(&mut game, DEMON_NAME_TABLE.0, 146, "Pixie");
        let record = DEMON_STOCK.0;
        put_u16(&mut game, record + STOCK_DEMON_ID_OFFSET, 146);
        put_u16(&mut game, record + STOCK_LEVEL_OFFSET, 2);
        put_u16(&mut game, record + STOCK_HP_OFFSET, 46);
        put_u16(&mut game, record + STOCK_MAX_HP_OFFSET, 46);
        put_u16(&mut game, record + STOCK_MP_OFFSET, 25);
        put_u16(&mut game, record + STOCK_MAX_MP_OFFSET, 25);
        let stock = read_stock(&game);
        assert_eq!(stock.len(), 1);
        assert_eq!(
            (
                stock[0].name.as_deref(),
                stock[0].level,
                stock[0].hp,
                stock[0].mp
            ),
            (Some("Pixie"), 2, 46, 25)
        );
    }

    #[test]
    fn empty_records_and_a_missing_name_table_are_skipped() {
        let mut game = FakeGame::default();
        assert_eq!(read_stock(&game), []);
        assert_eq!(demon_name(&game, 146), None);
        put_u16(&mut game, DEMON_STOCK.0 + STOCK_DEMON_ID_OFFSET, 146);
        assert_eq!(read_stock(&game)[0].name, None);
    }
}
