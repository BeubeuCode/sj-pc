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
| `0x0216AB60` | u16 | Scene flags, one bit per top-level scene, see `game_mode.rs` | diff of all lab samples | high |
| `0x02222700` | u16 ×8 | First-name edit buffer during Name Entry (heap) | diff during Name Entry | high |
| `0x02222680` | u16 ×8 | Last-name edit buffer during Name Entry (heap) | same | high |
| — | — | dungeon camera projection call site | Phase D target, see `WIDESCREEN.md` | — |

Character encoding: 16-bit codes, digit "1" is `0x0012`. Full table still to map.

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
| `0x0211BB48` | Battle EXP award | — |
| `0x02044AD4` | Macca award | — |
