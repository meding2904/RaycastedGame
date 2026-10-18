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
    c.hline(cx - rx + 2, cy - ry + 1, (rx / 2).max(1), STEEL_HI);
}

/// A finger wrapping a grip: lit knuckle, shaded underside and a nail on the tip
fn finger(c: &mut Canvas, a: (i32, i32), b: (i32, i32), radius: i32) {
    limb(c, a, b, radius + 1, SKIN_DEEP);
    limb(c, a, b, radius, SKIN);
    c.hline(
        a.0.min(b.0),
        a.1.max(b.1) - radius / 2,
        (b.0 - a.0).abs().max(1),
        SKIN_LIT,
    );
    c.circle(b.0, b.1, (radius / 2).max(1), NAIL);
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
fn hand(c: &mut Canvas, palm: (i32, i32), side: i32, reach: i32, fingers: i32, tone: i32) {
    let (px, py) = palm;
    let skin = shade(SKIN, tone);
    let lit = shade(SKIN_LIT, tone);
    c.ellipse(px, py, 8, 9, SKIN_DEEP);
    c.ellipse(px, py - 1, 7, 8, skin);
    c.ellipse(px - side * 2, py - 3, 3, 3, lit);

    for i in 0..fingers {
        let y = py - 5 + i * 5;
        let extra = [0, 2, 0, -2][i as usize];
        finger(c, (px + side * 2, y), (reach - side * extra, y), 2);
    }

    // Thumb laid along the top of the grip
    limb(c, (px + side, py - 8), (reach + side * 5, py - 10), 2, skin);
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

    forearm(c, (104, 128), (92, 106), 10);
    hand(c, (78, 100), 1, 64, 4, 0);
    forearm(c, (24, 128), (38, 118), 11);
    hand(c, (42, 116), -1, 62, 3, -14);

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
    let top = 24 + recoil;
    let (cx, w) = (60, 24);

    // Slide seen from behind, with the rear sight and hammer at the near edge
    muzzle(c, cx, top + 2, 7, 4);
    metal(c, cx - w / 2, top + 4, w, 44, STEEL);
    c.hline(cx - w / 2 + 2, top + 4, w - 4, STEEL_HI);
    c.rect(cx - 7, top + 5, 4, 3, STEEL_DARK);
    c.rect(cx + 3, top + 5, 4, 3, STEEL_DARK);
    c.rect(cx - w / 2 + 3, top + 14, 8, 7, BORE);
    for i in 0..3 {
        c.vline(cx + 5 + i * 2, top + 30, 9, STEEL_DARK);
    }
    metal(c, cx - w / 2 + 2, top + 48, w - 4, 8, shade(STEEL, -18));
    c.rect(cx - 3, top + 44, 6, 5, STEEL_DARK);

    // Grip running down and back between the hands
    quad(
        c,
        (cx - 11, top + 54),
        (cx + 11, top + 54),
        (cx + 18, 122),
        (cx - 6, 122),
        WOOD,
    );
    quad(
        c,
        (cx - 11, top + 54),
        (cx - 6, top + 54),
        (cx - 2, 122),
        (cx - 6, 122),
        WOOD_LIT,
    );
    for row in 0..6 {
        c.set(cx - 3 + row / 2, top + 62 + row * 9, WOOD_DARK);
        c.set(cx + 7 - row / 2, top + 62 + row * 9, WOOD_DARK);
    }

    forearm(c, (112, 140), (96, 112), 10);
    hand(c, (82, 100), 1, 62, 4, 0);
    forearm(c, (12, 140), (28, 122), 11);
    hand(c, (36, 112), -1, 64, 3, -14);

    if frame > 0 {
        flash(c, cx, top - 8, if frame == 1 { 20 } else { 12 });
    }
}

/// The machine gun: perforated shroud, drum magazine and a long spray on the fire frames
fn machine_gun(c: &mut Canvas, frame: usize) {
    let recoil = match frame {
        1 => 6,
        2 => 11,
        _ => 0,
    };
    let top = 22 + recoil;
    let cx = 60;

    muzzle(c, cx, top + 2, 9, 4);
    metal(c, cx - 16, top + 4, 32, 28, STEEL);
    for i in 0..4 {
        c.ellipse(cx - 12 + i * 7, top + 13, 2, 3, BORE);
        c.ellipse(cx - 12 + i * 7, top + 22, 2, 3, BORE);
    }
    metal(c, cx - 19, top + 32, 38, 26, shade(STEEL, -12));
    c.rect(cx - 13, top + 40, 9, 6, BORE);
    for i in 0..3 {
        c.vline(cx - 15 + i * 3, top + 46, 8, STEEL_DARK);
    }
    c.hline(cx - 19, top + 32, 38, STEEL_LIT);

    // Drum magazine clamped to the left of the receiver
    c.circle(cx - 26, top + 60, 13, STEEL_DARK);
    c.circle(cx - 26, top + 58, 11, STEEL_LIT);
    c.circle(cx - 26, top + 58, 8, STEEL);
    c.circle(cx - 26, top + 58, 3, STEEL_LIT);

    // Wooden stock running down into the hands
    quad(
        c,
        (cx - 12, top + 58),
        (cx + 12, top + 58),
        (cx + 16, 128),
        (cx - 12, 128),
        WOOD,
    );
    quad(
        c,
        (cx - 12, top + 58),
        (cx - 6, top + 58),
        (cx - 4, 128),
        (cx - 12, 128),
        WOOD_LIT,
    );

    forearm(c, (112, 140), (96, 112), 10);
    hand(c, (82, 104), 1, 60, 4, 0);
    forearm(c, (12, 140), (28, 122), 11);
    hand(c, (36, 116), -1, 62, 3, -14);

    if frame > 0 {
        flash(c, cx, top - 9, if frame == 1 { 24 } else { 14 });
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
    let top = 20 + recoil;
    let cx = 60;
    let spin = frame as f32 * std::f32::consts::FRAC_PI_4;

    // Four barrels turning inside a collar
    for i in 0..4 {
        let angle = spin + i as f32 * std::f32::consts::FRAC_PI_2;
        let bx = cx + (angle.cos() * 8.0) as i32;
        let by = top + 10 + (angle.sin() * 4.0) as i32;
        c.rect(bx - 4, by - 10, 8, 26, STEEL);
        c.vline(bx - 4, by - 10, 26, STEEL_LIT);
        c.vline(bx + 3, by - 10, 26, STEEL_DARK);
        muzzle(c, bx, by - 10, 5, 3);
    }
    c.ellipse(cx, top + 5, 16, 7, STEEL_DARK);
    c.ellipse(cx, top + 4, 14, 6, STEEL_LIT);

    metal(c, cx - 19, top + 26, 38, 10, STEEL_LIT);
    for i in 0..3 {
        c.rect(cx - 14 + i * 10, top + 29, 4, 4, STEEL_DARK);
    }
    metal(c, cx - 21, top + 36, 42, 30, shade(STEEL, -8));
    for i in 0..3 {
        c.hline(cx - 19, top + 42 + i * 9, 38, STEEL_DARK);
        c.hline(cx - 19, top + 43 + i * 9, 38, shade(STEEL, 20));
    }
    metal(c, cx - 10, top + 42, 20, 18, STEEL_LIT);
    c.rect(cx - 4, top + 47, 8, 8, STEEL_DARK);

    // Ammo belt feeding in from the left
    metal(c, cx - 34, top + 46, 12, 22, shade(STEEL, -20));
    for i in 0..4 {
        c.rect(cx - 38, top + 48 + i * 5, 14, 3, BRASS);
        c.hline(cx - 38, top + 48 + i * 5, 14, shade(BRASS, 44));
    }

    quad(
        c,
        (cx - 8, top + 64),
        (cx + 10, top + 64),
        (cx + 14, 128),
        (cx - 8, 128),
        WOOD,
    );

    forearm(c, (112, 140), (96, 112), 10);
    hand(c, (82, 106), 1, 60, 4, 0);
    forearm(c, (12, 140), (28, 122), 11);
    hand(c, (36, 118), -1, 62, 3, -14);

    if frame > 0 {
        flash(c, cx, top - 6, if frame == 1 { 22 } else { 14 });
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

            assert!(painted > cell * cell / 8, "cell {index} is nearly empty");
        }
    }
}
