# Architecture

```
            sj (binary)                         sj-emu                      melonDS-ds core
 ┌──────────────────────────────┐   ┌────────────────────────────┐   ┌──────────────────────┐
 │ App: SDL window, GL context, │   │ Core: safe wrapper         │   │ melondsds_libretro   │
 │ events, audio queue, pacing  │──▶│ ffi/host: C callbacks,     │──▶│ .dylib / .so / .dll  │
 │ Presenter: FBO + 2 blits     │   │ thread-local HostState     │◀──│ (loaded at runtime)  │
 │ HostInput, savestate slots   │   └────────────────────────────┘   └──────────────────────┘
 └──────────────┬───────────────┘                 │
                │ uses                            │ implements GameApi
                ▼                                 ▼
          sj-game (pure): settings, input mapping, layout math, touch mapping,
                          Arm9Addr, GameApi trait, RAM patches
```

## Crates
| Crate | Job | Allowed to |
|---|---|---|
| `sj-game` | Everything we know about the game and the screen, as pure functions | Nothing platform-specific. No SDL, GL, FFI or `unsafe`. |
| `sj-emu` | Load the libretro core and expose it safely | `unsafe`, only inside `src/ffi/` |
| `sj` | Window, GL, input devices, audio, hotkeys | `unsafe` only for GL calls (`present.rs`, GL loader in `app.rs`) |

## Startup
1. `main.rs` loads `sj.toml` and creates the `Platform`: SDL, one window, one GL context, controllers.
2. Unless `--play` is given, the launcher (`crates/sj/src/launcher/`) runs in that window with egui.
   It edits a `Settings` value, saves it on Play, then hands back the `Platform` and the settings.
3. `App::new` applies the video settings to the same window, loads the core and runs the game.

The launcher is split by concern:
- `mod.rs`: the loop and event routing.
- `state.rs`: what the screen holds, Play/Save/Reset/Quit.
- `binding_capture.rs`: press-to-bind logic, pure and tested.
- `*_section.rs`: one file per settings section.

Input names (`"X"`, `"pad:a"`, `"pad:leftx-"`) come from `host_input.rs` for both the launcher's
capture and the game, so a binding captured in the launcher always matches in game.

## Research tooling
- `sj-lab` (dev binary) runs the core headless in software mode and plays scripts from `re/scripts/`:
  `wait`, `press`, `hold`, `mash`, `snapshot`, `save`, `load`.
- Snapshots (`re/samples/<label>-<n>/`, gitignored) hold the savestate, main RAM and both screens as PNG.
  F9 in the app writes the same layout.
- F12 in the app opens the RAM search panel (value, changed, unchanged, increased, decreased, watch list).
- `cargo run -- --play --state <file>` starts from a savestate. The state is applied after the
  first frame because the core resets the console during it.

## Frame loop (`crates/sj/src/app.rs`)
1. **Events.** SDL events update `HostInput` (pressed key and pad names), the mouse position and hotkeys.
2. **Emulate.** The joypad mask comes from `sj_game::input::button_mask` and the touch pointer from
   `sj_game::touch`. `Core::run_frame` calls `retro_run`. Fast-forward runs 4 frames per present and drops audio.
3. **Present.** `Presenter::draw` clears the window and blits the top half of the core's frame into the
   letterboxed main rectangle, and the bottom half into the picture-in-picture rectangle.
4. **Throttle.** Sleep while the SDL audio queue holds more than ~4 frames of audio. Audio is the clock.
   Vsync, when on, only limits tearing.

## Video path
The core is pinned to its `top-bottom` layout with no gap (`crates/sj/src/core_options.rs`), so every
frame is one surface with the top screen above the touch screen. Our layout code splits it.

- **OpenGL (default).** The core asks for a GL core context through `SET_HW_RENDER`. We hand it our
  framebuffer object id via `get_current_framebuffer`, then call its `context_reset`. It renders at
  `video.scale` times native resolution straight into our FBO. Rows are stored bottom-up.
- **Software (fallback).** The core hands over XRGB8888 pixels. We upload them into the same FBO's
  colour texture. Rows are stored top-down. `present::source_rows` handles both orders.

There are no shaders. Two `glBlitFramebuffer` calls do the scaling and placement, with linear or
nearest filtering from `video.filter`. The presenter disables the scissor test first, because egui
leaves it enabled after the launcher.

## Touch path
Mouse position (window points) → drawable pixels (Retina-aware) → inside the PiP? →
bottom-screen pixel (`touch::mouse_to_bottom_screen`) → libretro pointer coordinates over the whole
256x384 surface (`touch::bottom_screen_to_pointer`). The core turns that back into a DS touch.

## libretro callbacks we implement (`crates/sj-emu/src/ffi/host.rs`)
| Environment command | What we do |
|---|---|
| `GET_CAN_DUPE` | yes |
| `SET_PIXEL_FORMAT` | accept XRGB8888 only |
| `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY` | `save_dir` from settings |
| `SET_HW_RENDER` | accept OpenGL / OpenGL core, fill in our framebuffer and proc-address callbacks |
| `GET_VARIABLE`, `GET_VARIABLE_UPDATE` | serve our pinned options, flag changes |
| `SET_SYSTEM_AV_INFO`, `SET_GEOMETRY` | record new sizes; the presenter grows its FBO |
| `GET_CORE_OPTIONS_VERSION` | 0, so the core uses plain `SET_VARIABLES`, which we ignore |
| `GET_PREFERRED_HW_RENDER` | OpenGL core |
| `GET_INPUT_BITMASKS` | yes; `input_state` answers the whole joypad mask in one call |
| anything else | `false`, and the core falls back to its defaults |

The log interface is not provided because Rust cannot define C-variadic functions on stable.
The core prints to stderr instead.

## Threading
Single-threaded. libretro callbacks have no user pointer, so they reach a thread-local `HostState`.
Only one `Core` may exist per process, which `Core::load` enforces.

## GameApi
`sj_game::game_api::GameApi` exposes main RAM (4 MB at `0x02000000`). `Core` implements it.
RAM patches, the future RAM watch and future Lua mods all go through it. Nothing else touches emulator
memory directly.

## Debug capture
`SJ_CAPTURE_AFTER_FRAMES=900 SJ_CAPTURE_PATH=shot.ppm cargo run -- --play` runs 900 game frames,
writes the window as a PPM image and quits. Without `--play` it captures the launcher instead.
Use it to check rendering without a screen recorder.
