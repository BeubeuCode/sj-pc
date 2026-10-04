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

## Game variables (none found yet)
| Address | Size | Meaning | How found |
|---|---|---|---|
| — | — | game mode (explore, battle, menu, dialog, map) | M3 target |
| — | — | dungeon camera projection call site | M3 target, see `WIDESCREEN.md` |
