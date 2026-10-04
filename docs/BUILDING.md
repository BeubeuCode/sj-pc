# Building

## What you need everywhere
- Rust stable (via rustup) with `clippy` and `rustfmt`.
- CMake 3.20 or newer, and a C/C++ compiler. CMake builds the melonDS-ds core and the bundled SDL2.
- Git, for the `extern/melonDS-ds` submodule.
- Your own dump of *Shin Megami Tensei: Strange Journey* (USA), placed at `roms/sj_usa.nds`.

```
git submodule update --init
./scripts/build-core.sh        # once, takes a few minutes; output lands in cores/
cargo run -p sj                # first run writes sj.toml with defaults
```

`sj.toml`, `roms/`, `cores/` and `saves/` are relative to the working directory, so run from the repo root.

## macOS
```
brew install rustup cmake
rustup default stable
rustup component add clippy rustfmt
```
Homebrew's rustup is keg-only. Add it to your shell's PATH:
```
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
```

## Linux (Debian/Ubuntu)
```
sudo apt install build-essential cmake git libgl1-mesa-dev libasound2-dev libpulse-dev \
                 libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev libwayland-dev libudev-dev
curl https://sh.rustup.rs -sSf | sh
```
The X11, Wayland, ALSA and PulseAudio headers let the bundled SDL2 build its backends.

## Windows
Install Visual Studio Build Tools (the C++ workload), CMake and rustup (MSVC toolchain).
Run `scripts/build-core.sh` from Git Bash, or run the two `cmake` commands from it by hand.
The core comes out as `melondsds_libretro.dll`.

## Notes
- `.cargo/config.toml` sets `CMAKE_POLICY_VERSION_MINIMUM=3.5`. CMake 4 refuses SDL2's old
  minimum-version line without it.
- SDL2 is compiled from source and linked statically. No system SDL is needed.
- The core is loaded at runtime from the path in `sj.toml`, so rebuilding it needs no Rust rebuild.

## Checks before calling anything done
```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -p sj-emu --test rom_smoke -- --ignored   # needs the ROM and the core
```
