# Configuration

The launcher that opens with `cargo run` edits everything on this page. Press **Play** to
save and start, or **Save settings** to save without playing. Start with `cargo run -- --play`
to skip the launcher.

Settings live in `sj.toml` in the working directory. Any key you leave out keeps its default, and
unbound buttons keep their default bindings. If the file is broken, the launcher shows defaults and
says why. `--play` refuses to start instead.

In the launcher you can drag a `.nds` file onto the window to set the ROM. It checks the cartridge
header and only enables Play for Strange Journey USA (`BMTE`) with a core present.

## Top level
| Key | Default | Meaning |
|---|---|---|
| `rom` | `roms/sj_usa.nds` | ROM path |
| `core` | `cores/melondsds_libretro.<dylib/so/dll>` | libretro core path |
| `save_dir` | `saves` | Game saves, savestates, and the core's own files |

## `[video]`
| Key | Default | Meaning |
|---|---|---|
| `scale` | `4` | 3D internal resolution, 1 to 8 times native |
| `fullscreen` | `false` | Start in borderless fullscreen |
| `vsync` | `true` | Sync presents to the display |
| `widescreen` | `true` | 16:9 3D when the top screen is shown alone; needs our patched core and the OpenGL renderer |
| `top_screen_60fps` | `true` | While the top screen is shown alone, keep the game's 3D on it every frame: 60fps instead of 30 on the ship and in facilities. The hidden bottom screen stops updating until it is shown again |
| `window_width_px`, `window_height_px` | `1280`, `960` | Starting window size; the window is resizable |
| `filter` | `smooth` | `smooth` (bilinear) or `sharp` (nearest-neighbour) scaling of the screens |

## `[pip]` (the touch screen overlay)
| Key | Default | Meaning |
|---|---|---|
| `visible` | `true` | Show the touch screen |
| `corner` | `bottom-right` | `top-left`, `top-right`, `bottom-left` or `bottom-right` |
| `height_fraction` | `0.35` | Overlay height as a share of the window height, 0.1 to 1.0 |
| `margin_px` | `16` | Gap to the window edges, in drawable pixels |

Click or drag inside the overlay with the left mouse button to touch the screen.

## `[hud]`
| Key | Default | Meaning |
|---|---|---|
| `enemy_panel` | `true` | Show enemy HP and MP cards in battle |
| `minimap` | `true` | Show the automap as a minimap while exploring a dungeon |
| `minimap_corner` | `top-right` | Corner of the minimap |
| `minimap_height_fraction` | `0.3` | Minimap height as a share of the game image |
| `minimap_opacity` | `0.85` | 0.2 to 1.0 |
| `margin_px` | `16` | Gap to the window edges |

## `[audio]`
| Key | Default | Meaning |
|---|---|---|
| `volume_percent` | `100` | 0 to 100 |
| `muted` | `false` | Silence the game |

## `[controller]`
| Key | Default | Meaning |
|---|---|---|
| `stick_deadzone_percent` | `50` | How far a stick must move to count as a d-pad press, 10 to 90 |
| `swap_screens_button` | `pad:rightstick` | Pad button that swaps to the bottom screen |

## `[hotkeys]`
| Key | Default |
|---|---|
| `fast_forward` (hold) | `` ` `` |
| `toggle_pip` | `P` |
| `toggle_fullscreen` | `F11` |
| `save_state` | `F5` |
| `load_state` | `F8` |
| `previous_slot`, `next_slot` | `F6`, `F7` |
| `swap_screens` | `M` |
| `snapshot` (research) | `F9` |

`F12` opens the RAM search panel. It is a fixed developer key.

The current slot (0 to 9) shows in the window title. Slots are `saves/slot<N>.state`.

The game also autosaves every 5 minutes and on quit to `saves/autosave.state`, keeping the one before
as `saves/autosave.previous.state`. It skips the title screen, so quitting right after launch never
replaces a real session. The launcher's "Resume last session" starts from the autosave.

## `[bindings]`
Each DS button maps to a list of inputs. Any one of them presses the button. In the launcher,
click **+ Add** and press a key or controller input to bind it, and click a binding to remove it.
Escape cancels. Hotkeys are keyboard-only.
```toml
[bindings]
a = ["X", "pad:a"]
up = ["Up", "pad:dpup", "pad:lefty-"]
```
- **Keyboard:** SDL key names, such as `X`, `Return`, `Right Shift`, `Up`, `Space`.
- **Controller buttons:** `pad:` plus the SDL name: `a`, `b`, `x`, `y`, `back`, `start`,
  `leftshoulder`, `rightshoulder`, `dpup`, `dpdown`, `dpleft`, `dpright`, `leftstick`, `rightstick`.
- **Sticks:** `pad:leftx-`, `pad:leftx+`, `pad:lefty-`, `pad:lefty+`, and the same for `rightx` and `righty`.
- **Triggers:** `pad:lefttrigger`, `pad:righttrigger`.

The launcher's Controls section shows connected controllers and the last pad input it received.
Press a button there to check your controller works before binding anything.

Default controls (bindings version 2):

| DS button | Keyboard | Pad (by label) | In Strange Journey |
|---|---|---|---|
| A | Enter, Space | A | confirm |
| B | Backspace | B | back |
| Y | Escape | Y | main menu |
| X | Tab | X | mission log |
| L / R | Q / E | LB / RB | automap floor |
| Start / Select | F / C | Start / Back | end name entry / config |
| D-pad | WASD, arrows | d-pad, left stick | move and turn |

Files saved before this change have no `bindings_version` and keep their old controls. The launcher
offers to switch with one click.

Buttons: `a b x y l r start select up down left right`.
