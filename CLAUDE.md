# Strange Journey PC

Single-screen PC frontend for *Shin Megami Tensei: Strange Journey* (NDS, USA, `BMTE`) on Linux,
Windows and macOS. The game runs on the melonDS core through the libretro C ABI. Game-specific
knowledge is reverse-engineered progressively and replaces original code hook by hook.

Read `docs/ROADMAP.md` for where we are. Read `docs/ARCHITECTURE.md` before touching code.

## Hard rules
- The ROM lives only in `roms/sj_usa.nds`. `roms/` is gitignored. Never move it elsewhere.
- Never commit ROM-derived data: extracted files, graphics, text dumps, disassembly listings,
  Ghidra databases. Addresses and names in `docs/re/symbols.txt` are fine. Local Ghidra work goes in
  `re/ghidra/` (gitignored).
- `unsafe` exists only in `crates/sj-emu/src/ffi/` (emulator FFI) and `crates/sj/src/present.rs`
  plus the GL loader in `app.rs` (OpenGL). Each block gets a `// SAFETY:` line.
- Everything that reads or writes game memory goes through the `GameApi` trait in `sj-game`.
  It is the future mod surface; our own code is its first user.
- Ask before committing. Never add co-author lines. Never mention Claude Code in PR bodies.

## Commands
Rust from Homebrew is keg-only: `export PATH="/opt/homebrew/opt/rustup/bin:$PATH"`.
```
./scripts/build-core.sh      # once, builds cores/melondsds_libretro.*
cargo build
cargo run                     # launcher first
cargo run -- --play           # straight into the game
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```
All of `test`, `clippy` and `fmt --check` pass before a task is done.
Research loop (see `docs/ARCHITECTURE.md`, "Research tooling"):
```
cargo build --release -p sj-lab && ./target/release/sj-lab re/scripts/<script>.txt
cargo test -p sj-game --test sample_modes -- --ignored   # mode detection vs lab snapshots
cargo run -p sj -- --play --state re/samples/<label>-<n>/savestate.state
```
The ROM smoke test is `#[ignore]`d. Run it with `cargo test -p sj-emu --test rom_smoke -- --ignored`.
To check rendering without a screen, capture a frame:
`SJ_CAPTURE_AFTER_FRAMES=900 SJ_CAPTURE_PATH=/tmp/shot.ppm cargo run -p sj -- --play`
(drop `--play` to capture the launcher).

## Crates
- `sj-game`: pure game knowledge. No SDL, no GL, no FFI. Nearly all tests live here.
- `sj-emu`: thin libretro wrapper. Callbacks reach a thread-local, so one core per process.
- `sj`: the binary. `Platform` (window, GL, controllers), the egui launcher, then the game:
  GL presenter, input, audio, hotkeys, savestates.

## Style
Full contract in `docs/STYLE.md`. The short version:
- Code must be readable by someone who opens one file cold.
- Names carry meaning and units: `window_width_px`, `frame_budget_us`, `arm9_addr`.
- Newtypes for addresses and pixel sizes. Constants instead of magic numbers.
- Small functions, early returns, no nesting pyramids.
- Plain structs with pub fields. Methods only to protect an invariant.
- `Result` + `thiserror` across crate boundaries. No `unwrap` outside tests and `main`.
- No docstrings. Comments only explain *why*: a hardware quirk, or a game workaround with
  its ARM9 address.
- One type with behaviour per file.
- Pure logic gets a `#[test]`. Parsers, layout math and patches are written test-first.

## Docs
Update the matching page under `docs/` in the same change as the code it describes.
Reverse-engineering notes go in `docs/re/`. Agent-facing NDS knowledge lives in
`.claude/skills/nds-re/SKILL.md`; extend it when you learn something reusable.
