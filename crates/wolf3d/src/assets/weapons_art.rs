//! The weapon in the player's hands, drawn at 128x128 and blitted into the bottom of the view.
//!
//! The look follows Wolfenstein 3D's weapon sprites: each weapon is small, sits at the bottom
//! centre of the cell and is seen from directly behind, so the guns are narrow cones that widen
//! towards the player. Everything is drawn on a coarse 64x64 art grid whose pixels are doubled.
//! Sleeves are grey, fists are peach, the pistol is near black, the machine gun is striped silver,
//! the chaingun is teal, and the muzzle flash is a jagged red, orange and yellow star.
use super::{sprites::WeaponKind, Atlas, Canvas};

/// Weapons are drawn at this size, unlike the 64 pixel actor cells
pub const WEAPON_CELL: i32 = 128;

/// Each art pixel is `SCALE` x `SCALE` real pixels, which gives the chunky look
const SCALE: i32 = 2;
/// Side of the art grid, in art pixels
const GRID: i32 = WEAPON_CELL / SCALE;
/// Centre line of every weapon on the art grid
const CX: i32 = 32;

type Rgba = [u8; 4];

const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    [r, g, b, 255]
}

// Grey cloth sleeves
const CLOTH_DARK: Rgba = rgb(92, 92, 96);
const CLOTH: Rgba = rgb(152, 152, 156);
const CLOTH_LIT: Rgba = rgb(196, 196, 200);

// Skin
const SKIN_LIT: Rgba = rgb(255, 216, 168);
const SKIN: Rgba = rgb(252, 172, 124);
const SKIN_DARK: Rgba = rgb(222, 128, 72);

// Pistol
const GUN_BLACK: Rgba = rgb(28, 28, 32);
const GUN_DARK: Rgba = rgb(62, 62, 68);
const GUN: Rgba = rgb(108, 108, 114);
const GUN_LIT: Rgba = rgb(176, 176, 182);

// Knife
const BLADE_WHITE: Rgba = rgb(250, 250, 252);
const BLADE_LIT: Rgba = rgb(206, 206, 210);
const BLADE: Rgba = rgb(160, 160, 166);
const BLADE_DARK: Rgba = rgb(100, 100, 106);
const HILT: Rgba = rgb(52, 56, 52);

// Machine gun, darkest to brightest
const SILVER_RAMP: [Rgba; 5] = [
    rgb(40, 40, 46),
    rgb(98, 98, 104),
    rgb(152, 152, 158),
    rgb(204, 204, 210),
    rgb(246, 246, 250),
];
const BORE: Rgba = rgb(16, 16, 20);

// Chaingun, darkest to brightest
const TEAL_RAMP: [Rgba; 5] = [
    rgb(0, 48, 52),
    rgb(0, 104, 108),
    rgb(24, 160, 164),
    rgb(110, 214, 216),
    rgb(214, 252, 252),
];
const RING_GREY: Rgba = rgb(116, 118, 124);

// Muzzle flash, outside in
const FLASH_EDGE: Rgba = rgb(138, 0, 0);
const FLASH_RED: Rgba = rgb(226, 20, 16);
const FLASH_ORANGE: Rgba = rgb(255, 132, 16);
const FLASH_YELLOW: Rgba = rgb(255, 234, 70);
const FLASH_WHITE: Rgba = rgb(255, 255, 232);

// ---------------------------------------------------------------------------------------------
// Primitives. Every coordinate below is on the 64x64 art grid and is clipped to it.
// ---------------------------------------------------------------------------------------------

fn dot(c: &mut Canvas, x: i32, y: i32, color: Rgba) {
    if x < 0 || y < 0 || x >= GRID || y >= GRID {
        return;
    }
    c.rect(x * SCALE, y * SCALE, SCALE, SCALE, color);
}

fn row(c: &mut Canvas, x: i32, y: i32, w: i32, color: Rgba) {
    for i in 0..w {
        dot(c, x + i, y, color);
    }
}

fn disc(c: &mut Canvas, cx: i32, cy: i32, r: i32, color: Rgba) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r + r {
                dot(c, cx + dx, cy + dy, color);
            }
        }
    }
}

/// A stamped thick line, used for sleeves and arms
fn limb(c: &mut Canvas, a: (i32, i32), b: (i32, i32), radius: i32, color: Rgba) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs())).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.0 as f32 + (b.0 - a.0) as f32 * t;
        let y = a.1 as f32 + (b.1 - a.1) as f32 * t;
        disc(c, x.round() as i32, y.round() as i32, radius, color);
    }
}

/// An oval whose colour is chosen per pixel from its position inside it, each axis running -1..1
fn oval_shaded(
    c: &mut Canvas,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    color: impl Fn(f32, f32) -> Rgba,
) {
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry + rx * ry {
                dot(
                    c,
                    cx + dx,
                    cy + dy,
                    color(dx as f32 / rx as f32, dy as f32 / ry as f32),
                );
            }
        }
    }
}

/// Stamps rows of pixel art, each row centred on `cx`. A space is left transparent.
fn art(c: &mut Canvas, cx: i32, top: i32, rows: &[&str], palette: impl Fn(char) -> Rgba) {
    for (j, line) in rows.iter().enumerate() {
        let half = line.len() as i32 / 2;
        for (i, ch) in line.chars().enumerate() {
            if ch != ' ' {
                dot(c, cx - half + i as i32, top + j as i32, palette(ch));
            }
        }
    }
}

/// Steps a colour down a ramp, to shade it
fn ramp_pick(ramp: &[Rgba; 5], level: i32) -> Rgba {
    ramp[level.clamp(0, 4) as usize]
}

// ---------------------------------------------------------------------------------------------
// Shared parts
// ---------------------------------------------------------------------------------------------

/// A peach fist, lit on the upper left and shaded on the right
fn fist(c: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32) {
    oval_shaded(c, cx, cy, rx, ry, |nx, ny| {
        if nx > 0.5 || ny > 0.7 {
            SKIN_DARK
        } else if nx < -0.1 && ny < 0.2 {
            SKIN_LIT
        } else {
            SKIN
        }
    });
}

/// Cloth with a little weave so it does not read as a flat block
fn cloth(x: i32, y: i32, edge: i32, half: i32) -> Rgba {
    if edge < 2 {
        CLOTH_LIT
    } else if edge >= half * 2 - 1 || (x * 3 + y * 5) % 7 == 0 {
        CLOTH_DARK
    } else {
        CLOTH
    }
}

/// A jagged starburst of dark red, red, orange, yellow and a white core. `aspect` stretches it
/// sideways and `down` is how many rows it may reach below its centre, since the gun hides the rest.
fn flash(c: &mut Canvas, cx: i32, cy: i32, size: f32, aspect: f32, spikes: i32, down: i32) {
    const JITTER: [f32; 11] = [1.0, 0.78, 1.14, 0.9, 1.06, 0.72, 1.1, 0.84, 1.18, 0.8, 0.95];
    let layers = [
        (1.0, FLASH_EDGE),
        (0.88, FLASH_RED),
        (0.64, FLASH_ORANGE),
        (0.42, FLASH_YELLOW),
    ];
    let reach_x = (size * aspect * 1.2) as i32 + 2;
    let reach_y = (size * 1.2) as i32 + 2;
    for y in (cy - reach_y)..=(cy + down) {
        for x in (cx - reach_x)..=(cx + reach_x) {
            let dx = (x - cx) as f32 / aspect;
            let dy = (y - cy) as f32;
            let r = (dx * dx + dy * dy).sqrt();
            let turn = dy.atan2(dx) / std::f32::consts::TAU + 1.0;
            let k = turn * spikes as f32;
            let frac = k - k.floor();
            let valley = (2.0 * frac - 1.0).abs();
            let jitter = JITTER[(k.floor() as usize) % JITTER.len()];
            let edge = size * (1.0 - 0.6 * valley) * jitter;

            if r <= size * 0.2 {
                dot(c, x, y, FLASH_WHITE);
                continue;
            }
            let mut painted = None;
            for (scale, color) in layers {
                if r <= edge * scale {
                    painted = Some(color);
                }
            }
            if let Some(color) = painted {
                dot(c, x, y, color);
            }
        }
    }
}

/// Draws one weapon frame into a `WEAPON_CELL` sized cell
pub fn draw(c: &mut Canvas, kind: WeaponKind, frame: usize) {
    match kind {
        WeaponKind::Knife => knife(c, frame),
        WeaponKind::Pistol => pistol(c, frame),
        WeaponKind::MachineGun => machine_gun(c, frame),
        WeaponKind::Chaingun => chaingun(c, frame),
    }
}

// ---------------------------------------------------------------------------------------------
// Weapons
// ---------------------------------------------------------------------------------------------

/// A small combat knife held point up and slightly away, in a fist at the end of a grey sleeve.
/// Frame 0 is the ready pose; later frames thrust it forward and up.
fn knife(c: &mut Canvas, frame: usize) {
    let (tx, ty, arm) = if frame > 0 { (29, 40, 6) } else { (33, 45, 4) };

    // Sleeve running off the bottom right edge, lighter on its upper left
    let from = (tx + 9, ty + 14);
    let to = (tx + 31, ty + 38);
    limb(c, from, to, arm, CLOTH_DARK);
    limb(
        c,
        (from.0 - 1, from.1 - 1),
        (to.0 - 1, to.1 - 1),
        arm - 1,
        CLOTH,
    );
    limb(
        c,
        (from.0 - 2, from.1 - 2),
        (to.0 - 2, to.1 - 2),
        (arm - 3).max(0),
        CLOTH_LIT,
    );

    // Fist behind the guard
    fist(c, tx + 7, ty + 13, 5, 3);

    // Blade: a wedge with a bright vertical edge, widening towards the guard
    for i in 0..9 {
        let width = 1 + (i as f32 * 1.3).round() as i32;
        for k in 0..width {
            let color = if k == 0 || (k == 1 && i >= 4) {
                BLADE_WHITE
            } else if k + 1 == width {
                BLADE_DARK
            } else if k * 3 < width * 2 {
                BLADE_LIT
            } else {
                BLADE
            };
            dot(c, tx + k, ty + i, color);
        }
    }

    // Dark crossguard with its two ends curling down round the fist
    row(c, tx - 1, ty + 9, 13, HILT);
    row(c, tx - 1, ty + 10, 2, HILT);
    row(c, tx + 11, ty + 10, 2, HILT);
}

/// The sidearm: a small near-black pistol seen end on, above a single fist and two grey cuffs.
/// Frame 0 is the ready pose, frame 1 fires and frame 2 is the kick, with the gun thrown upwards.
fn pistol(c: &mut Canvas, frame: usize) {
    let lift = if frame == 2 { 5 } else { 0 };
    let top = 40 - lift;

    // Cuffs either side of the wrist, with a dark gap between them
    let cuff_top = 58 - lift;
    for y in cuff_top..GRID {
        let half = 7 + (y - cuff_top) * 5 / (GRID - cuff_top).max(1);
        for x in (CX - half)..=(CX + half) {
            let color = if (x - CX).abs() <= 2 {
                CLOTH_DARK
            } else {
                cloth(x, y, x - (CX - half), half)
            };
            dot(c, x, y, color);
        }
    }

    fist(c, CX, 55 - lift, 7, 6);

    // The gun itself: sight, barrel, then the squat dark body that fills the hand
    const PISTOL: [&str; 13] = [
        "d",
        "dld",
        "dld",
        "dgd",
        "kgldk",
        "kgldk",
        "kgllgdk",
        "kglgddk",
        "kglgdkk",
        "kgldddk",
        "kdglddddk",
        "kdgdddddk",
        "kddddddkk",
    ];
    let barrel = if lift > 0 { 2 } else { 0 };
    // A longer barrel shows as the gun is thrown up by repeating its middle rows
    for extra in 0..barrel {
        art(c, CX, top + extra, &PISTOL[1..2], gun_pixel);
    }
    art(c, CX, top + barrel, &PISTOL, gun_pixel);

    match frame {
        1 => flash(c, CX, top - 3, 4.5, 1.2, 7, 1),
        2 => flash(c, CX, top - 3, 2.5, 1.2, 7, 0),
        _ => {}
    }
}

fn gun_pixel(ch: char) -> Rgba {
    match ch {
        'k' => GUN_BLACK,
        'd' => GUN_DARK,
        'g' => GUN,
        _ => GUN_LIT,
    }
}

/// The machine gun: a pointed silver shroud with vent bands, a dark open rear and a grey sleeve
/// reaching in from the left. Frame 0 is ready, frame 1 fires and frame 2 is the kick.
fn machine_gun(c: &mut Canvas, frame: usize) {
    let tip = if frame == 2 { 43 } else { 46 };

    // Left arm with the edge of a hand where it meets the gun
    let arm_y = tip + 14;
    limb(c, (11, arm_y + 9), (27, arm_y + 1), 2, CLOTH);
    limb(c, (11, arm_y + 7), (27, arm_y - 1), 0, CLOTH_LIT);
    limb(c, (11, arm_y + 11), (27, arm_y + 3), 0, CLOTH_DARK);
    dot(c, 27, arm_y, SKIN);
    dot(c, 28, arm_y, SKIN_DARK);
    dot(c, 28, arm_y + 1, SKIN_DARK);

    for y in tip..GRID {
        let into = y - tip;
        let half = if into < 3 {
            1.0
        } else {
            1.5 + (into - 3) as f32 * 0.43
        };
        let reach = half.round() as i32;
        for x in (CX - reach)..=(CX + reach) {
            let u = (x - CX) as f32 / half.max(1.0);
            if into < 3 {
                let color = if u.abs() < 0.5 {
                    SILVER_RAMP[0]
                } else {
                    SILVER_RAMP[1]
                };
                dot(c, x, y, color);
                continue;
            }
            let mut level = if u < -0.55 {
                3
            } else if u < 0.1 {
                4
            } else if u < 0.5 {
                2
            } else {
                1
            };
            // Vent bands every few rows
            if into % 4 == 0 {
                level -= 1;
            }
            let color = if into >= 15 && (x - CX).abs() <= 2 {
                BORE
            } else {
                ramp_pick(&SILVER_RAMP, level)
            };
            dot(c, x, y, color);
        }
    }

    match frame {
        1 => flash(c, CX + 1, tip - 1, 11.0, 1.6, 10, 8),
        2 => flash(c, CX + 1, tip - 2, 4.0, 1.4, 8, 1),
        _ => {}
    }
}

/// The chaingun: a big teal cone of brushed metal under a dark muzzle ring. Frame 0 is ready,
/// frames 1 and 2 fire with the gun rising, and frame 3 settles back down.
fn chaingun(c: &mut Canvas, frame: usize) {
    // Ring row, half width under the ring and half width at the bottom of the view
    let (ring, hw_top, hw_bot) = match frame {
        1 => (46, 7.0, 17.0),
        2 => (44, 8.0, 21.0),
        3 => (50, 6.0, 15.0),
        _ => (46, 7.0, 17.0),
    };
    let firing = frame == 1 || frame == 2;
    let phase = frame as i32;

    // Cone of brushed metal: bright streaks that crawl a little each frame
    for y in (ring + 2)..GRID {
        let t = (y - ring - 2) as f32 / (GRID - ring - 3).max(1) as f32;
        let half = hw_top + (hw_bot - hw_top) * t;
        let reach = half.round() as i32;
        for x in (CX - reach)..=(CX + reach) {
            let u = (x - CX) as f32 / half;
            let streak = (x * 7 + y / 6 + phase * 3).rem_euclid(5) as usize;
            let mut level = [4, 3, 3, 2, 2][streak];
            level -= if u < -0.2 {
                0
            } else if u < 0.35 {
                1
            } else if u < 0.8 {
                2
            } else {
                3
            };
            dot(c, x, y, ramp_pick(&TEAL_RAMP, level));
        }
    }

    // Dome over the barrels, then the dark ring they sit in
    let dome = hw_top.round() as i32 + 1;
    oval_shaded(c, CX, ring - 1, dome, 4, |nx, ny| {
        if ny < -0.6 {
            TEAL_RAMP[3]
        } else if nx < -0.3 {
            TEAL_RAMP[2]
        } else {
            TEAL_RAMP[1]
        }
    });
    dot(c, CX, ring - 4, TEAL_RAMP[4]);
    dot(c, CX - 1, ring - 3, TEAL_RAMP[4]);
    row(c, CX - dome, ring, dome * 2 + 1, TEAL_RAMP[0]);
    row(c, CX - dome, ring + 1, dome * 2 + 1, TEAL_RAMP[0]);
    if firing {
        // The flash lights the rim of the muzzle
        row(c, CX - dome + 1, ring - 1, dome * 2 - 1, RING_GREY);
    }

    match frame {
        1 => flash(c, CX + 1, ring - 1, 14.0, 1.8, 12, 9),
        2 => flash(c, CX + 1, ring - 3, 16.0, 1.5, 12, 9),
        _ => {}
    }
}

/// Builds the weapon atlas: one `WEAPON_CELL` cell per animation frame, in `WeaponKind` order
pub fn atlas() -> Atlas {
    let total: usize = WeaponKind::ALL.iter().map(|kind| kind.frames()).sum();
    let mut atlas = Atlas::from_cells(total, WEAPON_CELL);
    let mut index = 0;
    for kind in WeaponKind::ALL {
        for frame in 0..kind.frames() {
            let mut canvas = atlas.canvas(index);
            draw(&mut canvas, kind, frame);
            index += 1;
        }
    }
    atlas
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(atlas: &Atlas, x: i32, y: i32) -> bool {
        let off = ((y * atlas.width + x) * 4) as usize;
        atlas.data[off + 3] != 0
    }

    #[test]
    fn every_frame_is_drawn_and_keeps_off_the_side_edges() {
        let atlas = atlas();
        let cell = WEAPON_CELL;

        for index in 0..atlas.cell_count() {
            let x0 = (index as i32 % atlas.cols) * cell;
            let y0 = (index as i32 / atlas.cols) * cell;
            let mut painted = 0;

            for y in 0..cell {
                for x in 0..cell {
                    if !opaque(&atlas, x0 + x, y0 + y) {
                        continue;
                    }
                    painted += 1;
                    let on_edge = x == 0 || x == cell - 1;
                    assert!(!on_edge, "cell {index} paints the edge at ({x}, {y})");
                }
            }

            // Weapons are small in the cell, as in the original sprites
            assert!(painted > cell * cell / 64, "cell {index} is nearly empty");
        }
    }
}
