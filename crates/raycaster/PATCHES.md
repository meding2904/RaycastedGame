# Patch against upstream `markusmoenig/raycaster`

Upstream: <https://github.com/markusmoenig/raycaster>, commit `561d5030947e7d060c060b8bf1a90d83637d958c`
(tag 0.3.1, 2023-03-18), MIT licensed. Files are re-saved with LF line endings so the patch
below applies cleanly.

Everything outside this list is upstream code, unchanged. All changes are additive or fix
outright panic paths; none of them change rendering for worlds that use no doors.

## `src/lib.rs`

- Re-export `Raycaster` at the crate root, next to the existing `WorldMap`, `Tile`, `Sprite` and
  `Light` re-exports. Upstream only exposes it as `raycaster::raycaster::Raycaster`, which is an
  accident of the module layout.
- `#![forbid(unsafe_code)]` at the top of the crate, which upstream does not have. The upstream
  source contains no `unsafe` block, so this only makes that guarantee enforced rather than
  assumed.
- `#![allow(clippy::all)]` with a comment: the engine is vendored upstream code and is not linted
  strictly, so the game crate can be (see `.github/workflows/ci.yml`).

## `src/tile.rs`

- `Tile` gains two fields: `door: bool` and `opening: f32` (0.0 closed .. 1.0 open).
- New constructors/helpers: `Tile::door(image_id, rect)` and `Tile::set_opening(f32)`.
- The existing `textured`, `textured_anim` and `colored` constructors set `door: false`,
  `opening: 0.0`, so upstream behaviour is unchanged.

## `src/worldmap.rs`

- `is_blocking(x, y)`: like `has_wall`, but a door at `opening >= 1.0` does not block. Used by
  the movement helpers.
- `remove_wall`, `wall_mut`, `iter_walls`, `wall_count`, `clear_sprites`: the game layer needs
  to remove walls (pushwalls), animate doors in place, and rebuild the sprite list per frame.

## `src/raycaster.rs`

Camera access, so the game can own movement and collision instead of fighting the engine:

- `pos()`, `dir()`, `angle()`, `set_dir_angle(radians)`, `set_move_speed(speed)`.
  `set_dir_angle` normalizes the direction and rebuilds the camera plane perpendicular to it,
  keeping upstream's 0.66 horizontal FOV. The plane is `(-dir.y, dir.x) * 0.66`, not upstream's
  `(dir.y, -dir.x)`: our maps are authored with y increasing downwards, and upstream's sign turned
  every view into a mirror image, so pressing left turned the world right and the mouse ran
  backwards. `tests/orientation.rs` pins the convention: facing north, west is on the left,
  east is on the right, increasing the heading turns right.
- `move_by(world, dx, dy)`, used by `go_forward`/`go_backward`, plus new
  `strafe_left`/`strafe_right`. `move_by` tests both axes independently, which fixes upstream's
  `go_backward`, whose second test uses a position it has already modified.

Doors (the feature upstream lists as TODO):

- `wall_texture_column(...)` computes the texture column of a wall hit and returns `None` when
  the ray passes through a doorway column that the door panel has already slid past.
- The DDA in both `render_st` and `render_mt` uses it: instead of stopping at the first wall,
  a ray keeps travelling through an open doorway and hits whatever is behind it.
- The wall texturing in both renderers clamps `tex_x` and, for door tiles, slides the sampled
  column sideways by `opening` so the panel appears to recede into the wall.

Crash guards (all of these panicked or read out of bounds before):

- Clamp `tex_x` and `tex_pos` to the tile rect in both wall texturers. A `wall_x` fraction
  rounding to exactly 1.0 made `tex_x == rect.2`, reading a neighbouring tile or past the end
  of the image.
- Skip sprites with `transform_y <= 0.0` before any division, and clamp `sprite.shrink` to a
  minimum of 1. A sprite close to the camera produced a zero `sprite_width`/`sprite_height`
  and then divided by zero.
- Skip sprites with a zero projected size, and clamp `draw_end_y` to `>= 0` so a sprite above
  the screen cannot build a reversed (wrap-around) pixel range.
- Compute the sprite source row in `isize` instead of `usize`: `(y - v_move_screen)` underflowed
  and wrapped to a huge number for sprites pushed down the screen by `move_y`.
- Clamp `tex_y` into the tile rect in the single-threaded sprite path too.
- Compute the sprite source column in `i64` and clamp it to the texture width. `256 * offset *
  tex_width` overflows `i32` as soon as an enemy is close enough to the camera to project wider
  than a few thousand pixels, which panicked the worker threads ("attempt to add with overflow")
  the first time a guard walked into the player in a debug build.

Removed: the leftover `println!("tick time ...")` inside `render_st`. The multithreaded path
used by the game (`render_mt`) never had it.

## Rebase notes

`render_mt` already honours the `in_rect` offsets of `render(frame, rect, stride, world)`, so the
game renders the 3D view into the top 160 rows of a 320x200 frame and lets `pixels`-style HUD code
own the rest. No patch was needed for that, contrary to the first draft of the design note.
