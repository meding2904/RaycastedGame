# Wolfenstein 3D clone on `markusmoenig/raycaster` — design

Date: 2026-10-18
Status: awaiting user approval

## Goal

A feature-for-feature Wolfenstein 3D (1992, id Software) clone in memory-safe Rust that runs on
macOS, Linux and Windows, rendered with the `markusmoenig/raycaster` software raycaster.

"1 to 1 feature" means the *feature set* of the original, not its copyrighted assets:

- 4 wall/floor/ceiling-textured pseudo-3D dungeon, 64x64 tile maps
- Sliding doors (open/close, auto-close, locked with gold/silver keys)
- Pushwalls (secret walls that slide two tiles when used)
- Secret areas tracked for the end-of-level tally
- Player: 8-directional free movement with collision, strafing, run, mouse turn, use key
- 4 weapons (knife, pistol, machine gun, chaingun) with ammo, rate, damage, muzzle flash
- 5 enemy types (guard, SS, officer, guard dog, boss) with idle/patrol/chase/attack/pain/death
  states, hearing-based alerting, line-of-sight, dropped ammo
- Pickups: health, ammo, treasure (cross/chalice/chest/crown), gold key, silver key, extra life
- Static decorations: pillars, barrels, tables, lamps, chandeliers, plants
- HUD status bar: score, lives, level, health, ammo, 4 weapon icons, 2 key slots, animated face
  that reacts to health/pain/firing/god mode
- Level flow: "Get Psyched!", floor intro, elevator switch ends the floor, kill/secret/treasure
  tally, death/respawn, game over, high score
- 4 difficulty tiers scaling enemy count/health/damage, ammo and treasure like the original
- Sound effects and looping music bed
- Original 320x200 internal resolution, nearest-neighbour upscale, letterboxed

Levels shipped: E1M1, E1M2, E1M3 rebuilt from the original layout data.

## Constraint check: what the engine does and does not give us

`raycaster` v0.3.1 (MIT) renders into an RGBA frame: textured walls, textured/colored floor and
ceiling, sprites with transparency, z-buffered sprite occlusion, distance fog and tile-based lights.
It has **no** game logic: player position/direction are private, there is no strafe, no `remove_wall`,
no door concept ("Doors" is on its own TODO), and no door-aware renderer.

Therefore:

- **Approach A (chosen): vendor the engine as a workspace crate and extend it.**
  `crates/raycaster` is a copy of upstream at commit `561d503` plus a small, documented patch
  (`crates/raycaster/PATCHES.md`, single-file diffable list). MIT permits this. Upstream stays
  otherwise untouched so a future rebase is cheap.
- Approach B (git dependency + wrapper): impossible — `Raycaster.pos/dir/plane` are private and
  there is no API to strafe, set an absolute heading, or remove a wall. A wrapper cannot reach in.
- Approach C (write our own raycaster): contradicts the request; rejected.

### Engine patch (keep it small)

1. `Raycaster`: expose read-only `pos()`/`dir()`, `set_dir_angle(radians)`, `set_pos`, and
   speed control `set_move_speed`/`set_rot_speed`/`set_run(bool)`; add `strafe_left/right`
   (collision-checked); make `go_forward/backward` collision-check both axes independently
   (upstream's backward test is wrong when moving diagonally). Delete upstream's leftover
   `println!("tick time ...")` in `render_st`.
2. `WorldMap`: add `remove_wall`, `iter_walls`, `wall_count`, `clear_sprites`, and `set_sprite_tile`.
3. `Tile`: add `pub opening: f32` (0.0 closed .. 1.0 fully open). In the wall renderer, the door
   texture source column is shifted by `opening` and pixels that slide past the tile edge are
   skipped, which reproduces Wolf3D's door-receding-into-the-wall look. `WorldMap::has_wall`
   stays a pure map query; blocking is decided by the game layer (`opening < 0.75`).
4. `Tile`: add `pub offset_y: f32`? No — sprites already have `move_y`. Not needed.
5. `render_mt` currently ignores `in_rect.0/1` offsets (it renders from 0,0). We render the 3D view
   into a sub-rectangle of the 320x200 frame (y 0..160, status bar below), so this must be fixed:
   `render_mt` gets the upstream `render_st` behaviour for rect offsets (it is already computed for
   the rotated-copy step, only the buffer layout assumes origin 0). Patch accordingly.

If any patch item proves unnecessary it gets dropped rather than kept "for later".

## Architecture

```
Cargo.toml                     workspace
crates/raycaster/              vendored MIT engine (+ PATCHES.md)
crates/wolf3d/                 the game
  src/main.rs                  window, present loop, scaling, frame pacing
  src/bin/frames.rs           headless harness: render scripted frames to PNG
  src/game.rs                  state machine + the 70 Hz logic tick
  src/level.rs                 level format, map loading, doors, pushwalls, secrets, elevator
  src/raycast.rs               gameplay DDA (line of sight, hitscan) over the level grid
  src/player.rs                movement, collision, health, keys, score, lives, death
  src/actors.rs                enemy types, AI states, damage, drops
  src/objects.rs               static map objects and pickups
  src/weapons.rs               4 weapons + knife: animation, ammo, spread, damage
  src/hud.rs                   status bar, face, full-screen screens, tally
  src/audio.rs                 rodio effects + music, device-optional (no sound card => silent)
  src/input.rs                 keyboard + mouse state, default bindings
  src/assets/mod.rs            asset registry: CC0 files if present, else procedural
  src/assets/textures.rs       procedural 64x64 walls/floors/ceilings/doors/elevator
  src/assets/sprites.rs        procedural enemies, weapons, pickups, HUD face
  src/assets/sfx.rs            procedural sound effects and music (WAV in memory)
  assets/cc0/                  downloaded CC0 art/audio (git-ignored if large; see below)
tools/fetch_assets.sh          curl+sha256 fetch of CC0 packs
tools/ppm2png?...              not needed: harness writes PNG via the `png` crate
docs/                          this spec, WOLF3D_FEATURES.md (feature-by-feature matrix), README
.github/workflows/ci.yml       build+test on ubuntu-latest, macos-latest, windows-latest
```

Crates: `winit` + `softbuffer` (window/present, pure Rust, no C system deps), `rodio` for audio
(Linux needs `libasound2-dev` at build time — documented), `png` for asset decode and harness
encode, `rayon`/`rand`/`rustc-hash` via the vendored engine. No `unsafe` in our code: both crates
carry `#![forbid(unsafe_code)]`. Dependencies used for windowing/audio contain FFI internally;
that is inherent to any desktop app and is stated in the README.

Data flow per frame: `input -> game tick (70 Hz fixed timestep, accumulator) -> level/world update
-> build engine WorldMap view (walls, door openings, object/enemy sprites, lights) -> Raycaster
renders the 3D view into frame[0..160] -> HUD draws rows 160..200 -> scale to window -> present`.

Determinism: the whole game state is a plain struct mutated by a fixed-step tick with a seeded RNG,
so the headless harness can reproduce frames and unit tests can drive the game directly.

## Assets

Original Wolf3D art/sound is copyrighted (id's own LICENSE.DOC: artwork and music are not
redistributable), so it is not used.

1. `tools/fetch_assets.sh` downloads the verified CC0 candidates (OpenGameArt "Old school FPS wall
   textures" CC0, "FPS Weapons Overlay" CC0, "WolfClone sprite pack" CC0, qubodup door sounds CC0)
   into `assets/cc0/`, with SHA-256 pinning and failure isolation: a failed download is a warning,
   never a build failure.
2. `assets/mod.rs` uses a `assets/cc0/<name>.png` when it exists and otherwise calls the procedural
   generator, so the game is always runnable offline and needs no license review.
3. Nazi iconography (swastikas, SS runes) is replaced by neutral geometric emblems. Enemy types keep
   their gameplay roles and names of the original's roster (guard/SS/officer/dog/boss).

## Testing and verification

- Unit tests: map collision, door open/close/lock, pushwall 2-tile slide and secret reveal,
  difficulty scaling, weapon ammo/damage, actor state transitions, tally percentages.
- `cargo test`, `cargo clippy` clean, `cargo build --release`.
- Headless harness (`src/bin/frames.rs`) renders scripted frames (spawn, mid-level, door open,
  dogfight, tally, each difficulty) to PNGs; those are assembled into an HTML gallery and inspected
  visually in the browser panel, since a passing build alone does not prove the game looks right.
- The real binary is launched briefly to confirm window creation and the present loop.
- Cross-platform: CI matrix builds/tests on linux, macos and windows; local verification here is
  Windows-only (the machine I run on), which will be reported as a limitation rather than a pass.

## Out of scope (explicitly)

Network play, save/load games, the console-style level editor, AdLib/OPL2 emulation, the full 60
floors of the retail episodes, and DOS-exact timing/par-times.
