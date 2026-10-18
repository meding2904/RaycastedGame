//! The weapon in the player's hands, drawn at 64x64 and blitted into the bottom of the view.
//!
//! The composition follows the 1992 original rather than inventing one: a small gun centred just
//! above the bottom edge, two compact hands wrapped round the grip, sleeves running off the
//! corners, and a short red flash on the frames that fire. Keeping the gun small is what makes it
//! read as a weapon held at arm's length instead of a prop pasted over the view.
use super::{sprites::WeaponKind, Atlas, Canvas};

/// Weapons share the 64 pixel cell size of the other sprites
pub const WEAPON_CELL: i32 = 64;

const STEEL_DARK: [u8; 4] = [22, 24, 30, 255];
const STEEL: [u8; 4] = [74, 78, 88, 255];
const STEEL_LIT: [u8; 4] = [126, 132, 144, 255];
const STEEL_HI: [u8; 4] = [206, 212, 222, 255];
const BORE: [u8; 4] = [10, 10, 14, 255];

const SKIN: [u8; 4] = [214, 154, 110, 255];
const SKIN_LIT: [u8; 4] = [244, 194, 150, 255];
const SKIN_DEEP: [u8; 4] = [112, 66, 42, 255];

const SLEEVE: [u8; 4] = [78, 88, 66, 255];
const SLEEVE_LIT: [u8; 4] = [112, 124, 94, 255];
const CUFF: [u8; 4] = [52, 42, 32, 255];

const WOOD: [u8; 4] = [92, 58, 34, 255];
const WOOD_LIT: [u8; 4] = [136, 92, 52, 255];
const BRASS: [u8; 4] = [190, 152, 66, 255];
const FLASH: [u8; 4] = [255, 170, 60, 255];
const FLASH_HOT: [u8; 4] = [255, 246, 190, 255];
const FLASH_RIM: [u8; 4] = [206, 62, 32, 255];
const RIM: [u8; 4] = [12, 10, 14, 255];

fn shade(color: [u8; 4], delta: i32) -> [u8; 4] {
    let f = |v: u8| (v as i32 + delta).clamp(0, 255) as u8;
    [f(color[0]), f(color[1]), f(color[2]), color[3]]
}

fn tri(c: &mut Canvas, a: (i32, i32), b: (i32, i32), d: (i32, i32), color: [u8; 4]) {
    let min_x = a.0.min(b.0).min(d.0);
    let max_x = a.0.max(b.0).max(d.0);
    let min_y = a.1.min(b.1).min(d.1);
    let max_y = a.1.max(b.1).max(d.1);
    let edge = |p: (i32, i32), q: (i32, i32), x: i32, y: i32| {
        (q.0 - p.0) as f32 * (y - p.1) as f32 - (q.1 - p.1) as f32 * (x - p.0) as f32
    };
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge(b, d, x, y);
            let w1 = edge(d, a, x, y);
            let w2 = edge(a, b, x, y);
            if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                c.set(x, y, color);
            }
        }
    }
}

fn quad(
    c: &mut Canvas,
    a: (i32, i32),
    b: (i32, i32),
    d: (i32, i32),
    e: (i32, i32),
    color: [u8; 4],
) {
    tri(c, a, b, d, color);
    tri(c, a, d, e, color);
}

fn limb(c: &mut Canvas, a: (i32, i32), b: (i32, i32), radius: i32, color: [u8; 4]) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs())).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.0 as f32 + (b.0 - a.0) as f32 * t;
        let y = a.1 as f32 + (b.1 - a.1) as f32 * t;
        c.circle(x as i32, y as i32, radius, color);
    }
}

/// A gunmetal box with a lit left edge and a shadowed right one, which is the light direction
/// the original sprites use
fn metal(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, base: [u8; 4]) {
    c.rect(x, y, w, h, base);
    c.vline(x, y, h, shade(base, 46));
    c.vline(x + 1, y, h, shade(base, 20));
    c.vline(x + w - 1, y, h, shade(base, -34));
    c.hline(x, y, w, shade(base, 30));
    c.hline(x, y + h - 1, w, shade(base, -40));
}

/// The muzzle seen from behind: steel ring, dark bore, lit top edge
fn muzzle(c: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32) {
    c.ellipse(cx, cy, rx, ry, STEEL_DARK);
    c.ellipse(cx, cy - 1, (rx - 1).max(1), (ry - 1).max(1), STEEL_LIT);
    c.ellipse(cx, cy, (rx - 2).max(1), (ry - 2).max(1), BORE);
}

fn forearm(c: &mut Canvas, shoulder: (i32, i32), wrist: (i32, i32), radius: i32) {
    limb(c, shoulder, wrist, radius, SLEEVE);
    limb(
        c,
        (shoulder.0 - radius / 2, shoulder.1),
        (wrist.0 - radius / 2, wrist.1),
        (radius / 2).max(1),
        SLEEVE_LIT,
    );
    c.circle(wrist.0, wrist.1, radius, CUFF);
}

/// The silhouette of a fist gripping a vertical grip, seen from the shooter's side: four
/// knuckles along the top and the mass of the hand below.
const HAND: [&str; 8] = [
    "...#.#.#.#.....",
    "..##.##.##.....",
    ".#############.",
    "##############.",
    "##############.",
    ".############..",
    "..##########...",
    "...########....",
];

/// Blits a silhouette with a one pixel dark rim and shading from the top down, which is how the
/// hands keep a readable shape without a pixel table per pose
fn blob(c: &mut Canvas, mask: &[&str], x: i32, y: i32, flip: bool, tone: i32) {
    let w = mask[0].len() as i32;
    let h = mask.len() as i32;
    let solid = |mx: i32, my: i32| -> bool {
        if mx < 0 || my < 0 || mx >= w || my >= h {
            return false;
        }
        let col = if flip { w - 1 - mx } else { mx } as usize;
        mask[my as usize].as_bytes()[col] == b'#'
    };
    let px_of = |mx: i32| x + if flip { w - 1 - mx } else { mx };

    for my in 0..h {
        for mx in 0..w {
            if !solid(mx, my) {
                continue;
            }
            let edge = !solid(mx - 1, my)
                || !solid(mx + 1, my)
                || !solid(mx, my - 1)
                || !solid(mx, my + 1);
            let px = px_of(mx);
            if edge {
                c.set(px, y + my, RIM);
                continue;
            }
            let color = if my == 0 { SKIN_LIT } else { SKIN };
            c.set(px, y + my, shade(color, tone));
        }
    }

    for mx in 0..w {
        if solid(mx, h - 1) {
            c.set(px_of(mx), y + h - 1, shade(SKIN_DEEP, tone));
        }
    }
}

/// One hand wrapped round a grip. `palm` is the top left of the fist. `side` is -1 for the left
/// hand and 1 for the right, which is the same fist mirrored.
fn hand(c: &mut Canvas, palm: (i32, i32), side: i32, reach: i32, tone: i32) {
    let (px, py) = palm;
    let flip = side < 0;
    blob(c, &HAND, px, py, flip, tone);

    // Thumb and fingertips reaching round the far side of the grip
    let inner = if flip { px + 11 } else { px + 2 };
    limb(
        c,
        (inner, py + 6),
        (reach, py + 7),
        1,
        shade(SKIN_DEEP, tone),
    );
    limb(c, (inner, py + 7), (reach, py + 8), 1, shade(SKIN, tone));
}

fn clear_at(c: &Canvas, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= c.width || y >= c.height {
        return true;
    }
    let off = ((c.oy + y) * c.stride + c.ox + x) as usize * 4;
    off + 4 > c.data.len() || c.data[off + 3] == 0
}

/// A dark rim round the silhouette, kept off the cell edges so the sprite does not get framed
fn outline(c: &mut Canvas) {
    let mut rim = Vec::new();
    for y in 1..c.height - 1 {
        for x in 1..c.width - 1 {
            if !clear_at(c, x, y) {
                continue;
            }
            if [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| !clear_at(c, x + dx, y + dy))
            {
                rim.push((x, y));
            }
        }
    }
    for (x, y) in rim {
        c.set(x, y, RIM);
    }
}

pub fn draw(c: &mut Canvas, kind: WeaponKind, frame: usize) {
    match kind {
        WeaponKind::Knife => knife(c, frame),
        WeaponKind::Pistol => pistol(c, frame),
        WeaponKind::MachineGun => machine_gun(c, frame),
        WeaponKind::Chaingun => chaingun(c, frame),
    }
    outline(c);
}

/// A knife in the right hand, blade up: it leans back on the windup and drives forward on the stab
fn knife(c: &mut Canvas, frame: usize) {
    let thrust = if frame > 0 { 5 } else { 0 };
    let (hx, hy) = (44, 48 - thrust);

    // Blade, angled up and to the right from the fist
    quad(
        c,
        (hx - 1, hy - 2),
        (hx + 5, hy - 2),
        (hx + 3, hy - 26),
        (hx + 1, hy - 26),
        STEEL,
    );
    quad(
        c,
        (hx - 1, hy - 2),
        (hx + 1, hy - 2),
        (hx, hy - 26),
        (hx + 1, hy - 26),
        STEEL_HI,
    );
    c.set(hx + 2, hy - 27, STEEL_LIT);
    c.vline(hx + 4, hy - 5, 3, STEEL_DARK);

    // Guard, then the fist closed round the handle
    c.hline(hx - 4, hy - 2, 12, BRASS);
    c.hline(hx - 3, hy - 1, 10, shade(BRASS, -60));
    c.rect(hx - 1, hy, 6, 9, WOOD);
    c.vline(hx, hy, 9, WOOD_LIT);
    forearm(c, (62, 70), (50, 56), 6);
    hand(c, (34, hy + 2), 1, hx - 8, 0);
    forearm(c, (2, 70), (16, 60), 7);
    hand(c, (18, hy + 6), -1, hx + 6, -18);

    if frame > 0 {
        for i in 0..3 {
            c.hline(
                hx - 2,
                hy - 18 + i * 3,
                6,
                [236, 242, 252, 180 - i as u8 * 50],
            );
        }
    }
}

/// The service pistol: a small black body held in both hands, front sight up, red flash on firing
fn pistol(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 2,
        2 => 4,
        _ => 0,
    };
    let top = 38 - recoil;
    let cx = 32;

    // Slide and frame
    metal(c, cx - 8, top, 16, 17, STEEL);
    c.vline(cx - 7, top + 1, 15, STEEL_LIT);
    c.hline(cx - 8, top, 16, STEEL_HI);
    c.rect(cx - 1, top - 3, 3, 3, STEEL_DARK);
    c.set(cx, top - 3, STEEL_LIT);
    c.rect(cx - 4, top + 7, 5, 4, BORE);
    c.hline(cx - 8, top + 16, 16, STEEL_LIT);

    // Grip running down between the hands
    c.rect(cx - 4, top + 17, 9, 8, WOOD);
    c.vline(cx - 4, top + 17, 8, WOOD_LIT);
    c.vline(cx + 4, top + 17, 8, STEEL_DARK);

    // Right hand on the grip, left hand braced under it, sleeves off the corners
    forearm(c, (68, 76), (52, 62), 6);
    hand(c, (31, 49), 1, 26, 0);
    forearm(c, (-4, 76), (12, 64), 7);
    hand(c, (18, 54), -1, 36, -18);

    if frame > 0 {
        flash(c, cx, top - 3, if frame == 1 { 9 } else { 6 });
    }
}

/// The machine gun: a short perforated shroud, and a wide flash when it fires
fn machine_gun(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 2,
        2 => 4,
        _ => 0,
    };
    let top = 30 - recoil;
    let cx = 31;

    metal(c, cx - 9, top, 18, 20, STEEL);
    for i in 0..3 {
        c.rect(cx - 6 + i * 5, top + 4, 2, 3, BORE);
        c.rect(cx - 6 + i * 5, top + 11, 2, 3, BORE);
    }
    muzzle(c, cx, top + 1, 8, 3);
    c.hline(cx - 9, top - 1, 18, STEEL_HI);
    metal(c, cx - 10, top + 20, 20, 10, shade(STEEL, -14));

    c.rect(cx - 3, top + 30, 7, 8, WOOD);
    c.vline(cx - 3, top + 30, 8, WOOD_LIT);

    forearm(c, (68, 76), (52, 60), 6);
    hand(c, (29, 47), 1, 25, 0);
    forearm(c, (-4, 76), (12, 62), 7);
    hand(c, (16, 52), -1, 35, -18);

    if frame > 0 {
        flash(c, cx, top - 2, if frame == 1 { 12 } else { 8 });
    }
}

/// The chaingun: four barrels in a spinning cluster above a boxy housing
fn chaingun(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 2,
        2 => 3,
        3 => 2,
        _ => 0,
    };
    let top = 24 - recoil;
    let cx = 31;
    let spin = frame as f32 * std::f32::consts::FRAC_PI_4;

    for i in 0..4 {
        let angle = spin + i as f32 * std::f32::consts::FRAC_PI_2;
        let bx = cx + (angle.cos() * 4.0) as i32;
        let by = top + 8 + (angle.sin() * 2.0) as i32;
        c.rect(bx - 2, by - 8, 4, 12, STEEL);
        c.vline(bx - 2, by - 8, 12, STEEL_LIT);
        c.vline(bx + 1, by - 8, 12, STEEL_DARK);
        muzzle(c, bx, by - 8, 2, 1);
    }
    c.ellipse(cx, top + 2, 8, 4, STEEL_DARK);
    c.ellipse(cx, top + 1, 7, 3, STEEL_LIT);

    metal(c, cx - 9, top + 12, 18, 8, STEEL_LIT);
    metal(c, cx - 10, top + 20, 20, 12, shade(STEEL, -10));
    for i in 0..2 {
        c.hline(cx - 7, top + 23 + i * 4, 14, STEEL_DARK);
    }
    c.rect(cx - 3, top + 22, 6, 6, STEEL_DARK);

    // Ammo belt feeding in from the left
    for i in 0..3 {
        c.rect(cx - 14, top + 21 + i * 4, 7, 2, BRASS);
    }

    c.rect(cx - 3, top + 32, 7, 8, WOOD);
    forearm(c, (68, 76), (52, 60), 6);
    hand(c, (29, 49), 1, 25, 0);
    forearm(c, (-4, 76), (12, 62), 7);
    hand(c, (16, 54), -1, 35, -18);

    if frame > 0 {
        flash(c, cx, top - 2, if frame == 1 { 11 } else { 7 });
    }
}

/// A short muzzle flash: hot core ringed in orange, with a red rim that reads at 320x200
fn flash(c: &mut Canvas, cx: i32, cy: i32, size: i32) {
    c.circle(cx, cy, size, FLASH_RIM);
    c.circle(cx, cy, (size - 1).max(1), FLASH);
    c.circle(cx, cy, (size / 2).max(1), FLASH_HOT);
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in 2..=size {
            c.set(cx + dx * step, cy + dy * step, FLASH);
        }
    }
    c.set(cx + 1, cy - size - 1, FLASH_RIM);
    c.set(cx - 1, cy + size + 1, FLASH_RIM);
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
                    // Sleeves run off the bottom corners, but nothing frames the cell
                    let on_edge = y == 0 || (y < cell * 2 / 3 && (x == 0 || x == cell - 1));
                    assert!(!on_edge, "cell {index} paints the edge at ({x}, {y})");
                }
            }

            assert!(painted > cell * cell / 16, "cell {index} is nearly empty");
        }
    }
}
