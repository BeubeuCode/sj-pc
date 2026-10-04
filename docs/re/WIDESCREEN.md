# Widescreen (M3 plan)

## Goal
The dungeon and battle 3D fill a 16:9 window. 2D UI stays 4:3 and centred, never stretched.

## Plan
1. Find where the game builds its 3D projection. Search Ghidra for writes to the geometry engine's
   projection matrix (`MTX_MODE` = projection, then `MTX_LOAD_4x4` or `MTX_MULT_4x4`), or for the
   Nitro SDK `G3i_PerspectiveW_` helper and its callers.
2. Identify the aspect ratio argument. For 4:3 it is `FX32(4/3) = 0x1555` in 20.12 fixed point.
3. Patch it to `FX32(16/9) = 0x1C71` at boot with an `sj_game::patch::Patch`. The patch checks the
   original bytes first, so a different ROM revision fails loudly instead of corrupting code.
4. Make the core render a wider 3D framebuffer, which needs a melonDS-ds fork change, and present it
   with `layout::screen_rects` using a 16:9 top aspect.

## Risks
- The game may compute the matrix in several places (dungeon, battle, cutscenes). Patch each, one by one.
- Culling may still clip objects at the old 4:3 edges. That needs a second patch in the visibility code.
- 2D layers composited over the 3D will need repositioning or pillarboxing.

## Fallback
If the aspect is not a clean argument, patch the instruction that writes the projection matrix
column scale instead.
