# Roadmap

Status legend: `done`, `in progress`, `todo`.

| Milestone | Status | Summary |
|---|---|---|
| Housekeeping | done | git repo, ROM moved to gitignored `roms/`, `CLAUDE.md`, docs, skill |
| M0 Tooling | done (Ghidra setup is manual, see `re/README.md`) | Rust, CMake, melonDS-ds submodule, core build script |
| M1 Boots and plays | done, needs a play-test pass | libretro host, top screen + bottom PiP, input, savestates, fast-forward |
| M2 Settings UI | half done | launcher screen done (egui); in-game F1 overlay still todo |
| One-screen UI (A) research tooling | done | `sj-lab` headless runner, F9 snapshots, F12 RAM search |
| One-screen UI (B) foundation | done | game-mode detection (battle included), screen director, minimap overlay, new bindings |
| One-screen UI (C) crop HUD | skipped for battle | battle went straight to native cards; minimap waits on a dungeon sample |
| One-screen UI (D) widescreen | todo | 16:9 dungeon camera |
| One-screen UI (E) native HUD | in progress | enemy panels with names and HP/MP done; party bars, heading-up minimap |
| One-screen UI (F) text entry | todo | name buffers found, encoding and pointer chain to map |
| M4 Native UI via decomp | todo | ARM9 hooks, native map/menus/dialog drawn at full resolution |
| M5 Modding | todo | asset redirect + Lua scripts over `GameApi` |

## M1 checklist
- [x] `sj-game`: settings, layout math, touch mapping, input mapping, patch engine
- [x] `sj-emu`: load core, environment/video/audio/input callbacks, savestates, main RAM via `GameApi`
- [x] `sj`: SDL window, OpenGL presentation at 1-8x, controller hot-plug, audio-clocked pacing
- [x] savestates (10 slots), autosave every 5 minutes and on quit, fast-forward
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
- The summon-list flag (battle scene `+0x68`) rests on one snapshot. Item and skill lists probably
  need the same treatment: snapshots of each confirm it.
- Heading-up minimap needs player position and facing (collision code at `0x0202FDD4` is the lead).
