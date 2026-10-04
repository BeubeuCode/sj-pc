---
name: nds-re
description: Nintendo DS reverse-engineering and libretro-host knowledge for the Strange Journey PC frontend. Use when finding game variables in RAM, writing ARM9 patches or hooks, reading Ghidra output for the ARM9, touching the melonDS-ds libretro integration, or debugging rendering, touch or input in this repo.
---

# NDS reverse engineering for Strange Journey

## Facts you can rely on
- ROM `roms/sj_usa.nds`, game code `BMTE`. ARM9 loads at `0x02000000`, entry `0x02000800`.
  Two tiny compressed overlays share `0x02176EA0`. ARM7 at `0x02380000` is the stock Nitro sound driver: ignore it.
- Main RAM is 4 MB at `0x02000000`. Read and write it only through `sj_game::game_api::GameApi`.
  Convert addresses with `Arm9Addr::main_ram_offset(len)`; it rejects anything outside main RAM.
- The CPU is an ARM946E-S (ARMv5TE), little-endian, mixed ARM and Thumb. AAPCS calls: args `r0`–`r3`, return `r0`.
- Nitro SDK fixed-point: `fx32` 20.12 (`0x1000` = 1.0), `fx16` 4.12. 4:3 aspect is `0x1555`, 16:9 is `0x1C71`.
- 3D geometry ports start at `0x04000400`. Main 2D engine registers at `0x04000000`, sub engine at `0x04001000`.
- Hardware reference: GBATEK (problemkaputt.de/gbatek.htm). Name hardware constants exactly as GBATEK does.

## Recipes
- **Find a variable:** follow the memory-diff recipe in `docs/re/README.md`, then confirm with a Ghidra xref.
- **Patch code or data:** add a `sj_game::patch::Patch` with the exact original bytes. `apply` refuses on
  mismatch, so a different ROM revision fails loudly. Test it against `FakeGame` in the same file.
- **Record every find** in `docs/re/symbols.txt` (`address size name notes`) and `docs/re/MEMORY_MAP.md`.
- **See the screen without a display:** `SJ_CAPTURE_AFTER_FRAMES=N SJ_CAPTURE_PATH=x.ppm cargo run -p sj -- --play`
  (without `--play` it captures the launcher).
  Convert the PPM to PNG with a few lines of stdlib Python (zlib + struct) to view it.
- **Boot test without a window:** `cargo test -p sj-emu --test rom_smoke -- --ignored` runs 600 frames in software mode.

## libretro host gotchas (learned the hard way)
- The core is pinned to the `top-bottom` layout with a zero gap (`crates/sj/src/core_options.rs`).
  Touch mapping in `sj_game::touch` assumes it. Change both together or touches land in the wrong place.
- GL frames from the core are stored bottom-up; uploaded software frames are top-down. `present::source_rows` handles it.
- libretro callbacks have no user pointer, so state is thread-local and only one `Core` can exist per process.
  Integration tests that load the core must each live in their own `tests/*.rs` file.
- Rust cannot define C-variadic functions on stable, so we don't provide the log interface. Core logs go to stderr.
- Option keys and values come from `extern/melonDS-ds/src/libretro/config/constants.hpp` and `definitions/*.hpp`.
  Unknown keys fall back to core defaults, with a "Failed to get value" line on stderr. That line is harmless.
- On high-DPI macOS, SDL mouse events are in window points while egui-sdl2 expects drawable pixels.
  Every event going to egui must pass through `launcher::mouse_points::mouse_event_in_drawable_pixels`,
  or clicks land at half their position. The future F1 menu needs the same treatment.
- `RETRO_MEMORY_SYSTEM_RAM` returns null while the core shows its own error screen. `GameApi` then yields an empty slice.

- Controllers: the launcher's Controls section lists connected pads and the last pad input received.
  Use it first when a controller "doesn't work": it tells detection problems from input problems.
  On macOS, Xbox pads go through Apple's driver (docs/DECISIONS.md 013).

## Build gotchas
- Homebrew rustup is keg-only: `export PATH="/opt/homebrew/opt/rustup/bin:$PATH"`.
- CMake 4 needs `CMAKE_POLICY_VERSION_MINIMUM=3.5` for the bundled SDL2. It is set in `.cargo/config.toml`.
- Rebuild the core with `scripts/build-core.sh`. Rust never builds it.

## Planned extensions (not built)
- **M4 hooks:** a melonDS-ds fork command to register ARM9 PC breakpoints that call into Rust. See `docs/HOOKS.md`.
- **M5 mods:** card-read redirect by file id to `mods/<name>/Data/...`, plus Lua over `GameApi`.
