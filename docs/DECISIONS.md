# Decisions

Short records. Newest at the bottom. Each one says what, why, and what would change our mind.

## 001 Hybrid strategy: emulator frontend now, decomp later
A full matching decomp of a 785 KB ARM9 binary is years of solo work with nothing playable in
between. Embedding an accurate core gives a playable build in weeks. Reverse-engineered code then
replaces original functions one hook at a time, so the game is always playable.
Revisit if a community decomp of Strange Journey appears.

## 002 Rust over C++
The code we will write most is code that pokes emulator memory. Rust confines that danger to one
module. Cargo gives identical build, test, format and lint commands on all three OSes. Newtypes make
unit mistakes (address vs offset, pixels vs scale) compile errors.
Cost: melonDS is C++ with no C API, so we go through libretro.

## 003 libretro over direct embedding
libretro is a stable C ABI, so Rust talks to it with plain FFI. melonDS-ds is the maintained
libretro port of melonDS and already exposes OpenGL rendering, scale factor, screen layouts and
savestates. We fork it only for what libretro cannot express: ARM9 PC hooks and file redirects.

## 004 Bottom screen as picture-in-picture until native UI exists
The target is a native one-screen UI (M4). Until then the bottom screen is a movable overlay.
We deliberately skip compositing individual 2D layers: it is real work that M4 throws away.

## 005 Lua for mods, no native plugin ABI
Lua (via `mlua`) is safe, hot-reloadable and needs no toolchain for modders. A native plugin ABI
would freeze our internals. Revisit if someone needs to reimplement a whole game system as a mod.

## 006 Game state from RAM, not from video heuristics
Reading the game's own mode variables is robust and doubles as the first reverse-engineering
task. Video heuristics (which layers are on) break on edge cases.

## 007 The core is built by a script, not by build.rs
The melonDS-ds build takes minutes and pulls its own dependencies. Running it from `build.rs` would
slow every clean Rust build and tie Rust builds to C++ toolchain issues. `scripts/build-core.sh`
builds it once, and `sj` loads it at runtime from the path in `sj.toml`.

## 008 GL calls are the second place `unsafe` is allowed
Every `glow` call is `unsafe`. Wrapping them in a third crate would add indirection for nothing.
GL `unsafe` lives in `crates/sj/src/present.rs` plus the one loader line in `app.rs`.
Emulator-memory `unsafe` stays in `crates/sj-emu/src/ffi/`.

## 009 No shaders in the presenter
Two `glBlitFramebuffer` calls do scaling, flipping and placement. Add shaders only when we need filters
(CRT, sharp bilinear) or rounded PiP corners.

## 010 egui instead of Dear ImGui for the launcher and future in-game menu
The Rust imgui crates pin SDL2 0.37 and glow 0.14, which would mean downgrading both. `egui-sdl2`
targets our SDL2 0.38 exactly. egui is pure Rust, so there is no C++ to compile, and its form widgets
fit a settings screen. Cost: glow is pinned to 0.17 to match `egui_glow`.

## 011 The launcher is the game's first screen, not a separate program
One executable and one window. Play hands the same window and GL context to the game. Returning to
the launcher from the game is deliberately not supported. In-game changes will come with the F1 menu.

## 012 ROM is picked by path field and drag-and-drop
No native file dialog, because that would need GTK or desktop-portal libraries on Linux.
Revisit if friends find typing paths painful.

## 013 On macOS, Xbox controllers use Apple's GameController driver
The bundled SDL (2.26) reads Bluetooth Xbox controllers through its own HID driver, which falls behind
Microsoft firmware updates. A Series controller on firmware 5.23 connected but delivered no input when
rebinding. `Platform::new` sets `SDL_JOYSTICK_HIDAPI_XBOX=0` on macOS, so SDL uses Apple's framework
instead. Apple tracks the firmware and no Input Monitoring permission is needed. Linux and Windows are
unchanged. Revisit when we move to a newer SDL.

## 017 Widescreen is done in the emulator, not by patching the game
Eight call sites build the game's perspective projections, once per scene, and the whole top-screen
interface (status bar, party panel, menus, text boxes, enemy sprites) is 3D too. Patching the game's
aspect would have meant one patch per scene and a stretched interface. Instead our melonDS patch squeezes
every rendered vertex's clip-space x by 3/4 and draws the 3D into a 4/3 wider target. Perspective scenes
gain field of view at their original scale, and 3D interface pieces land centred at their original size.
Box, position and vector tests keep the real matrices, so game logic is unaffected. The 2D layers stay
4:3 in the middle of the wider composite. The game does not cull the wider view in dungeons or battles.
Only the regular OpenGL renderer and the Top/Bottom layout support it; the software renderer stays 4:3.
Scenes that are pre-rendered 2D images (ship rooms) stay 4:3 with black sides.
The status bar's moon-phase and SEARCH/ANALYZE blocks are pegged to the screen edges by the same patch,
by position, since this fork only ever runs Strange Journey.

## 014 Menus show both screens side by side, not the bottom screen alone
The plan said menus would put the bottom screen in front. Lab captures showed the Y menu, the mission
log and name entry split their content: list on the bottom, details on the top. Showing only one
screen would hide half the menu, so these modes show both screens side by side until native menus exist.

## 015 HUD overlays are drawn by egui from the game framebuffer
The framebuffer's colour texture is registered with egui. Overlays are textured rectangles with crop
UVs and an alpha tint. This avoids a hand-written shader pipeline, and native HUD widgets later sit
in the same egui pass.

## 016 Research runs headless and is scripted
`sj-lab` plays input scripts at roughly 8x real time and snapshots RAM and screens. Mode detection was
found by diffing about 90 snapshots. Public Action Replay codes supplied player, battle-unit and item
addresses.

## 017 Code patches that switch at runtime go through the cheat engine
melonDS's JIT keeps compiled blocks when the host writes main RAM, so a patch written through
`GameApi` only works if the block was not compiled yet (right after a state load). Writes from the
core's Action Replay engine go through the emulated bus, which drops compiled blocks. The
top-screen 60fps patch must turn on and off as the layout changes, so it is an Action Replay code
set with `retro_cheat_set`, guarded by "if equal" checks so it only touches the expected
instructions. A `sj_game::patch::Patch` stays the tool for patches applied once.

