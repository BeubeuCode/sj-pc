# Roadmap

Status legend: `done`, `in progress`, `todo`.

| Milestone | Status | Summary |
|---|---|---|
| Housekeeping | done | git repo, ROM moved to gitignored `roms/`, `CLAUDE.md`, docs, skill |
| M0 Tooling | done (Ghidra setup is manual, see `re/README.md`) | Rust, CMake, melonDS-ds submodule, core build script |
| M1 Boots and plays | done, needs a full play-test | libretro host, input, savestates with autosave and a menu bar, fast-forward, frame rate in the title |
| M2 Settings UI | half done | launcher screen done (egui); in-game F1 overlay still todo |
| One-screen UI (A) research tooling | done | `sj-lab` headless runner, F9 snapshots, F12 RAM search |
| One-screen UI (B) foundation | done | game-mode detection (dungeon from the running scene), screen director, minimap overlay, new bindings |
| One-screen UI (C) crop HUD | partly skipped | minimap is a crop of the game's map; battle went straight to native pages |
| One-screen UI (D) widescreen | done | melonDS patch: 3D drawn 4/3 wider, 2D centred; status bar and command menu pegged to the edges |
| One-screen UI (E) native HUD | in progress | battle pages done (enemies, party, summon list: race, affinities, skills); heading-up minimap todo |
| One-screen UI (G) 60fps | done where possible | battles and dungeons native 60; ship and facilities 60 when top-only; side-by-side menus stay 30 |
| One-screen UI (F) text entry | todo | name buffers found, encoding and pointer chain to map |
| One-screen menus | todo, biggest risk | Y menu, mission log and compendium split their content across both screens |
| M4 Native UI via decomp | todo | ARM9 hooks, native map/menus/dialog drawn at full resolution |
| M5 Modding | todo | asset redirect + Lua scripts over `GameApi` |

Rough progress (October 2026): a playable single-screen version is about 60 to 65% there; the full
plan including M4 and M5 about 25 to 30%.

## M1 checklist
- [x] `sj-game`: settings, layout math, touch mapping, input mapping, patch engine
- [x] `sj-emu`: load core, environment/video/audio/input callbacks, savestates, main RAM via `GameApi`
- [x] `sj`: SDL window, OpenGL presentation at 1-8x, controller hot-plug, audio-clocked pacing
- [x] savestates (10 slots), autosave every 5 minutes and on quit, fast-forward
- [x] menu bar at the top edge to load and save states; emulated and top-screen frame rate in the title
- [x] ROM boots to the intro screens (verified by capture and by the ROM smoke test)
- [ ] hands-on play-test: controller feel, touch accuracy, savestates mid-dungeon
- [ ] build and run on Linux and Windows

## M2 checklist
- [x] Launcher before the game: ROM check, drag-and-drop, display, touch overlay, sound, controls
- [x] Press-to-bind for DS buttons and hotkeys, stick deadzone, volume/mute, sharp/smooth scaling
- [x] Verified: the launcher renders, and Play hands off to the game (title screen captured)
- [ ] In-game F1 overlay for live changes

Deliberately not built yet: the RAM watch. It needs real addresses from M3 first.

## Dependencies between milestones
- M3 widescreen needs M1 rendering at arbitrary framebuffer width.
- M4 needs core fork extension #1 (ARM9 PC breakpoint callback).
- M5 asset redirect needs core fork extension #2 (card read redirect). Lua needs `GameApi` stable.

## Out of scope
HD textures, netplay, Redux (3DS) content, public release packaging, native plugin ABI.

## One-screen UI: what blocks what
- Native pages show name, level, HP, MP, race, affinities (only the non-normal ones, as text) and
  skills. The game's element icons would need its graphics loaded from the ROM at runtime. The hero's
  affinities come from gear and are not mapped yet.
- Battle details left: the hero's affinities (from gear), the battle results page, element icons.
- Heading-up minimap needs player position and facing (collision code at `0x0202FDD4` is the lead).
- Menus on one screen need each menu's two halves recomposed (native pages from RAM, or M4 hooks).
- 60fps: battles and dungeons already run at 60. The ship and facilities run at 60 while shown
  top-only (`top_screen_60fps`). Menus shown side by side stay at 30 per screen: the game draws one
  screen per frame, and lifting that means making it build both screens every frame.
