# Memory map

## From the ROM header (verified)
| What | Value |
|---|---|
| Game code | `BMTE` (USA), title `MGTNDSNA` |
| ARM9 load address | `0x02000000`, entry `0x02000800` |
| ARM9 size in ROM | 784,648 bytes |
| ARM7 load address | `0x02380000`, entry `0x02380000`, 160,048 bytes |
| ARM9 overlays | 2, both compressed |
| Overlay 0 | RAM `0x02176EA0`, 7,776 bytes + 32 BSS, file id 0 |
| Overlay 1 | RAM `0x02176EA0`, 8,256 bytes, file id 1 |
| File system | 4,972 files under `Data/` and `Snd/` |

Both overlays load at the same address, so only one is resident at a time.

## To confirm
- The ARM9 binary is probably BLZ-compressed too. The overlays load at `0x02176EA0`, which suggests the
  decompressed ARM9 occupies roughly `0x02000000`–`0x02176EA0` (about 1.5 MB). NTRGhidra will tell.

## DS hardware (fixed)
| Range | What |
|---|---|
| `0x02000000`–`0x023FFFFF` | Main RAM, 4 MB. This is what `GameApi::main_ram` exposes. |
| `0x04000000` | Main 2D engine registers |
| `0x04000400`+ | 3D geometry engine command ports |
| `0x04001000` | Sub 2D engine registers |
| `0x06000000`+ | VRAM banks |

## Game variables
Found with `sj-lab` snapshots (see `re/scripts/`) and public Action Replay codes for BMTE
(melonDS cheat file and Codejunkies list, see the research notes in `docs/re/README.md`).

| Address | Size | Meaning | How found | Confidence |
|---|---|---|---|---|
| `0x022142E8` | u8 ×5 | Player ST/VI/AG/LU/MA, base | public AR codes | high |
| `0x022142EE` | u16 ×2 | Player max HP, max MP | value search 56/32 + AR codes | high |
| `0x022142F2` | u8 ×5 | Player ST/VI/AG/LU/MA, effective | AR codes | high |
| `0x022142F8` | u16 ×2 | Player current HP, MP | value search + AR "Full MP" at `+2` | high |
| `0x022142FC` | u16 ×8 | Player first name, persistent copy | name typed as eight "1" = `0x0012` | high |
| `0x0221434C` | u8 | Alignment: `0x00` Law, `0x80` Neutral, `0xFF` Chaos | AR codes | high |
| `0x02214620` | u32 | Macca | AR codes | high |
| `0x0221463C` | u8 | Moon phase (`0x00` full, `0x28` half, `0x50` new) | AR codes | high |
| `0x0221468F` | u8 ×0x39 | Expendable item counts | AR codes | high |
| `0x022159F2 + n×0x24` | u16 | Compendium entry field, 491 entries | AR codes | medium |
| `0x0216B440` | ptr | Current top-level scene object; child scene at `+0x1C`. Battle scene = current (ship) or its child (dungeon) | 4 battle snapshots, no false hit in ~120 others | high |
| scene `+0x94` | ptr ×10 | Battle unit table: hero, allies 1-3, enemies 1-6 (empty slots point at zeroed units). Hero unit follows the table, so `table[0] == table + 0x28` identifies a battle scene | 4 battle snapshots | high |
| battle scene `+0x1C` → `+0x64` | u32 | Bottom-screen page: 1 enemy status, 2 party status, 3 Summon list, 5 battle results (L/R change it) | 9 battle snapshots, ship and dungeon | high |
| battle UI `+0x70` | u8 | Cursor in the bottom page's list (Summon list red frame), wraps | two-demon Summon list, 3 cursor positions | high |
| battle scene `+0x68` | u32 | 1 while summoning, 3 on results, else 0; not the page | 5 snapshots | low |
| current scene `+0x08` | ptr | Scene update function: `0x0203034C` dungeon (exploring, battles and menus there), `0x020B6608` title, `0x02124538` facility. Tells the dungeon from the title (both scene flags `0x0000`) | all 160 lab and play snapshots | high |
| `0x0215F03C` | struct | Render state. `+0x24` display mode (0 3D alternates between screens, 1 top only, 2 bottom only, 3 none; set by `0x02051DD0`), `+0x33` frame rendered, `+0x38` main loop counter (every frame, 30fps scenes included) | ymenu, facility, battle, dungeon snapshots; per-frame RAM dumps | high |
| `0x021437DC` | u32 | Screen parity in mode 0: 0 the frame sends top-screen geometry, 1 bottom. Flipped by the VBlank code `0x020523B8`, which also calls `0x02051FB8` (top) or `0x020520A0` (bottom) to set the screen swap bit and display capture | swap/capture register log per frame | high |
| `0x0216B44C` | u32 | `0x01` on the ship and title; in dungeons a value that climbs over time (0x11 to 0x24 seen). Not a dungeon bit: an early guess read bit `0x10`, which failed at `0x24` | lab and play snapshots | low |
| `0x0216B9AC` | ptr | Message window object, non-null while a dialogue box is open | ship + dungeon dialogue, 35 snapshots, one mislabel | high |
| unit | struct | Battle unit, stride `0x2E0`: HP `+0x2C`, max HP `+0x30`, MP `+0x38`, max MP `+0x3C` (u32), level `+0x44` (u16), record ptr `+0x28` (demon: name table entry; hero: save record starting with the 16-bit name), skills `+0x1B8` (6 × u16 skill ID; an enemy's current moves, the hero's gun skill), demon ID `+0x1D8` (u16, 0 for the hero), flags `+0x282` (bit `0x04`: a demon never analyzed, shown as "??? UNKNOWN"; `0x020EAD78` swaps the name on it) | battle snapshots, AR codes | high |
| `0x02214350 + n×0x3C` | struct ×12 | Demon stock: max HP `+0x0A`, max MP `+0x0C`, HP `+0x1E`, MP `+0x20`, skills `+0x28` (6 × u16 skill ID), demon ID `+0x36`, level `+0x38`, slot index `+0x3A` (kept in empty slots) | Pixie record + empty slots in a real save | high |
| `0x021E6CE0` | MTBL | Demon name table (491 entries, `NKMBaseData.mbb`), loaded in every scene; name = entry ID + 2 | 6 enemy units + stock | high |
| `0x021E6860` | MTBL | Race names (56 entries, first block of `NKMBaseData.mbb`); name = entry race ID + 2 | Fairy, Jirae, Foul, Spirit match the game's cards | high |
| `0x021CA430 + id×100` | struct ×491 | Demon base records (first table of `NKMBaseTable.tbb`, file at `0x021CA400`), loaded in every scene: ID `+0x00`, race `+0x02` (u8), base level `+0x03`, innate skills `+0x2E` (6 × u16), affinities `+0x3A` (8 × u16: Phys Gun Fire Ice Elec Force Expel Curse; kind = value >> 10 in the order -, Nu, Wk, Rf, Dr, St, low bits = damage %), ailment affinities follow | Pixie St Fire, Knocker Wk Fire / St Ice on the game's party and summon pages | high |
| `0x021D6410 + id×128` | struct ×490 | Second table of `NKMBaseTable.tbb` (file `+0xC000`), also resident. `+0x5A` and `+0x5B` (u8) are Analyze gauge thresholds; the status card shows affinities and skills once the gauge reaches `+0x5B` (`0x020D9970`). Pixie 10/44, Slime 10/40, a boss 0/100 | 4 battles against the game's cards | high |
| `0x022159D4 + (id−1)×36` | struct ×491 | Per-demon save records (save block `0x022142D8 + 0x16FC`; `0x0202E774` returns the block). Analyze gauge = `+0x1E` & `0x7F`, 0 to 100 (`0x0202EBB8` reads, `0x0202EB48` writes, clamped) | Slime 0→21→26, Pixie 10→48→70, Knocker 0→10→45 across play-* and two-demon snapshots | high |
| `0x0214283C + n×16` | struct ×49 | Loaded file table: `+0x04` data pointer, `+0x08` loading flag. `0x0204459C` returns file n, `0x020445D0`/`0x020445F4` a TBL block of it. 10 `SkillStrData.mbb`, 18 a name table, 19 `NKMBaseTable.tbb` | code | high |
| `0x021428E0` | ptr | Loaded `SkillStrData.mbb` (battle and menus only, null elsewhere; the file moves). Skill names are its first MTBL block at `+0x20`, name = entry skill ID + 2 | Agi 1, Bufu 10, Dia 101, Fire Shot 201 | high |
| `0x02229514` | — | Unit table address in public AR codes. Right for dungeon battles, wrong for the ship's first battle (`0x0222BE34`) | AR codes | partial |
| `0x0216AB60` | u16 | Scene flags, one bit per top-level scene, see `game_mode.rs` | diff of all lab samples | high |
| `0x02222700` | u16 ×8 | First-name edit buffer during Name Entry (heap) | diff during Name Entry | high |
| `0x02222680` | u16 ×8 | Last-name edit buffer during Name Entry (heap) | same | high |

Character encoding: ASCII minus `0x1F` (digit "1" is `0x12`, "B" `0x23`, space `0x01`). Save data and
edit buffers use 16-bit codes; loaded string tables (demon names, `Data/Enemy/NKMBaseData.mbb`) use
one byte per character between `0xFF 0xFF` and a `0x00`/`0xFE` end. Decoder: `sj_game::text`.

## Screen usage observed (DS version)
| Situation | Top screen | Bottom screen |
|---|---|---|
| Title | logo | NEW GAME / CONFIG |
| Name Entry | name fields | character grid, START ends |
| Ship facility (Command Room, Deck) | room view + command list (Talk, Move, Disembark) | automap of the ship |
| Dialogue | portrait + text box | automap |
| Y menu (DEMONICA) | detail page (e.g. full status) | menu list + party table (HP/MP) |
| X Mission Log | document text | document list |

## Code addresses from public cheat codes
| ARM9 address | What | Use for us |
|---|---|---|
| `0x0202FDD4` | Wall collision test; r6/r7 are tile coordinates, maps up to 0x40×0x33 | player position (hook) |
| `0x02036610` | Encounter gauge update, s16 at `[r4+0xAF2]` | encounter gauge HUD |
| `0x020586D0` | Perspective setter (fovy sin, cos, aspect in `r2`, near, far on stack); caches args at `0x02168C24` | widescreen research, see `WIDESCREEN.md` |
| `0x0211BB48` | Battle EXP award | — |
| `0x02044AD4` | Macca award | — |
