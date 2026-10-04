# Style

The goal is code a tired person can read cold, one file at a time.

## Naming
- Names say what a value is and its unit: `window_width_px`, `frame_budget_us`, `scale_factor`.
- Abbreviations allowed: `px`, `us`, `ms`, `id`, `ram`, `vram`, `arm9`, `arm7`, `nds`.
- Addresses and sizes use newtypes (`Arm9Addr`, `SizePx`) so mixing them fails to compile.
- Hardware constants are named as in GBATEK and live in one module.

## Shape
- A function fits on one screen and does one thing. Return early instead of nesting.
- Data is a plain `struct` with `pub` fields. Add methods only to protect an invariant.
- No trait hierarchies. The one trait that matters is `GameApi`.
- Generics only where the standard library needs them.
- One type with behaviour per file.

## Errors
- `Result<T, Error>` with a `thiserror` enum at crate boundaries.
- No `unwrap` or `expect` outside tests and `main`.

## Unsafe
- Emulator FFI only in `crates/sj-emu/src/ffi/`.
- OpenGL calls only in `crates/sj/src/present.rs`, plus the GL loader line in `app.rs`.
- Each `unsafe` block has a `// SAFETY:` line saying why it holds.

## Comments
- No docstrings.
- A comment explains *why*, never *what*. Typical cases: a hardware quirk, or a game workaround
  with the ARM9 address it depends on.

## Tests
- Every pure function in `sj-game` has a test in the same file under `#[cfg(test)]`.
- Parsers, layout math and patches are written test-first.
- No test frameworks. Plain `assert_eq!`.
- Tests that need the ROM are `#[ignore]` and skip quietly when it is missing.

## Tooling
`cargo fmt` defaults. `cargo clippy` with `pedantic` on, minus the lints listed in the workspace
`Cargo.toml`. Both run before a task is done.
