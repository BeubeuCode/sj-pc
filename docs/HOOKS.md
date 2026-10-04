# Hooks (M4, not built yet)

A hook replaces one original ARM9 function with native Rust while the rest of the game keeps running
on the emulated CPU. This is how the decomp lands: function by function, always playable.

## Mechanism (planned)
1. Fork extension in melonDS-ds: a private environment command lets the frontend register
   `(arm9_addr, callback)`. The core checks the ARM9 PC against the registered set before executing
   an instruction block. This needs the JIT disabled, or block-boundary checks, for hooked addresses.
2. On a hit the core calls our callback with access to the registers (`r0`–`r3` arguments, `sp`, `lr`).
3. The callback runs the native version through `GameApi`, writes the return value to `r0`,
   and sets `pc = lr`. The original function body never runs.

## Rules for every hook
- The native function is pure over plain structs and lives in `sj-game/src/hooks/`.
- The ARM9 address and the original's Ghidra name go in `docs/re/symbols.txt`.
- A unit test exercises it with a small hand-checked fixture. No ROM data in fixtures.
- It can be turned off from `sj.toml` so we can A/B against the original.
- A verification run compares RAM side effects of hooked and unhooked execution over the same savestate.

## Ledger
| ARM9 address | Original name | Native function | Status |
|---|---|---|---|
| — | — | — | none yet |

## Order
Map screen renderer first: it is the biggest one-screen win and fairly self-contained.
Then menus, then dialog boxes.
