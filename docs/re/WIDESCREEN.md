# Widescreen

## Goal
The dungeon and battle 3D fill a 16:9 window. 2D UI stays 4:3 and centred, never stretched.

## What we found in the game
- The projection helper is `0x020586D0` (NitroSystem-style: caches fovy sin/cos, aspect, near, far at
  `0x02168C24`, builds the matrix with `0x02001E48`). Eight call sites pass aspect `0x1555` (4:3) in `r2`
  from their own literal pools: `0x02037D40`, `0x020A426C`, `0x020B7824`, `0x020DD2F4`, `0x020EE170`,
  `0x020F3BCC`, `0x020F7108`, `0x021218BC`.
- The projection is built once when a scene is set up, so patching the literals in a running scene does
  nothing until the scene is rebuilt, and closing the Y menu does not rebuild the dungeon camera.
- The whole top-screen interface is drawn with the 3D engine: status bar, party panel, command menu,
  text boxes and enemy sprites. Any aspect change on the game side would stretch them.
- With the view widened, dungeon and battle geometry keeps going past the old edges: no culling problem.
- Ship rooms are pre-rendered: the 3D room is captured to VRAM once and shown as a 2D bitmap.

## What we built
The patch in `patches/melonDS/` (see `docs/DECISIONS.md` 017):
1. `GPU3D::SubmitVertex` multiplies clip-space x by 3/4 when `WideScreen` is set. Box, position and
   vector tests are untouched.
2. The OpenGL 3D renderer draws into a target 4/3 wider and scales vertex x to match.
3. The 2D compositor outputs 4/3 wider: the 3D layer spans the whole width, BG and OBJ layers sit in
   the middle 256 columns, the sides show the 3D layer or the backdrop, and window state is taken from
   the nearest edge column.
4. The final pass centres VRAM display modes, and display capture reads the middle 256 columns.
   `GLRenderer::Reset` clears the configs, so the widescreen values are re-applied after it.
5. `GPU3D::PegWideScreenHud` moves the status bar's corner blocks out to the edges of the wide picture:
   flat (equal-w, orthographic) polygons that start in the top 4 rows, end within row 26 and lie
   within x 0-75 (moon phase) or 181-256 (SEARCH/ANALYZE) shift by a quarter of w, which is exactly the
   side margin. The whole bar is 3D, including its hatched frame. The battle command menu (COMMANDS
   header, rows and the submenu beside them) moves to the left edge and 12 rows down, clear of the moon
   block, while its header is on screen:
   flat polygons in front of the camera plane (z < 0), in rows 15-101, no taller than one 15-pixel row,
   starting left of x 194. Battle sprites are flat too (every flat polygon has w 409600) and some are cut
   into strips as short as a menu row (Pixie's legs), but they sit at z > 0 (128000-156000) while the
   interface sits at z -140000 to -216000. Bounds come from logging the polygons of dungeon and battle
   frames.

Engine B's 2D renderer does not compile its own shaders: `InitShaders(other)` copies engine A's programs
and uniform locations, so every new compositor uniform location must be copied there too.

melonDS DS (`patches/melonDS-ds/`) adds the `melonds_widescreen` option, stretches the top screen over a
4/3 wider buffer and centres the bottom screen's 256 columns under it, so touch maths stays in DS pixels.

## Known limits
- Regular OpenGL renderer and Top/Bottom layout only. The compute renderer and the software renderer
  (used by `sj-lab`) stay 4:3.
- 3D horizontal resolution is the internal scale spread over 4/3 more width (about 1365 pixels at 4x).
- Effects that read the screen back through display capture see only the middle 4:3.
