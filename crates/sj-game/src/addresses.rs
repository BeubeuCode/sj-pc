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
