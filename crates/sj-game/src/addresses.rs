use crate::addr::Arm9Addr;

// One bit per active top-level scene. Found by diffing lab snapshots of every mode
// (docs/re/MEMORY_MAP.md); identical in a software-rendered lab run and an OpenGL app run.
pub const SCENE_FLAGS: Arm9Addr = Arm9Addr(0x0216_AB60);

// Bit 0x10 is set while the player is in a dungeon (Schwarzwelt sector), battles there included.
// It tells dungeon exploration apart from the title screen, which share scene flags 0x0000.
pub const AREA_FLAGS: Arm9Addr = Arm9Addr(0x0216_B44C);
pub const AREA_FLAG_DUNGEON: u32 = 0x10;

// Message window object: non-null while a dialogue box is on screen (ship and dungeon alike).
pub const MESSAGE_WINDOW: Arm9Addr = Arm9Addr(0x0216_B9AC);

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
// Non-zero while a list the game draws on the bottom screen is open (seen: Summon). One sample so
// far; a wrong guess only shows the bottom screen when it is not needed.
pub const BATTLE_SCENE_BOTTOM_MENU_OFFSET: u32 = 0x68;
pub const BATTLE_FIRST_ENEMY_SLOT: u32 = 4;
pub const BATTLE_ENEMY_SLOT_COUNT: u32 = 6;
pub const BATTLE_UNIT_HP_OFFSET: u32 = 0x2C;
pub const BATTLE_UNIT_MAX_HP_OFFSET: u32 = 0x30;
pub const BATTLE_UNIT_MP_OFFSET: u32 = 0x38;
pub const BATTLE_UNIT_MAX_MP_OFFSET: u32 = 0x3C;
// Points at the demon's entry in the loaded name table ("Pixie", "Slime", "Unknown").
pub const BATTLE_UNIT_NAME_OFFSET: u32 = 0x28;
