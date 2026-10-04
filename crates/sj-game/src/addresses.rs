use crate::addr::Arm9Addr;

// One bit per active top-level scene. Found by diffing lab snapshots of every mode
// (docs/re/MEMORY_MAP.md); identical in a software-rendered lab run and an OpenGL app run.
pub const SCENE_FLAGS: Arm9Addr = Arm9Addr(0x0216_AB60);

// The current scene's update function (code, so fixed for this ROM) is this one in a dungeon
// (Schwarzwelt sector), battles and menus there included. It tells dungeon exploration apart from
// the title screen, which share scene flags 0x0000. The title's is 0x020B6608.
pub const SCENE_UPDATE_FN_OFFSET: u32 = 0x08;
pub const DUNGEON_SCENE_UPDATE_FN: u32 = 0x0203_034C;

// Render state. Display mode: 0 the 3D alternates between the screens (menus, facilities, the ship;
// 30fps each), 1 top screen only (battles, dungeons), 2 bottom only, 3 none; set by 0x02051DD0.
// The main loop counter advances once per game frame, so it stalls when the game lags.
pub const RENDER_STATE: Arm9Addr = Arm9Addr(0x0215_F03C);
pub const RENDER_DISPLAY_MODE_OFFSET: u32 = 0x24;
pub const RENDER_MAIN_LOOP_COUNTER_OFFSET: u32 = 0x38;
pub const DISPLAY_MODE_ALTERNATE: u32 = 0;
pub const DISPLAY_MODE_TOP_ONLY: u32 = 1;

// Message window object: non-null while a dialogue box is on screen (ship and dungeon alike).
pub const MESSAGE_WINDOW: Arm9Addr = Arm9Addr(0x0216_B9AC);

// Demon stock: 12 records of 0x3C bytes right after the player record. Empty records keep only
// their slot index; a record is in use when its demon ID is non-zero.
pub const DEMON_STOCK: Arm9Addr = Arm9Addr(0x0221_4350);
pub const DEMON_STOCK_RECORD_SIZE: u32 = 0x3C;
pub const DEMON_STOCK_SLOTS: u32 = 12;
pub const STOCK_MAX_HP_OFFSET: u32 = 0x0A;
pub const STOCK_MAX_MP_OFFSET: u32 = 0x0C;
pub const STOCK_HP_OFFSET: u32 = 0x1E;
pub const STOCK_MP_OFFSET: u32 = 0x20;
pub const STOCK_DEMON_ID_OFFSET: u32 = 0x36;
pub const STOCK_LEVEL_OFFSET: u32 = 0x38;
// Learned skills: six u16 skill IDs, 0 when empty (Pixie: Agi, Dia).
pub const STOCK_SKILLS_OFFSET: u32 = 0x28;

// String tables (MTBL blocks: magic, header size, byte size, entry count, then one offset per
// entry from the block start) number their entries from 2, so an ID's string is entry ID + 2.
// Demon names and race names come from Data/Enemy/NKMBaseData.mbb, loaded for the whole game.
pub const TABLE_ID_OFFSET: u32 = 2;
pub const DEMON_NAME_TABLE: Arm9Addr = Arm9Addr(0x021E_6CE0);
pub const RACE_NAME_TABLE: Arm9Addr = Arm9Addr(0x021E_6860);
// Skill names (Data/Skill/SkillStrData.mbb) are loaded only in battle and in the menus, at an
// address that changes; this pointer holds the loaded file, whose first block is the names.
pub const SKILL_STRINGS_FILE: Arm9Addr = Arm9Addr(0x0214_28E0);
pub const SKILL_NAME_BLOCK_OFFSET: u32 = 0x20;
pub const SKILL_SLOTS: u32 = 6;

// Demon base records (first table of Data/Enemy/NKMBaseTable.tbb, loaded for the whole game),
// 100 bytes per demon ID. Race is a race-table ID. Affinities are eight u16, Phys, Gun, Fire, Ice,
// Elec, Force, Expel, Curse: the top six bits are the kind the status card shows, the rest a damage
// percentage. Checked against the game's cards for Pixie (St Fire) and Knocker (Wk Fire, St Ice).
pub const DEMON_BASE_TABLE: Arm9Addr = Arm9Addr(0x021C_A430);
pub const DEMON_BASE_TABLE_ENTRIES: u32 = 491;
pub const DEMON_BASE_RECORD_SIZE: u32 = 100;
pub const BASE_RACE_OFFSET: u32 = 0x02;
pub const BASE_AFFINITIES_OFFSET: u32 = 0x3A;

// Player record in the save block. Max values and stats are confirmed by public Action Replay
// codes; current HP/MP sit right before the name.
pub const PLAYER_MAX_HP: Arm9Addr = Arm9Addr(0x0221_42EE);
pub const PLAYER_MAX_MP: Arm9Addr = Arm9Addr(0x0221_42F0);
pub const PLAYER_HP: Arm9Addr = Arm9Addr(0x0221_42F8);
pub const PLAYER_MP: Arm9Addr = Arm9Addr(0x0221_42FA);
pub const PLAYER_FIRST_NAME: Arm9Addr = Arm9Addr(0x0221_42FC);

// Points at the current top-level scene object; each scene points at its child at +0x1C. The battle
// scene is the current one on the ship and its child in a dungeon. Its unit table (hero, three ally
// slots, six enemy slots) is followed by the hero's unit, so table[0] == table + 0x28 identifies it.
// The table moves between battles, which is why the fixed address in public cheat codes
// (0x02229514) only matched dungeon battles. See docs/re/MEMORY_MAP.md.
pub const CURRENT_SCENE: Arm9Addr = Arm9Addr(0x0216_B440);
pub const SCENE_CHILD_OFFSET: u32 = 0x1C;
pub const BATTLE_SCENE_UNIT_TABLE_OFFSET: u32 = 0x94;
pub const BATTLE_TABLE_HERO_UNIT_OFFSET: u32 = 0x28;
// The battle scene's child (at SCENE_CHILD_OFFSET) runs the bottom screen; this field is the page
// it shows: 1 enemy status, 2 party status, 3 summon list, 5 battle results. L and R change it.
pub const BATTLE_UI_PAGE_OFFSET: u32 = 0x64;
pub const BATTLE_PAGE_ENEMY_STATUS: u32 = 1;
pub const BATTLE_PAGE_PARTY_STATUS: u32 = 2;
pub const BATTLE_PAGE_SUMMON_LIST: u32 = 3;
// Cursor in the page's list (u8), wrapping like the game's red frame; checked moving through a
// two-demon Summon list.
pub const BATTLE_UI_LIST_CURSOR_OFFSET: u32 = 0x70;
pub const BATTLE_PARTY_SLOT_COUNT: u32 = 4;
pub const BATTLE_FIRST_ENEMY_SLOT: u32 = 4;
pub const BATTLE_ENEMY_SLOT_COUNT: u32 = 6;
pub const BATTLE_UNIT_HP_OFFSET: u32 = 0x2C;
pub const BATTLE_UNIT_MAX_HP_OFFSET: u32 = 0x30;
pub const BATTLE_UNIT_MP_OFFSET: u32 = 0x38;
pub const BATTLE_UNIT_MAX_MP_OFFSET: u32 = 0x3C;
pub const BATTLE_UNIT_LEVEL_OFFSET: u32 = 0x44;
// Points at the unit's record, which starts with its name: a demon's entry in the loaded name
// table ("Pixie", "Slime", "Unknown"), or the hero's save record (16-bit text).
pub const BATTLE_UNIT_NAME_OFFSET: u32 = 0x28;
// Skills the unit can use in this battle (u16 skill IDs, 0 when empty): the hero's gun skill, an
// enemy's current moves (a Lv2 Pixie enemy only knows Agi).
pub const BATTLE_UNIT_SKILLS_OFFSET: u32 = 0x1B8;
// u16 demon ID; 0 for the hero.
pub const BATTLE_UNIT_DEMON_ID_OFFSET: u32 = 0x1D8;
