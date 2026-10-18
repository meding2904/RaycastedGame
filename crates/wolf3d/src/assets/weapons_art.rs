//! The weapon in the player's hands, drawn at 128x128 and blitted into the bottom of the view.
use super::{sprites::WeaponKind, Atlas, Canvas};

/// Weapons are drawn at this size, unlike the 64 pixel actor cells
pub const WEAPON_CELL: i32 = 128;

const STEEL_DARK: [u8; 4] = [26, 28, 34, 255];
const STEEL: [u8; 4] = [80, 84, 94, 255];
const STEEL_LIT: [u8; 4] = [142, 148, 160, 255];
const STEEL_HI: [u8; 4] = [214, 220, 230, 255];
const BORE: [u8; 4] = [12, 12, 16, 255];

const SKIN: [u8; 4] = [206, 148, 106, 255];
const SKIN_LIT: [u8; 4] = [240, 190, 148, 255];
const SKIN_DEEP: [u8; 4] = [104, 60, 38, 255];
const NAIL: [u8; 4] = [228, 194, 170, 255];

const SLEEVE: [u8; 4] = [70, 80, 50, 255];
const SLEEVE_LIT: [u8; 4] = [106, 120, 72, 255];
const CUFF: [u8; 4] = [56, 42, 28, 255];
const CUFF_LIT: [u8; 4] = [96, 76, 50, 255];

const WOOD: [u8; 4] = [96, 60, 32, 255];
const WOOD_LIT: [u8; 4] = [144, 98, 54, 255];
const WOOD_DARK: [u8; 4] = [58, 34, 18, 255];
const BRASS: [u8; 4] = [188, 150, 64, 255];
const FLASH: [u8; 4] = [255, 226, 140, 255];
const FLASH_HOT: [u8; 4] = [255, 255, 244, 255];
const SMOKE: [u8; 4] = [120, 112, 104, 190];
const RIM: [u8; 4] = [16, 14, 18, 255];

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

/// A stamped thick line, used for fingers, thumbs and sleeves
fn limb(c: &mut Canvas, a: (i32, i32), b: (i32, i32), radius: i32, color: [u8; 4]) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs())).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.0 as f32 + (b.0 - a.0) as f32 * t;
        let y = a.1 as f32 + (b.1 - a.1) as f32 * t;
        c.circle(x as i32, y as i32, radius, color);
    }
}

fn metal(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, base: [u8; 4]) {
    c.rect(x, y, w, h, base);
    c.hline(x + 1, y, (w - 2).max(1), shade(base, 52));
    c.hline(x, y + h - 1, w, shade(base, -30));
    c.vline(x, y + 1, (h - 2).max(1), shade(base, 16));
    c.vline(x + w - 1, y + 1, (h - 2).max(1), shade(base, -20));
}

/// A barrel seen from behind: cylinder, dark bore, hot rim
fn muzzle(c: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32) {
    c.ellipse(cx, cy, rx, ry, STEEL_DARK);
    c.ellipse(cx, cy - 1, (rx - 1).max(1), (ry - 1).max(1), STEEL_LIT);
    c.ellipse(cx, cy, (rx - 3).max(1), (ry - 2).max(1), STEEL_DARK);
    c.ellipse(cx, cy, (rx - 4).max(1), (ry - 3).max(1), BORE);
    c.hline(cx - (rx - 3), cy - ry + 1, (rx * 2 - 6).max(1), STEEL_HI);
}

/// A finger wrapping a grip: lit knuckle, shaded underside and a nail on the tip
fn finger(c: &mut Canvas, a: (i32, i32), b: (i32, i32), radius: i32) {
    limb(c, a, b, radius + 1, SKIN_DEEP);
    limb(c, a, b, radius, SKIN);
    limb(
        c,
        (a.0, a.1 - radius / 2),
        (b.0, b.1 - radius / 2),
        (radius / 2).max(1),
        SKIN_LIT,
    );
    c.circle(b.0, b.1 - 1, (radius / 2).max(1), NAIL);
}

/// A forearm with a cuff, coming in from the bottom edge
fn forearm(c: &mut Canvas, shoulder: (i32, i32), wrist: (i32, i32), radius: i32) {
    limb(c, shoulder, wrist, radius, SLEEVE);
    limb(
        c,
        (shoulder.0 - radius / 2, shoulder.1),
        (wrist.0 - radius / 2, wrist.1),
        (radius / 2).max(2),
        SLEEVE_LIT,
    );
    c.circle(wrist.0, wrist.1, radius, CUFF);
    c.circle(
        wrist.0 - radius / 3,
        wrist.1 - radius / 3,
        radius / 2,
        CUFF_LIT,
    );
}

/// One hand: a palm, a thumb laid along the gun and fingers curling round the grip.
/// `side` is -1 for a hand reaching in from the left, 1 for one from the right.
fn hand(c: &mut Canvas, palm: (i32, i32), side: i32, reach: i32, fingers: i32) {
    let (px, py) = palm;
    c.ellipse(px, py + 3, 9, 10, SKIN_DEEP);
    c.ellipse(px, py, 9, 10, SKIN);
    c.ellipse(px - side * 2, py - 3, 5, 5, SKIN_LIT);

    for i in 0..fingers {
        let y = py - 5 + i * 5;
        finger(c, (px + side * 4, y), (reach, y), 2);
    }

    // Thumb laid along the top of the grip
    limb(
        c,
        (px + side * 2, py - 8),
        (reach + side * 5, py - 10),
        3,
        SKIN,
    );
    c.circle(reach + side * 5, py - 10, 2, NAIL);
}

fn clear_at(c: &Canvas, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= c.width || y >= c.height {
        return true;
    }
    let off = ((c.oy + y) * c.stride + c.ox + x) as usize * 4;
    off + 4 > c.data.len() || c.data[off + 3] == 0
}

/// A one pixel dark rim round the silhouette, so the weapon separates from the wall behind it
fn outline(c: &mut Canvas) {
    let mut rim = Vec::new();
    for y in 0..c.height {
        for x in 0..c.width {
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

/// Draws one weapon frame into a `WEAPON_CELL` sized cell
pub fn draw(c: &mut Canvas, kind: WeaponKind, frame: usize) {
    match kind {
        WeaponKind::Knife => knife(c, frame),
        WeaponKind::Pistol => pistol(c, frame),
        WeaponKind::MachineGun => machine_gun(c, frame),
        WeaponKind::Chaingun => chaingun(c, frame),
    }
    outline(c);
}

/// A knife held in the right hand, leaning back on the windup and driving forward on the stab
fn knife(c: &mut Canvas, frame: usize) {
    let thrust = if frame > 0 { 10 } else { 0 };

    let tip = (58, 20 - thrust);
    let heel = (78, 74 - thrust);
    for i in 0..=(heel.1 - tip.1) {
        let t = i as f32 / (heel.1 - tip.1).max(1) as f32;
        let y = tip.1 + i;
        let left = tip.0 + (t * 8.0) as i32;
        let width = 4 + (t * 12.0) as i32;
        c.hline(left, y, width, STEEL);
        c.hline(left, y, (width / 3).max(1), STEEL_HI);
        c.hline(left + width - 2, y, 2, STEEL_DARK);
    }

    c.rect(tip.0 - 6, heel.1, 30, 5, BRASS);
    c.hline(tip.0 - 6, heel.1, 30, shade(BRASS, 44));
    c.hline(tip.0 - 6, heel.1 + 4, 30, shade(BRASS, -58));

    quad(
        c,
        (heel.0 - 6, heel.1 + 5),
        (heel.0 + 8, heel.1 + 5),
        (heel.0 + 12, heel.1 + 34),
        (heel.0 - 2, heel.1 + 34),
        WOOD,
    );
    quad(
        c,
        (heel.0 - 6, heel.1 + 5),
        (heel.0 - 2, heel.1 + 5),
        (heel.0 + 2, heel.1 + 34),
        (heel.0 - 2, heel.1 + 34),
        WOOD_LIT,
    );

    forearm(c, (104, 128), (94, 108), 10);
    hand(c, (82, 94), 1, 64, 4);
    forearm(c, (24, 128), (36, 116), 11);
    hand(c, (44, 112), -1, 62, 3);

    if frame > 0 {
        for i in 0..3 {
            c.hline(
                tip.0 - 4,
                tip.1 + i * 4,
                9,
                [236, 242, 252, 170 - i as u8 * 50],
            );
        }
    }
}

/// The service pistol, held in both hands and pointing away from the camera
fn pistol(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 7,
        2 => 12,
        _ => 0,
    };
    let top = 26 + recoil;

    muzzle(c, 60, top, 9, 5);
    metal(c, 46, top + 3, 28, 44, STEEL);
    c.rect(46, top + 12, 9, 6, BORE);
    for i in 0..4 {
        c.vline(64 + i * 2, top + 10, 12, STEEL_DARK);
    }
    c.hline(46, top + 46, 28, STEEL_LIT);
    metal(c, 48, top + 47, 24, 10, shade(STEEL, -16));
    c.rect(54, top + 49, 12, 4, STEEL_LIT);

    // Grip, angled back to the right like a hand holding it
    quad(
        c,
        (52, top + 57),
        (70, top + 57),
        (78, top + 92),
        (58, top + 92),
        WOOD,
    );
    quad(
        c,
        (52, top + 57),
        (56, top + 57),
        (62, top + 92),
        (58, top + 92),
        WOOD_LIT,
    );
    quad(
        c,
        (66, top + 57),
        (70, top + 57),
        (78, top + 92),
        (74, top + 92),
        WOOD_DARK,
    );
    for row in 0..5 {
        let y = top + 64 + row * 6;
        for col in 0..3 {
            c.set(58 + col * 5 - row / 2, y, WOOD_DARK);
        }
    }

    forearm(c, (110, 128), (92, 106), 10);
    hand(c, (80, top + 78), 1, 60, 4);
    forearm(c, (18, 128), (36, 116), 11);
    hand(c, (42, top + 96), -1, 64, 3);

    if frame > 0 {
        flash(c, 60, top - 8, if frame == 1 { 20 } else { 12 });
    }
}

/// The machine gun: perforated shroud, drum magazine and a long spray on the fire frames
fn machine_gun(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 6,
        2 => 11,
        _ => 0,
    };
    let top = 24 + recoil;

    metal(c, 42, top, 36, 26, STEEL);
    for i in 0..4 {
        c.rect(46 + i * 8, top + 7, 5, 12, STEEL_DARK);
        c.hline(46 + i * 8, top + 7, 5, shade(STEEL_DARK, 34));
    }
    muzzle(c, 60, top + 1, 12, 5);
    metal(c, 40, top + 26, 40, 26, shade(STEEL, -10));
    c.rect(46, top + 34, 10, 6, BORE);
    for i in 0..3 {
        c.vline(44 + i * 3, top + 40, 8, STEEL_DARK);
    }

    c.circle(34, top + 62, 14, STEEL_DARK);
    c.circle(34, top + 60, 12, STEEL_LIT);
    c.circle(34, top + 60, 9, STEEL);
    c.circle(34, top + 60, 4, STEEL_LIT);
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        c.rect(
            34 + (angle.cos() * 8.0) as i32,
            top + 60 + (angle.sin() * 8.0) as i32,
            2,
            2,
            BRASS,
        );
    }

    quad(
        c,
        (64, top + 52),
        (80, top + 52),
        (84, 128),
        (62, 128),
        WOOD,
    );
    quad(
        c,
        (64, top + 52),
        (68, top + 52),
        (66, 128),
        (62, 128),
        WOOD_LIT,
    );

    forearm(c, (112, 128), (94, 108), 10);
    hand(c, (82, top + 70), 1, 62, 4);
    forearm(c, (16, 128), (34, 114), 11);
    hand(c, (44, top + 88), -1, 66, 3);

    if frame > 0 {
        flash(c, 60, top - 9, if frame == 1 { 24 } else { 14 });
    }
}

/// The chaingun: a cluster of barrels in a housing, spinning as it fires
fn chaingun(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 5,
        2 => 9,
        3 => 6,
        _ => 0,
    };
    let top = 22 + recoil;
    let spin = frame as f32 * std::f32::consts::FRAC_PI_4;

    for i in 0..4 {
        let angle = spin + i as f32 * std::f32::consts::FRAC_PI_2;
        let bx = 60 + (angle.cos() * 11.0) as i32;
        let by = top + 10 + (angle.sin() * 5.0) as i32;
        c.rect(bx - 4, by - 8, 8, 22, STEEL);
        c.vline(bx - 4, by - 8, 22, STEEL_LIT);
        c.vline(bx + 3, by - 8, 22, STEEL_DARK);
        muzzle(c, bx, by - 8, 5, 3);
    }
    metal(c, 40, top + 12, 40, 10, STEEL_LIT);
    for i in 0..3 {
        c.rect(46 + i * 11, top + 14, 4, 4, STEEL_DARK);
    }

    metal(c, 36, top + 22, 48, 32, shade(STEEL, -8));
    for i in 0..3 {
        c.hline(38, top + 28 + i * 9, 44, STEEL_DARK);
        c.hline(38, top + 29 + i * 9, 44, shade(STEEL, 20));
    }
    metal(c, 50, top + 28, 20, 18, STEEL_LIT);
    c.rect(56, top + 33, 8, 8, STEEL_DARK);

    // Ammo belt feeding in from the left
    metal(c, 26, top + 34, 12, 22, shade(STEEL, -20));
    for i in 0..4 {
        c.rect(22, top + 36 + i * 5, 14, 3, BRASS);
        c.hline(22, top + 36 + i * 5, 14, shade(BRASS, 44));
    }

    quad(
        c,
        (62, top + 52),
        (78, top + 52),
        (82, 128),
        (60, 128),
        WOOD,
    );

    forearm(c, (112, 128), (94, 108), 10);
    hand(c, (80, top + 74), 1, 60, 4);
    forearm(c, (16, 128), (32, 114), 11);
    hand(c, (42, top + 92), -1, 64, 3);

    if frame > 0 {
        flash(c, 60, top - 4, if frame == 1 { 22 } else { 14 });
    }
}

/// A muzzle flash: hot core, star of flame and a puff of smoke either side
fn flash(c: &mut Canvas, cx: i32, cy: i32, size: i32) {
    for (dx, dy) in [
        (0, -1),
        (1, -1),
        (-1, -1),
        (1, 0),
        (-1, 0),
        (0, 1),
        (1, 1),
        (-1, 1),
    ] {
        let len = if dx != 0 && dy != 0 { size / 2 } else { size };
        for step in 1..=len {
            let w = ((len - step) / 3 + 1).max(1);
            c.rect(cx + dx * step - w / 2, cy + dy * step - w / 2, w, w, FLASH);
        }
    }
    c.circle(cx, cy, (size / 3).max(2), FLASH);
    c.circle(cx, cy, (size / 5).max(1), FLASH_HOT);
    c.ellipse(cx - size, cy - size / 2, size / 2, size / 3, SMOKE);
    c.ellipse(cx + size, cy - size / 2, size / 2, size / 3, SMOKE);
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
