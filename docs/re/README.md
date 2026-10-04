# Reverse engineering

## Ground rules
- Work from your own ROM in `roms/`. Never commit extracted files, disassembly listings or Ghidra databases.
- What we commit: addresses, names, struct layouts and prose. `docs/re/symbols.txt` is the source of truth.

## Tools
- **Ghidra** with the **NTRGhidra** loader, which understands `.nds`, the ARM9/ARM7 split and overlays.
- **ndstool** (devkitPro) to list and extract the file system: `ndstool -x roms/sj_usa.nds -9 arm9.bin -d data/`.
  Extract into a gitignored folder such as `re/ghidra/`.
- **melonDS debugger builds** or our own frontend plus `GameApi` for live memory reads.

## Ghidra setup
1. New project in `re/ghidra/` (gitignored).
2. Import `roms/sj_usa.nds` with NTRGhidra. It decompresses the ARM9 and overlays (they are BLZ-compressed).
3. Language: `ARM:LE:32:v5t`. The ARM9 is an ARM946E-S. Mixed ARM and Thumb code is normal.
4. Run auto-analysis, then apply known Nitro SDK function names as you find them.

## Nitro SDK conventions worth knowing
- Calling convention is AAPCS: arguments in `r0`–`r3`, return in `r0`, `lr` holds the return address.
- Fixed-point is everywhere. `fx32` is 20.12, `fx16` is 4.12. `FX32_ONE = 0x1000`.
- 3D goes through `G3_*` and `G3X_*` calls that write the geometry engine registers at `0x04000400`+.
  `G3_Perspective` / `G3i_PerspectiveW_` builds the projection, whose aspect argument is the widescreen target.
- 2D engine registers: main engine at `0x04000000`, sub engine at `0x04001000`.

## Finding a game-state variable (memory diff recipe)
1. Save a state in situation A (walking in a dungeon). Dump main RAM through `GameApi`.
2. Enter situation B (a battle). Dump again. Return to A. Dump again.
3. Keep addresses whose value is equal in both A dumps and different in B.
4. Repeat with C (menu), D (dialog) to narrow down. A small integer that takes a distinct value per mode
   is usually the mode variable.
5. Confirm with a Ghidra cross-reference: who writes it, and from which state-machine function.
6. Record it in `symbols.txt` and `MEMORY_MAP.md`.

## Files
- `MEMORY_MAP.md`: what lives where.
- `WIDESCREEN.md`: the projection patch plan.
- `symbols.txt`: one line per known symbol.
