//! Procedural sprites: enemies in eight rotations, pickups, decorations and weapons.
use super::textures::CELL;
use super::{cc0_image, Atlas, Canvas, Rng};

/// Rotations per actor, matching the eight headings relative to the player. 0 is the actor
pub const VIEWS: usize = 8;

/// Animation slots inside one rotation.
pub const SLOT_STAND: usize = 0;
pub const SLOT_WALK: usize = 1; // 1..=4
/// Raise, aim, fire, recover: the attack has a windup before the shot lands
pub const SLOT_SHOOT: usize = 5; // 5..=8
pub const SLOT_PAIN: usize = 9;
pub const SLOT_DIE: usize = 10; // 10..=13
pub const SLOTS: usize = 14;

const WALK_FRAMES: usize = 4;
const DIE_FRAMES: usize = 4;
pub const SHOOT_FRAMES: usize = 4;
/// The frame of the attack animation on which the shot is actually fired
pub const FIRE_FRAME: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActorKind {
    Guard,
    Ss,
    Officer,
    Dog,
    Boss,
}

impl ActorKind {
    pub const ALL: [ActorKind; 5] = [
        ActorKind::Guard,
        ActorKind::Ss,
        ActorKind::Officer,
        ActorKind::Dog,
        ActorKind::Boss,
    ];

    pub fn index(self) -> usize {
        ActorKind::ALL.iter().position(|k| *k == self).unwrap_or(0)
    }

    /// Names in the original game's roster, minus the political iconography
    pub fn name(self) -> &'static str {
        match self {
            ActorKind::Guard => "guard",
            ActorKind::Ss => "elite guard",
            ActorKind::Officer => "officer",
            ActorKind::Dog => "guard dog",
            ActorKind::Boss => "boss",
        }
    }
}

struct Palette {
    uniform: [u8; 4],
    uniform_dark: [u8; 4],
    uniform_lit: [u8; 4],
    skin: [u8; 4],
    skin_lit: [u8; 4],
    helmet: [u8; 4],
    boots: [u8; 4],
    accent: [u8; 4],
    gun: [u8; 4],
    gun_lit: [u8; 4],
}

fn shade(color: [u8; 4], delta: i32) -> [u8; 4] {
    let channel = |value: u8| (value as i32 + delta).clamp(0, 255) as u8;
    [
        channel(color[0]),
        channel(color[1]),
        channel(color[2]),
        color[3],
    ]
}

fn palette(kind: ActorKind) -> Palette {
    match kind {
        ActorKind::Guard => Palette {
            uniform: [126, 88, 52, 255],
            uniform_dark: [88, 60, 34, 255],
            uniform_lit: [162, 118, 74, 255],
            skin: [222, 174, 136, 255],
            skin_lit: [242, 200, 166, 255],
            helmet: [92, 64, 38, 255],
            boots: [58, 42, 28, 255],
            accent: [154, 116, 66, 255],
            gun: [58, 60, 66, 255],
            gun_lit: [112, 116, 126, 255],
        },
        ActorKind::Ss => Palette {
            uniform: [62, 78, 118, 255],
            uniform_dark: [38, 50, 82, 255],
            uniform_lit: [92, 112, 158, 255],
            skin: [222, 174, 136, 255],
            skin_lit: [242, 200, 166, 255],
            helmet: [40, 42, 48, 255],
            boots: [26, 26, 30, 255],
            accent: [128, 146, 196, 255],
            gun: [48, 50, 56, 255],
            gun_lit: [104, 108, 120, 255],
        },
        ActorKind::Officer => Palette {
            uniform: [188, 186, 176, 255],
            uniform_dark: [138, 136, 128, 255],
            uniform_lit: [216, 214, 204, 255],
            skin: [226, 180, 142, 255],
            skin_lit: [246, 206, 172, 255],
            helmet: [64, 62, 62, 255],
            boots: [48, 44, 40, 255],
            accent: [212, 178, 78, 255],
            gun: [58, 60, 66, 255],
            gun_lit: [112, 116, 126, 255],
        },
        ActorKind::Dog => Palette {
            uniform: [104, 74, 46, 255],
            uniform_dark: [72, 50, 30, 255],
            uniform_lit: [138, 104, 68, 255],
            skin: [148, 114, 80, 255],
            skin_lit: [176, 140, 102, 255],
            helmet: [40, 30, 22, 255],
            boots: [40, 32, 24, 255],
            accent: [196, 176, 148, 255],
            gun: [58, 60, 66, 255],
            gun_lit: [112, 116, 126, 255],
        },
        ActorKind::Boss => Palette {
            uniform: [82, 74, 66, 255],
            uniform_dark: [54, 48, 44, 255],
            uniform_lit: [114, 104, 92, 255],
            skin: [230, 184, 146, 255],
            skin_lit: [248, 210, 176, 255],
            helmet: [48, 46, 44, 255],
            boots: [34, 32, 30, 255],
            accent: [198, 166, 88, 255],
            gun: [52, 52, 56, 255],
            gun_lit: [104, 108, 118, 255],
        },
    }
}

/// All enemy cells: one `SLOTS`-long run of rotations per actor kind.
pub struct ActorSprites {
    pub atlas: Atlas,
    base: [usize; 5],
}

impl ActorSprites {
    pub fn generate() -> Self {
        let mut atlas = Atlas::from_cells(ActorKind::ALL.len() * VIEWS * SLOTS, CELL);
        let mut base = [0usize; 5];
        let mut rng = Rng::new(0x5EED_0F51);

        for kind in ActorKind::ALL {
            base[kind.index()] = kind.index() * VIEWS * SLOTS;
            let p = palette(kind);

            for view in 0..VIEWS {
                for slot in 0..SLOTS {
                    let index = base[kind.index()] + view * SLOTS + slot;

                    if let Some((data, w, h)) =
                        cc0_image(&format!("{}_{view}_{slot}", kind.name().replace(' ', "_")))
                    {
                        atlas.blit(index, &data, w, h, false);
                        continue;
                    }

                    let mut c = atlas.canvas(index);
                    match kind {
                        ActorKind::Dog => draw_dog(&mut c, &mut rng, &p, view, slot),
                        _ => draw_soldier(&mut c, &mut rng, &p, kind, view, slot),
                    }
                    outline(&mut c);
                }
            }
        }

        ActorSprites { atlas, base }
    }

    pub fn cell(&self, kind: ActorKind, view: usize, slot: usize) -> usize {
        self.base[kind.index()] + (view % VIEWS) * SLOTS + slot.min(SLOTS - 1)
    }

    pub fn tex_rect(&self, cell: usize) -> (usize, usize, usize, usize) {
        self.atlas.tex_rect(cell)
    }

    /// Which rotation of the eight to draw, given the actor's heading and the camera's
    pub fn view_for(actor_angle: f32, camera_angle: f32) -> usize {
        use std::f32::consts::TAU;
        let relative = (actor_angle - camera_angle - std::f32::consts::PI).rem_euclid(TAU);
        let view = (relative / TAU * VIEWS as f32).round() as usize;
        view % VIEWS
    }

    /// The animation slot and the walk phase for a walk cycle frame counter
    pub fn walk_slot(frame: usize) -> usize {
        SLOT_WALK + frame % WALK_FRAMES
    }

    pub fn die_slot(frame: usize) -> usize {
        SLOT_DIE + frame.min(DIE_FRAMES - 1)
    }
}

struct Body {
    turn: f32,
    facing: f32,
    width: i32,
    head_dx: i32,
    bob: i32,
}

fn body_for(view: usize) -> Body {
    use std::f32::consts::TAU;
    let phase = view as f32 / VIEWS as f32 * TAU;
    let turn = phase.sin();
    let facing = phase.cos();

    Body {
        turn,
        facing,
        width: (26.0 - 11.0 * turn.abs()) as i32,
        head_dx: (turn * 5.0) as i32,
        bob: 0,
    }
}

/// Fills a box with a lit top-left edge and a shadowed bottom-right edge
fn block(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, base: [u8; 4]) {
    if w <= 0 || h <= 0 {
        return;
    }
    c.rect(x, y, w, h, base);
    c.hline(x, y, w, shade(base, 34));
    c.vline(x, y, h, shade(base, 22));
    if h > 2 {
        c.hline(x, y + h - 1, w, shade(base, -34));
    }
    if w > 2 {
        c.vline(x + w - 1, y, h, shade(base, -26));
    }
}

/// A dark rim around everything drawn so far, which is what makes a sprite pop off a wall
fn outline(c: &mut Canvas) {
    let mut rim = Vec::new();
    for y in 0..CELL {
        for x in 0..CELL {
            if c.get(x, y)[3] > 8 {
                continue;
            }
            let touches = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| c.get(x + dx, y + dy)[3] > 150);
            if touches {
                rim.push((x, y));
            }
        }
    }
    for (x, y) in rim {
        c.set(x, y, [14, 11, 9, 235]);
    }
}

fn muzzle_flash(c: &mut Canvas, x: i32, y: i32) {
    c.circle(x, y, 4, [252, 236, 140, 255]);
    c.circle(x, y, 2, [255, 255, 224, 255]);
    for (dx, dy) in [(6, 0), (-6, 0), (0, 6), (0, -6), (4, 4), (-4, -4)] {
        c.set(x + dx, y + dy, [248, 208, 96, 255]);
    }
}

/// What the arms and weapon are doing on a given frame
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pose {
    Idle,
    /// Raising the weapon, 0 for the first frame of the attack
    Raise(usize),
    Pain,
}

/// Draws a line `thickness` pixels wide, which is how the arms and the raised weapon get drawn at
fn thick_line(c: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, thickness: i32, color: [u8; 4]) {
    let half = (thickness - 1).max(0) / 2;
    for offset in -half..=half {
        c.line(x0 + offset, y0, x1 + offset, y1, color);
    }
}

/// One soldier, from the front, the side or behind.
fn draw_soldier(
    c: &mut Canvas,
    _rng: &mut Rng,
    p: &Palette,
    kind: ActorKind,
    view: usize,
    slot: usize,
) {
    let mut b = body_for(view);

    if slot >= SLOT_DIE {
        draw_corpse(c, p, view, slot - SLOT_DIE);
        return;
    }

    let walk_phase = if (SLOT_WALK..SLOT_WALK + WALK_FRAMES).contains(&slot) {
        (slot - SLOT_WALK) as f32 / WALK_FRAMES as f32
    } else {
        0.0
    };
    b.bob = ((walk_phase * std::f32::consts::TAU).sin() * 2.0) as i32;

    let attack = if (SLOT_SHOOT..SLOT_SHOOT + SHOOT_FRAMES).contains(&slot) {
        Some(slot - SLOT_SHOOT)
    } else {
        None
    };
    let pose = match attack {
        Some(frame) => Pose::Raise(frame),
        None if slot == SLOT_PAIN => Pose::Pain,
        None => Pose::Idle,
    };
    let firing = pose == Pose::Raise(FIRE_FRAME);

    let scale = match kind {
        ActorKind::Boss => 1.15,
        _ => 1.0,
    };
    let s = |value: f32| (value * scale) as i32;

    let kick = if firing { -2 } else { 0 };
    let lean = match pose {
        Pose::Pain => (-b.turn * 4.0) as i32,
        Pose::Raise(_) => 1,
        Pose::Idle => 0,
    } + kick;

    let cx = 32 + lean;
    let foot_y = if kind == ActorKind::Boss { 62 } else { 60 };
    let leg_h = s(15.0);
    let torso_h = s(24.0);
    let head = s(13.0);
    let half = b.width / 2;
    let torso_y = foot_y - leg_h - torso_h + b.bob;
    let head_y = torso_y - head + 2 + kick;

    c.ellipse(
        cx,
        foot_y,
        (b.width as f32 * 0.62) as i32,
        3,
        [0, 0, 0, 100],
    );

    // --- Legs. The walk cycle swings them across the axis the body is seen from.
    let swing = (walk_phase * std::f32::consts::TAU).sin() * 5.0;
    let leg_w = s(9.0);
    let leg_dx = (b.turn * 4.0) as i32;
    for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
        let outward = (*side * b.width as f32 * 0.26) as i32;
        let step = if i == 0 { swing } else { -swing };
        let x = cx + outward + leg_dx + (step * b.turn.abs()) as i32;
        let y = foot_y - leg_h + (step.abs() * 0.5) as i32;

        block(c, x - leg_w / 2, y, leg_w, leg_h, p.uniform_dark);
        c.hline(
            x - leg_w / 2 + 1,
            y + 3,
            leg_w - 2,
            shade(p.uniform_dark, 18),
        );

        let boot_h = s(6.0);
        let boot_y = foot_y - boot_h;
        block(c, x - leg_w / 2 - 1, boot_y, leg_w + 2, boot_h, p.boots);
        c.hline(x - leg_w / 2 - 1, boot_y, leg_w + 2, shade(p.boots, 46));
        c.hline(
            x - leg_w / 2 - 1,
            foot_y - 1,
            leg_w + 2,
            shade(p.boots, -18),
        );
    }

    // --- Torso and shoulders
    block(c, cx - half, torso_y, b.width, torso_h, p.uniform);
    for side in [-1, 1] {
        let sx = cx + side * (half + 1) - 3;
        block(c, sx, torso_y + 1, 6, s(7.0), p.uniform_lit);
    }

    let belt_y = torso_y + torso_h - 4;
    c.hline(cx - half, belt_y, b.width, p.boots);
    c.rect(cx - 1, belt_y - 1, 3, 3, p.accent);
    c.hline(cx - half + 1, torso_y, b.width - 2, p.accent);

    match kind {
        ActorKind::Guard => {
            c.vline(cx - 3, torso_y + 4, torso_h - 10, p.uniform_dark);
            c.vline(cx + 3, torso_y + 4, torso_h - 10, p.uniform_dark);
            for side in [-1, 1] {
                let px = cx + side * 8 - 3;
                block(c, px, torso_y + 10, 7, 6, p.uniform_lit);
                c.hline(px, torso_y + 10, 7, p.uniform_dark);
            }
        }
        ActorKind::Ss => {
            for side in [-1, 1] {
                block(c, cx + side * (half - 3) - 3, torso_y + 1, 7, 3, p.accent);
            }
            c.hline(cx - 3, torso_y - 1, 7, p.uniform_dark);
            c.rect(cx - 1, torso_y + 6, 3, torso_h - 12, p.uniform_dark);
        }
        ActorKind::Officer => {
            for i in 0..4 {
                c.rect(cx - 1, torso_y + 5 + i * 4, 2, 2, p.accent);
            }
            for side in [-1, 1] {
                let x = cx + side * (half - 2) - 2;
                block(c, x, torso_y, 5, 3, p.accent);
            }
        }
        ActorKind::Boss => {
            let plate_h = torso_h - 10;
            block(
                c,
                cx - half + 2,
                torso_y + 3,
                b.width - 4,
                plate_h,
                p.uniform_lit,
            );
            c.frame(cx - half + 2, torso_y + 3, b.width - 4, plate_h, p.helmet);
            for i in 0..3 {
                c.rect(
                    cx - half + 4 + i * (b.width / 3),
                    torso_y + 5,
                    2,
                    2,
                    p.accent,
                );
            }
            c.hline(cx - half, torso_y, b.width, p.accent);
        }
        ActorKind::Dog => {}
    }

    // --- Head
    let head_x = cx + b.head_dx;
    block(c, head_x - head / 2, head_y, head, head, p.skin);
    c.hline(
        head_x - head / 2,
        head_y + head - 2,
        head,
        shade(p.skin, -30),
    );

    block(c, head_x - head / 2 - 1, head_y - 4, head + 2, 5, p.helmet);
    c.hline(
        head_x - head / 2 - 1,
        head_y - 4,
        head + 2,
        shade(p.helmet, 44),
    );
    match kind {
        ActorKind::Officer => {
            block(c, head_x - head / 2 - 3, head_y + 1, head + 6, 2, p.helmet);
            c.rect(head_x - head / 2 - 3, head_y + 2, head + 4, 1, p.accent);
        }
        ActorKind::Ss | ActorKind::Boss => {
            c.rect(head_x - head / 2, head_y + head - 4, head, 4, p.helmet);
            c.hline(
                head_x - head / 2,
                head_y + head - 4,
                head,
                shade(p.helmet, 30),
            );
        }
        _ => {
            c.rect(head_x - head / 2 - 2, head_y + 1, head + 4, 2, p.helmet);
        }
    }

    if b.facing > 0.3 {
        let eye_y = head_y + 5;
        let eye_sep = if b.facing > 0.75 { 3 } else { 1 };
        let eye = [26, 20, 16, 255];
        c.rect(head_x - 2 - eye_sep, eye_y, 2, 2, eye);
        c.rect(head_x + 1 + eye_sep, eye_y, 2, 2, eye);
        c.vline(head_x, eye_y + 3, 2, shade(p.skin, -40));
        c.hline(head_x - 2, eye_y + 5, 5, [116, 66, 56, 255]);
        if kind == ActorKind::Guard {
            c.rect(head_x - 3, eye_y + 4, 7, 1, [66, 44, 30, 255]);
        }
    }

    // --- Arms and weapon
    if b.facing > 0.55 {
        let gy = torso_y + torso_h - s(14.0) + kick;
        let shoulder_y = torso_y + s(5.0);

        let (gun_x, gun_y) = match pose {
            Pose::Raise(0) => (cx + 3, gy - 3),
            Pose::Raise(1) | Pose::Raise(FIRE_FRAME) => (cx, gy - 1),
            Pose::Raise(_) => (cx + 2, gy + 4),
            Pose::Pain => (cx - 3, gy + 5),
            Pose::Idle => (cx + 2, gy + 6),
        };

        for side in [-1, 1] {
            let shoulder = (cx + side * (half - 1), shoulder_y + 6);
            thick_line(
                c,
                shoulder.0,
                shoulder.1,
                gun_x + side * 7,
                gun_y + 3,
                s(5.0).max(4),
                if side > 0 { p.uniform } else { p.uniform_dark },
            );
        }

        block(c, gun_x - 6, gun_y, 13, 6, p.gun);
        c.hline(gun_x - 6, gun_y, 13, p.gun_lit);
        c.circle(gun_x, gun_y + 2, 2, p.gun);
        c.circle(gun_x, gun_y + 2, 1, [10, 9, 9, 255]);
        c.rect(gun_x - 5, gun_y - 2, 2, 2, p.gun_lit);
        block(c, gun_x - 9, gun_y + 3, 6, 5, p.skin);
        block(c, gun_x + 4, gun_y + 3, 6, 5, p.skin);

        if firing {
            muzzle_flash(c, gun_x, gun_y - 3);
        }
    } else {
        let dir = if b.turn >= 0.0 { 1 } else { -1 };
        let arm_w = s(6.0);
        let arm_h = s(14.0);
        let turn = b.turn.abs();
        let near_x = if dir > 0 {
            cx + half - arm_w / 2
        } else {
            cx - half - arm_w / 2
        };
        let far_x = if dir > 0 {
            cx - half + 1
        } else {
            cx + half - arm_w - 1
        };

        if turn < 0.92 {
            block(
                c,
                far_x,
                torso_y + 4 + b.bob,
                (arm_w as f32 * (1.0 - turn)) as i32 + 1,
                arm_h,
                p.uniform_dark,
            );
        }

        let raise_arm = match pose {
            Pose::Raise(0) => -4,
            Pose::Raise(1) | Pose::Raise(FIRE_FRAME) => -8,
            Pose::Raise(_) => -3,
            Pose::Pain => -5,
            Pose::Idle => 0,
        };
        let arm_y = torso_y + 4 + b.bob + raise_arm;
        block(c, near_x, arm_y, arm_w, arm_h, p.uniform);
        c.rect(near_x, arm_y + arm_h - 1, arm_w, 4, p.skin);

        let reach = match kind {
            ActorKind::Boss => s(26.0),
            ActorKind::Ss => s(21.0),
            _ => s(16.0),
        };
        let gun_x = if dir > 0 {
            near_x + arm_w - 1
        } else {
            near_x - reach + 1
        };
        let gun_y = arm_y + arm_h - 5;

        block(c, gun_x, gun_y, reach, 4, p.gun);
        c.hline(gun_x, gun_y, reach, p.gun_lit);
        c.rect(
            gun_x + if dir > 0 { 4 } else { reach - 7 },
            gun_y + 4,
            3,
            5,
            p.gun,
        );
        if kind == ActorKind::Boss {
            for i in 0..4 {
                c.rect(gun_x + reach / 2 - 6 + i * 4, gun_y + 4, 3, 2, p.accent);
            }
        }
        c.rect(
            gun_x + if dir > 0 { 2 } else { reach - 6 },
            gun_y + 4,
            4,
            4,
            p.skin,
        );

        if firing {
            let flash_x = if dir > 0 { gun_x + reach } else { gun_x - 1 };
            muzzle_flash(c, flash_x, gun_y + 1);
        }
    }
}

fn draw_corpse(c: &mut Canvas, p: &Palette, view: usize, frame: usize) {
    let b = body_for(view);
    let sink = frame.min(DIE_FRAMES - 1) as i32;

    let (h, w) = ((14 - sink * 3).max(4), 22 + sink * 3);
    let cx = 32;
    let y = 60 - h;

    c.ellipse(cx, 60, w / 2 + 3, 3, [0, 0, 0, 110]);
    c.ellipse(cx, y + h / 2, w / 2, (h / 2).max(2), p.uniform);
    c.ellipse(cx, y + h / 2 - 2, w / 2 - 2, (h / 3).max(2), p.uniform_lit);
    c.rect(cx - w / 2, y + h - 4, w, 4, p.uniform_dark);

    let head_x = cx + (b.turn * (w as f32 * 0.35)) as i32;
    c.circle(head_x, y + 2, 5, p.skin);
    c.circle(head_x, y + 1, 5, p.helmet);
    c.circle(head_x + 2, y + 2, 2, shade(p.skin, -30));

    c.ellipse(cx, 59, 6 + sink * 3, 2, [86, 16, 16, 170]);
}

fn draw_dog(c: &mut Canvas, _rng: &mut Rng, p: &Palette, view: usize, slot: usize) {
    let mut b = body_for(view);

    if slot >= SLOT_DIE {
        let frame = slot - SLOT_DIE;
        c.ellipse(32, 56, 16, 5, [0, 0, 0, 110]);
        c.ellipse(32, 52, 15 - frame as i32, 4, p.uniform);
        c.ellipse(32, 50, 12 - frame as i32, 2, p.uniform_lit);
        c.circle(46, 52, 5, p.uniform_dark);
        c.ellipse(32, 55, 8 + frame as i32 * 2, 2, [86, 16, 16, 170]);
        return;
    }

    let walk_phase = if (SLOT_WALK..SLOT_WALK + WALK_FRAMES).contains(&slot) {
        (slot - SLOT_WALK) as f32 / WALK_FRAMES as f32
    } else {
        0.0
    };
    let attack = if (SLOT_SHOOT..SLOT_SHOOT + SHOOT_FRAMES).contains(&slot) {
        Some(slot - SLOT_SHOOT)
    } else {
        None
    };
    let pain = slot == SLOT_PAIN;
    b.bob = ((walk_phase * std::f32::consts::TAU).sin() * 2.0) as i32;

    let cx = 32;
    let leap = match attack {
        Some(0) => -3,          // crouch
        Some(1) => -9,          // in the air
        Some(FIRE_FRAME) => -5, // jaws on the player
        Some(_) => -1,          // landing
        None => 0,
    };
    let push = match attack {
        Some(1) => 3,
        Some(FIRE_FRAME) => 6,
        Some(_) => 2,
        None => 0,
    };
    let body_y = 44 + b.bob + leap + if pain { 2 } else { 0 };

    c.ellipse(cx + push / 2, 60, 16, 3, [0, 0, 0, 95]);

    for (i, dx) in [-11, -6, 6, 11].iter().enumerate() {
        let step = ((walk_phase * std::f32::consts::TAU + i as f32).sin() * 2.0) as i32;
        let swing = match attack {
            Some(1) => -4,
            Some(FIRE_FRAME) => 2,
            _ => 0,
        };
        let x = cx + dx + push;
        block(c, x, body_y + 6, 4, 10 + step + swing, p.uniform_dark);
        c.rect(x, body_y + 14 + step + swing, 4, 2, p.boots);
    }

    c.ellipse(cx + push / 3, body_y + 4, 15, 7, p.uniform);
    c.ellipse(cx + push / 3, body_y + 1, 13, 4, p.uniform_dark);
    c.ellipse(cx + push / 3, body_y + 2, 11, 2, p.uniform_lit);

    let head_x = cx + (b.turn * 16.0) as i32 + 10 + push;
    let head_y = body_y - 2 + if pain { -2 } else { 0 };
    c.circle(head_x, head_y, 7, p.skin);
    c.circle(head_x + 2, head_y - 2, 5, p.skin_lit);
    c.ellipse(head_x + 5, head_y + 1, 4, 3, p.uniform_dark);

    c.rect(head_x + 8, head_y + 1, 5, 3, p.helmet);
    c.rect(head_x + 12, head_y + 1, 1, 3, [20, 16, 14, 255]);
    c.ellipse(head_x - 1, head_y - 6, 2, 3, p.uniform_dark);
    c.ellipse(head_x + 2, head_y - 6, 2, 3, p.uniform_dark);
    if b.facing > 0.2 {
        c.rect(head_x + 5, head_y - 2, 2, 2, [240, 60, 40, 255]);
        c.set(head_x + 5, head_y - 2, [250, 120, 90, 255]);
    }
    c.hline(head_x - 5, head_y + 5, 7, p.accent);

    let tail = ((walk_phase * 6.0).sin() * 4.0) as i32;
    block(c, cx - 18 + push, body_y - 2 + tail, 6, 3, p.uniform_dark);

    if attack == Some(FIRE_FRAME) {
        c.rect(head_x + 8, head_y + 4, 7, 2, [236, 236, 232, 255]);
        c.rect(head_x + 9, head_y, 6, 1, [236, 236, 232, 255]);
        c.rect(head_x + 8, head_y + 2, 7, 2, [92, 24, 24, 255]);
        c.circle(head_x + 14, head_y + 3, 1, [250, 250, 250, 255]);
    }
}

/// --- Objects: pickups and decorations -------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectKind {
    HealthSmall,
    HealthLarge,
    AmmoBox,
    AmmoClip,
    TreasureCross,
    TreasureChalice,
    TreasureChest,
    TreasureCrown,
    KeyGold,
    KeySilver,
    ExtraLife,
    Barrel,
    Table,
    Pillar,
    Lamp,
    Chandelier,
    Plant,
    Puddle,
}

impl ObjectKind {
    pub const ALL: [ObjectKind; 18] = [
        ObjectKind::HealthSmall,
        ObjectKind::HealthLarge,
        ObjectKind::AmmoBox,
        ObjectKind::AmmoClip,
        ObjectKind::TreasureCross,
        ObjectKind::TreasureChalice,
        ObjectKind::TreasureChest,
        ObjectKind::TreasureCrown,
        ObjectKind::KeyGold,
        ObjectKind::KeySilver,
        ObjectKind::ExtraLife,
        ObjectKind::Barrel,
        ObjectKind::Table,
        ObjectKind::Pillar,
        ObjectKind::Lamp,
        ObjectKind::Chandelier,
        ObjectKind::Plant,
        ObjectKind::Puddle,
    ];

    pub fn index(self) -> usize {
        ObjectKind::ALL.iter().position(|k| *k == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            ObjectKind::HealthSmall => "health_small",
            ObjectKind::HealthLarge => "health_large",
            ObjectKind::AmmoBox => "ammo_box",
            ObjectKind::AmmoClip => "ammo_clip",
            ObjectKind::TreasureCross => "treasure_cross",
            ObjectKind::TreasureChalice => "treasure_chalice",
            ObjectKind::TreasureChest => "treasure_chest",
            ObjectKind::TreasureCrown => "treasure_crown",
            ObjectKind::KeyGold => "key_gold",
            ObjectKind::KeySilver => "key_silver",
            ObjectKind::ExtraLife => "extra_life",
            ObjectKind::Barrel => "barrel",
            ObjectKind::Table => "table",
            ObjectKind::Pillar => "pillar",
            ObjectKind::Lamp => "lamp",
            ObjectKind::Chandelier => "chandelier",
            ObjectKind::Plant => "plant",
            ObjectKind::Puddle => "puddle",
        }
    }

    /// Decorations never block the player, items are picked up on touch
    pub fn is_decoration(self) -> bool {
        matches!(
            self,
            ObjectKind::Barrel
                | ObjectKind::Table
                | ObjectKind::Pillar
                | ObjectKind::Lamp
                | ObjectKind::Chandelier
                | ObjectKind::Plant
                | ObjectKind::Puddle
        )
    }

    /// Only pillars and barrels are solid, like the original's blocking decorations
    pub fn is_solid(self) -> bool {
        matches!(
            self,
            ObjectKind::Barrel | ObjectKind::Table | ObjectKind::Pillar
        )
    }
}

pub struct ObjectSprites {
    pub atlas: Atlas,
}

impl ObjectSprites {
    pub fn generate() -> Self {
        let mut atlas = Atlas::from_cells(ObjectKind::ALL.len(), CELL);

        for kind in ObjectKind::ALL {
            let index = kind.index();
            if let Some((data, w, h)) = cc0_image(kind.name()) {
                atlas.blit(index, &data, w, h, false);
                continue;
            }
            let mut rng = Rng::new(0x0BEE_0000 + index as u64);
            let mut c = atlas.canvas(index);
            match kind {
                ObjectKind::HealthSmall => medkit(&mut c, 18, 14, false),
                ObjectKind::HealthLarge => medkit(&mut c, 24, 20, true),
                ObjectKind::AmmoBox => ammo_box(&mut c, &mut rng),
                ObjectKind::AmmoClip => ammo_clip(&mut c),
                ObjectKind::TreasureCross => treasure_cross(&mut c),
                ObjectKind::TreasureChalice => chalice(&mut c),
                ObjectKind::TreasureChest => chest(&mut c, &mut rng),
                ObjectKind::TreasureCrown => crown(&mut c),
                ObjectKind::KeyGold => key(&mut c, [212, 176, 68, 255]),
                ObjectKind::KeySilver => key(&mut c, [198, 202, 210, 255]),
                ObjectKind::ExtraLife => extra_life(&mut c),
                ObjectKind::Barrel => barrel(&mut c, &mut rng),
                ObjectKind::Table => table(&mut c, &mut rng),
                ObjectKind::Pillar => pillar(&mut c, &mut rng),
                ObjectKind::Lamp => lamp(&mut c),
                ObjectKind::Chandelier => chandelier(&mut c),
                ObjectKind::Plant => plant(&mut c, &mut rng),
                ObjectKind::Puddle => puddle(&mut c, &mut rng),
            }
        }

        ObjectSprites { atlas }
    }

    pub fn tex_rect(&self, kind: ObjectKind) -> (usize, usize, usize, usize) {
        self.atlas.tex_rect(kind.index())
    }
}

fn medkit(c: &mut Canvas, w: i32, h: i32, large: bool) {
    let x = 32 - w / 2;
    let y = 60 - h;
    c.ellipse(32, 60, w / 2 + 2, 3, [0, 0, 0, 90]);
    c.rect(x, y, w, h, [232, 232, 226, 255]);
    c.hline(x, y, w, [255, 255, 255, 255]);
    c.hline(x, y + h - 1, w, [172, 172, 166, 255]);
    c.vline(x + w - 1, y, h, [172, 172, 166, 255]);
    let arm = if large { 3 } else { 2 };
    c.rect(32 - arm / 2, y + 3, arm, h - 6, [196, 40, 40, 255]);
    c.rect(x + 3, 60 - h / 2 - arm / 2, w - 6, arm, [196, 40, 40, 255]);
    if large {
        c.rect(28, y + 2, 8, 2, [140, 140, 136, 255]);
    }
}

fn ammo_box(c: &mut Canvas, rng: &mut Rng) {
    let x = 20;
    let y = 44;
    c.ellipse(32, 60, 14, 3, [0, 0, 0, 90]);
    c.rect(x, y, 24, 16, [120, 92, 48, 255]);
    c.hline(x, y, 24, [156, 122, 68, 255]);
    c.hline(x, y + 15, 24, [78, 58, 30, 255]);
    c.rect(x + 8, y - 3, 8, 3, [150, 116, 62, 255]);
    for i in 0..4 {
        c.rect(x + 2 + i * 6, y + 5, 4, 6, [196, 172, 96, 255]);
        c.set(x + 2 + i * 6, y + 5, [240, 224, 160, 255]);
    }
    c.noise(rng, 0.08, true);
}

fn ammo_clip(c: &mut Canvas) {
    c.ellipse(32, 60, 8, 3, [0, 0, 0, 90]);
    c.rect(26, 50, 12, 10, [96, 82, 54, 255]);
    for i in 0..3 {
        c.rect(27 + i * 4, 44, 3, 8, [200, 168, 84, 255]);
        c.rect(27 + i * 4, 44, 3, 2, [248, 226, 150, 255]);
    }
}

fn treasure_cross(c: &mut Canvas) {
    let gold = [206, 168, 66, 255];
    let gold_hi = [248, 226, 150, 255];
    c.ellipse(32, 60, 8, 3, [0, 0, 0, 90]);
    c.rect(30, 26, 5, 32, gold);
    c.rect(20, 34, 25, 5, gold);
    c.vline(30, 26, 32, gold_hi);
    c.hline(20, 34, 25, gold_hi);
    c.rect(28, 56, 9, 4, [150, 118, 46, 255]);
}

fn chalice(c: &mut Canvas) {
    let gold = [204, 164, 60, 255];
    c.ellipse(32, 60, 10, 3, [0, 0, 0, 90]);
    c.ellipse(32, 34, 11, 7, gold);
    c.rect(30, 38, 5, 14, gold);
    c.ellipse(32, 55, 10, 4, gold);
    c.hline(22, 30, 20, [246, 222, 148, 255]);
    c.vline(28, 32, 8, [246, 222, 148, 255]);
}

fn chest(c: &mut Canvas, rng: &mut Rng) {
    let wood = [122, 70, 32, 255];
    c.ellipse(32, 60, 16, 3, [0, 0, 0, 90]);
    c.rect(16, 40, 32, 20, wood);
    c.rect(16, 36, 32, 6, [148, 92, 44, 255]);
    c.hline(16, 36, 32, [178, 118, 60, 255]);
    c.rect(16, 42, 32, 3, [206, 168, 66, 255]);
    c.rect(29, 44, 6, 8, [206, 168, 66, 255]);
    c.rect(31, 47, 2, 4, [96, 74, 30, 255]);
    c.vline(24, 40, 20, [86, 48, 22, 255]);
    c.vline(40, 40, 20, [86, 48, 22, 255]);
    c.noise(rng, 0.07, true);
}

fn crown(c: &mut Canvas) {
    let gold = [212, 172, 62, 255];
    let jewel = [188, 40, 52, 255];
    c.ellipse(32, 60, 14, 3, [0, 0, 0, 90]);
    c.rect(16, 46, 32, 12, gold);
    c.rect(16, 50, 32, 2, [248, 224, 142, 255]);
    for i in 0..5 {
        let x = 17 + i * 7;
        c.rect(x, 34, 5, 12, gold);
        c.circle(x + 2, 33, 2, jewel);
    }
    c.rect(16, 56, 32, 2, [156, 120, 40, 255]);
}

fn key(c: &mut Canvas, color: [u8; 4]) {
    let hi = [
        (color[0] as f32 * 1.25).min(255.0) as u8,
        (color[1] as f32 * 1.25).min(255.0) as u8,
        (color[2] as f32 * 1.25).min(255.0) as u8,
        255,
    ];
    c.ellipse(32, 60, 8, 3, [0, 0, 0, 90]);
    c.circle(32, 34, 7, color);
    c.circle(32, 34, 3, [0, 0, 0, 0]);
    c.circle(32, 34, 2, [40, 40, 44, 255]);
    c.rect(30, 40, 4, 18, color);
    c.rect(34, 48, 6, 3, color);
    c.rect(34, 54, 4, 3, color);
    c.vline(30, 40, 18, hi);
    c.circle(32, 34, 7, hi);
    c.circle(32, 34, 6, color);
}

fn extra_life(c: &mut Canvas) {
    let gold = [216, 178, 66, 255];
    c.ellipse(28, 60, 12, 3, [0, 0, 0, 90]);
    c.circle(26, 32, 6, gold);
    c.rect(21, 38, 10, 12, gold);
    c.rect(21, 50, 4, 9, gold);
    c.rect(28, 50, 4, 9, gold);
    c.rect(18, 40, 3, 8, gold);
    c.rect(33, 40, 3, 8, gold);
    c.rect(40, 30, 4, 18, [248, 226, 150, 255]);
    c.rect(33, 37, 18, 4, [248, 226, 150, 255]);
}

fn barrel(c: &mut Canvas, rng: &mut Rng) {
    c.ellipse(32, 60, 15, 4, [0, 0, 0, 100]);
    c.ellipse(32, 32, 14, 6, [122, 82, 44, 255]);
    c.rect(18, 32, 28, 26, [104, 68, 36, 255]);
    c.ellipse(32, 58, 14, 5, [78, 52, 28, 255]);
    for y in [36, 46, 54] {
        c.hline(17, y, 30, [92, 96, 104, 255]);
        c.hline(17, y + 1, 30, [58, 62, 68, 255]);
    }
    c.vline(22, 34, 24, [140, 96, 52, 255]);
    c.noise(rng, 0.08, true);
}

fn table(c: &mut Canvas, rng: &mut Rng) {
    c.ellipse(32, 60, 18, 4, [0, 0, 0, 90]);
    c.rect(8, 34, 48, 6, [128, 84, 42, 255]);
    c.hline(8, 34, 48, [164, 112, 58, 255]);
    c.rect(14, 40, 5, 20, [96, 62, 30, 255]);
    c.rect(45, 40, 5, 20, [96, 62, 30, 255]);
    c.noise(rng, 0.06, true);
}

fn pillar(c: &mut Canvas, rng: &mut Rng) {
    c.ellipse(32, 60, 14, 4, [0, 0, 0, 100]);
    c.rect(20, 8, 24, 52, [118, 118, 114, 255]);
    c.rect(16, 4, 32, 6, [134, 134, 130, 255]);
    c.rect(16, 58, 32, 4, [104, 104, 100, 255]);
    c.vline(22, 10, 48, [140, 140, 136, 255]);
    c.vline(41, 10, 48, [88, 88, 84, 255]);
    for y in (14..56).step_by(10) {
        c.hline(20, y, 24, [100, 100, 96, 255]);
    }
    c.noise(rng, 0.09, true);
}

fn lamp(c: &mut Canvas) {
    c.ellipse(32, 60, 10, 3, [0, 0, 0, 90]);
    c.rect(30, 34, 4, 26, [86, 74, 58, 255]);
    c.ellipse(32, 58, 9, 4, [86, 74, 58, 255]);
    c.ellipse(32, 28, 11, 8, [212, 186, 112, 255]);
    c.ellipse(32, 26, 9, 6, [252, 240, 180, 255]);
    c.hline(22, 24, 20, [255, 252, 220, 255]);
}

fn chandelier(c: &mut Canvas) {
    c.rect(30, 0, 4, 14, [78, 74, 68, 255]);
    c.ellipse(32, 24, 20, 5, [96, 92, 84, 255]);
    for i in 0..5 {
        let x = 14 + i * 9;
        c.rect(x, 20, 5, 5, [236, 222, 150, 255]);
        c.circle(x + 2, 22, 3, [255, 248, 200, 255]);
    }
    c.ellipse(32, 32, 6, 8, [86, 82, 74, 255]);
}

fn plant(c: &mut Canvas, rng: &mut Rng) {
    c.ellipse(32, 60, 12, 3, [0, 0, 0, 90]);
    c.rect(22, 48, 20, 12, [148, 92, 58, 255]);
    c.hline(22, 48, 20, [186, 122, 78, 255]);
    for _ in 0..9 {
        let (dx, dy) = (rng.range(-11, 12), rng.range(-22, 0));
        c.ellipse(
            32 + dx,
            46 + dy,
            rng.range(3, 6),
            rng.range(4, 8),
            [46, 108, 52, 255],
        );
    }
    for _ in 0..6 {
        c.line(
            32,
            48,
            32 + rng.range(-10, 11),
            46 + rng.range(-20, 0),
            [34, 82, 40, 255],
        );
    }
}

fn puddle(c: &mut Canvas, rng: &mut Rng) {
    c.ellipse(32, 56, 20, 6, [92, 16, 16, 170]);
    c.ellipse(30, 55, 12, 4, [120, 24, 24, 190]);
    for _ in 0..12 {
        let (x, y) = (rng.range(-18, 19), rng.range(-3, 4));
        c.set(32 + x, 58 + y, [72, 10, 10, 200]);
    }
}

/// --- Weapons ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponKind {
    Knife,
    Pistol,
    MachineGun,
    Chaingun,
}

impl WeaponKind {
    pub const ALL: [WeaponKind; 4] = [
        WeaponKind::Knife,
        WeaponKind::Pistol,
        WeaponKind::MachineGun,
        WeaponKind::Chaingun,
    ];

    pub fn index(self) -> usize {
        WeaponKind::ALL.iter().position(|w| *w == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Knife => "Knife",
            WeaponKind::Pistol => "Pistol",
            WeaponKind::MachineGun => "Machine gun",
            WeaponKind::Chaingun => "Chaingun",
        }
    }

    /// Frames in the firing animation of this weapon
    pub fn frames(self) -> usize {
        match self {
            WeaponKind::Knife => 2,
            WeaponKind::Pistol => 3,
            WeaponKind::MachineGun => 3,
            WeaponKind::Chaingun => 4,
        }
    }
}

pub struct WeaponSprites {
    pub atlas: Atlas,
    base: [usize; 4],
}

impl WeaponSprites {
    pub fn generate() -> Self {
        let mut base = [0usize; 4];
        let mut at = 0;
        for kind in WeaponKind::ALL {
            base[kind.index()] = at;
            at += kind.frames();
        }

        WeaponSprites {
            atlas: super::weapons_art::atlas(),
            base,
        }
    }

    pub fn tex_rect(&self, kind: WeaponKind, frame: usize) -> (usize, usize, usize, usize) {
        let frame = frame.min(kind.frames() - 1);
        self.atlas.tex_rect(self.base[kind.index()] + frame)
    }
}

/// Draws the animated status bar face, 24x32 pixels, at the given position.
pub fn draw_face(c: &mut Canvas, x: i32, y: i32, health: i32, look: i32, mood: FaceMood) {
    let skin = [226, 178, 138, 255];
    let skin_dark = [188, 140, 104, 255];
    let hair = [122, 88, 52, 255];

    c.rect(x, y, 24, 32, [96, 84, 72, 255]);
    c.frame(x - 1, y - 1, 26, 34, [58, 50, 44, 255]);

    let dead = health <= 0;
    let healthy = health > 60;

    let face = if dead { skin_dark } else { skin };
    c.ellipse(x + 12, y + 16, 9, 12, face);
    c.ellipse(x + 12, y + 6, 10, 5, hair);
    let eye_offset = look.clamp(-2, 2);
    let wounded = health <= 60 && !dead;
    let badly = health <= 20 && !dead;

    if dead {
        for dx in [-4, 4] {
            c.line(
                x + 12 + dx - 2,
                y + 14,
                x + 12 + dx + 2,
                y + 18,
                [60, 30, 30, 255],
            );
            c.line(
                x + 12 + dx + 2,
                y + 14,
                x + 12 + dx - 2,
                y + 18,
                [60, 30, 30, 255],
            );
        }
        c.ellipse(x + 12, y + 24, 4, 3, [70, 20, 20, 255]);
    } else {
        for dx in [-4, 4] {
            c.rect(x + 12 + dx + eye_offset, y + 15, 3, 3, [248, 248, 240, 255]);
            c.rect(x + 13 + dx + eye_offset, y + 16, 2, 2, [40, 40, 60, 255]);
        }
        let brow_y = y + 11 + if badly { 1 } else { 0 };
        for dx in [-6, 4] {
            c.hline(x + 12 + dx, brow_y, 4, hair);
        }
        match mood {
            FaceMood::Normal => c.hline(x + 9, y + 24, 7, [140, 60, 60, 255]),
            FaceMood::Firing => c.ellipse(x + 12, y + 24, 3, 3, [90, 20, 20, 255]),
            FaceMood::Pain => {
                c.ellipse(x + 12, y + 24, 4, 3, [110, 26, 26, 255]);
                c.hline(x + 10, y + 23, 5, [240, 220, 220, 255]);
            }
        }
    }

    if wounded && !dead {
        c.vline(x + 6, y + 12, 5, [170, 20, 20, 255]);
    }
    if badly && !dead {
        c.vline(x + 18, y + 13, 7, [170, 20, 20, 255]);
        c.set(x + 8, y + 20, [170, 20, 20, 255]);
        c.set(x + 15, y + 22, [170, 20, 20, 255]);
    }

    if !dead && healthy {
        c.set(x + 12, y + 16, skin_dark);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaceMood {
    Normal,
    Firing,
    Pain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_actor_facing_the_camera_uses_the_front_view() {
        let camera = 0.6f32;
        assert_eq!(
            ActorSprites::view_for(camera + std::f32::consts::PI, camera),
            0,
            "an actor aiming at the player must show its front"
        );
        assert_eq!(
            ActorSprites::view_for(camera, camera),
            VIEWS / 2,
            "an actor looking away shows its back"
        );
        let side = ActorSprites::view_for(camera + std::f32::consts::FRAC_PI_2, camera);
        assert!(
            side == 2 || side == 6,
            "a quarter turn should be a profile, got {side}"
        );
    }

    #[test]
    fn every_actor_has_a_cell_for_every_pose() {
        let sprites = ActorSprites::generate();
        for kind in ActorKind::ALL {
            for view in 0..VIEWS {
                for slot in 0..SLOTS {
                    let rect = sprites.tex_rect(sprites.cell(kind, view, slot));
                    assert_eq!(rect.2, CELL as usize, "{} cell width", kind.name());
                    assert_eq!(rect.3, CELL as usize, "{} cell height", kind.name());
                }
            }
        }
    }

    #[test]
    fn the_attack_animation_has_four_distinct_frames() {
        let sprites = ActorSprites::generate();
        let cells: Vec<usize> = (0..SHOOT_FRAMES)
            .map(|frame| sprites.cell(ActorKind::Guard, 0, SLOT_SHOOT + frame))
            .collect();

        let mut unique = cells.clone();
        unique.dedup();
        assert_eq!(
            unique.len(),
            SHOOT_FRAMES,
            "the four attack frames must be separate cells"
        );
    }

    /// Copies one atlas cell out as plain RGBA pixels
    fn cell_pixels(atlas: &Atlas, index: usize) -> Vec<u8> {
        let (x, y, width, height) = atlas.tex_rect(index);
        let stride = atlas.width as usize * 4;
        let mut pixels = Vec::with_capacity(width * height * 4);
        for row in 0..height {
            let start = y + row * stride + x;
            pixels.extend_from_slice(&atlas.data[start..start + width * 4]);
        }
        pixels
    }

    fn pixels_that_differ(a: &[u8], b: &[u8]) -> usize {
        a.chunks_exact(4)
            .zip(b.chunks_exact(4))
            .filter(|(left, right)| left != right)
            .count()
    }

    #[test]
    fn the_attack_frames_are_drawn_differently_not_just_reindexed() {
        // has to raise, fire and recover, and the last frame must not match the first.
        let sprites = ActorSprites::generate();
        for kind in ActorKind::ALL {
            let frames: Vec<Vec<u8>> = (0..SHOOT_FRAMES)
                .map(|frame| cell_pixels(&sprites.atlas, sprites.cell(kind, 0, SLOT_SHOOT + frame)))
                .collect();

            for (pair, (a, b)) in frames.iter().zip(frames.iter().skip(1)).enumerate() {
                let changed = pixels_that_differ(a, b);
                assert!(
                    changed > 80,
                    "{}: attack frames {pair} and {} differ in only {changed} pixels",
                    kind.name(),
                    pair + 1
                );
            }

            let stand = cell_pixels(&sprites.atlas, sprites.cell(kind, 0, SLOT_STAND));
            assert!(
                pixels_that_differ(&stand, &frames[0]) > 80,
                "{}: the first attack frame repeats the stand pose",
                kind.name()
            );
        }
    }
}
