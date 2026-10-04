# Contributing

Thanks for helping. This page is the short version; the docs it links to have the details.

## Ground rules
- **Nothing from the game goes in the repository.** No ROM, no extracted files, graphics, sounds,
  text dumps, disassembly listings or Ghidra databases, and no screenshots. Addresses, names and
  descriptions of what the code does are fine (`docs/re/symbols.txt`). The ROM lives only in `roms/`,
  which git ignores; research output goes to the ignored `re/samples/`, `re/ghidra/` and
  `re/scripts/checkpoints/`.
- **Game memory is read and written through `GameApi`** (`crates/sj-game/src/game_api.rs`), never
  directly. It is the future mod surface and our code is its first user.
- **`unsafe` is limited** to the emulator FFI (`crates/sj-emu/src/ffi/`) and the OpenGL code
  (`crates/sj/src/present.rs` and the GL loader in `app.rs`), each block with a `// SAFETY:` comment.

## Setting up
Follow [docs/BUILDING.md](docs/BUILDING.md), then read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
before touching code.

## Before you open a pull request
All three must pass:
```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```
Tests that need the ROM or lab snapshots are `#[ignore]`d; run them locally with `-- --ignored` when
you touch game knowledge (see the commands in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)).

## Code style
[docs/STYLE.md](docs/STYLE.md) is the contract. In short: code readable by someone who opens one file
cold, names that carry meaning and units, small functions with early returns, no docstrings, comments
only for the *why* (a hardware quirk, a game workaround with its ARM9 address). Pure logic gets a test.

## Reverse engineering
New findings go in `crates/sj-game/src/addresses.rs` with a comment saying how they were found, and
in [docs/re/MEMORY_MAP.md](docs/re/MEMORY_MAP.md) and `docs/re/symbols.txt`. The research tools
(headless lab, snapshots, RAM search) are described under "Research tooling" in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), and the memory-diff recipe in
[docs/re/README.md](docs/re/README.md).

## Docs
Update the matching page under `docs/` in the same change as the code it describes. Decisions that
shape the project get an entry in [docs/DECISIONS.md](docs/DECISIONS.md).

## Commits
Small, focused commits with an imperative subject line ("Add Save state to the menu bar"). One
change per commit, so any of them can be reverted on its own.
