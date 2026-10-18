//! Procedural wall, floor and ceiling textures.

use super::{cc0_image, Atlas, Canvas, Rng};

pub const CELL: i32 = 64;

/// Every texture in the game, in atlas order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tex {
    Cell,
    Bluestone,
    Greystone,
    Brick,
    BrickWood,
    Wood,
    WoodEmblem,
    Banner,
    Door,
    CellDoor,
    Metal,
    ElevatorSwitch,
    FloorStone,
    FloorWood,
    FloorBlue,
    FloorGrate,
    CeilDark,
    CeilLight,
    CeilWood,
}

impl Tex {
    pub const ALL: [Tex; 19] = [
        Tex::Cell,
        Tex::Bluestone,
        Tex::Greystone,
        Tex::Brick,
        Tex::BrickWood,
        Tex::Wood,
        Tex::WoodEmblem,
        Tex::Banner,
        Tex::Door,
        Tex::CellDoor,
        Tex::Metal,
        Tex::ElevatorSwitch,
        Tex::FloorStone,
        Tex::FloorWood,
        Tex::FloorBlue,
        Tex::FloorGrate,
        Tex::CeilDark,
        Tex::CeilLight,
        Tex::CeilWood,
    ];

    pub fn index(self) -> usize {
        Tex::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Tex::Cell => "wall_cell",
            Tex::Bluestone => "wall_bluestone",
            Tex::Greystone => "wall_greystone",
            Tex::Brick => "wall_brick",
            Tex::BrickWood => "wall_brick_wood",
            Tex::Wood => "wall_wood",
            Tex::WoodEmblem => "wall_wood_emblem",
            Tex::Banner => "wall_banner",
            Tex::Door => "wall_door",
            Tex::CellDoor => "wall_cell_door",
            Tex::Metal => "wall_metal",
            Tex::ElevatorSwitch => "wall_elevator_switch",
            Tex::FloorStone => "floor_stone",
            Tex::FloorWood => "floor_wood",
            Tex::FloorBlue => "floor_blue",
            Tex::FloorGrate => "floor_grate",
            Tex::CeilDark => "ceil_dark",
            Tex::CeilLight => "ceil_light",
            Tex::CeilWood => "ceil_wood",
        }
    }
}

/// The texture atlas: one 64x64 cell per `Tex`, plus the tile rectangles the engine wants.
pub struct Textures {
    pub atlas: Atlas,
}

impl Textures {
    pub fn generate() -> Self {
        let mut atlas = Atlas::from_cells(Tex::ALL.len(), CELL);

        for tex in Tex::ALL {
            let index = tex.index();

            if let Some((data, w, h)) = cc0_image(tex.name()) {
                atlas.blit(index, &data, w, h, false);
                continue;
            }

            let mut rng = Rng::new(0x5EED_0000 + index as u64);
            let mut c = atlas.canvas(index);
            match tex {
                Tex::Cell => masonry(
                    &mut c,
                    &mut rng,
                    [138, 138, 132, 255],
                    [58, 58, 56, 255],
                    16,
                    8,
                    0.08,
                    true,
                ),
                Tex::Bluestone => rough_stone(
                    &mut c,
                    &mut rng,
                    [92, 108, 140, 255],
                    [34, 40, 58, 255],
                    14,
                    10,
                    20,
                    0.10,
                ),
                Tex::Greystone => rough_stone(
                    &mut c,
                    &mut rng,
                    [116, 116, 112, 255],
                    [48, 48, 46, 255],
                    16,
                    14,
                    26,
                    0.09,
                ),
                Tex::Brick => masonry(
                    &mut c,
                    &mut rng,
                    [156, 62, 44, 255],
                    [112, 96, 82, 255],
                    16,
                    8,
                    0.10,
                    true,
                ),
                Tex::BrickWood => {
                    masonry(
                        &mut c,
                        &mut rng,
                        [150, 60, 44, 255],
                        [104, 90, 78, 255],
                        16,
                        8,
                        0.10,
                        true,
                    );
                    wood_band(&mut c, &mut rng, 0, 16, [122, 78, 38, 255]);
                    wood_band(&mut c, &mut rng, 48, 16, [122, 78, 38, 255]);
                }
                Tex::Wood => planks(&mut c, &mut rng, [138, 88, 42, 255], 8, true),
                Tex::WoodEmblem => {
                    planks(&mut c, &mut rng, [128, 80, 38, 255], 8, true);
                    emblem(&mut c, [214, 214, 206, 255], [70, 44, 20, 255]);
                }
                Tex::Banner => {
                    masonry(
                        &mut c,
                        &mut rng,
                        [128, 128, 124, 255],
                        [56, 56, 54, 255],
                        16,
                        8,
                        0.08,
                        true,
                    );
                    banner(&mut c);
                }
                Tex::Door => door_texture(&mut c, &mut rng, [104, 62, 30, 255]),
                Tex::CellDoor => door_texture(&mut c, &mut rng, [92, 92, 96, 255]),
                Tex::Metal => metal(&mut c, &mut rng),
                Tex::ElevatorSwitch => elevator_switch(&mut c, &mut rng),
                Tex::FloorStone => floor_stone(&mut c, &mut rng),
                Tex::FloorWood => planks(&mut c, &mut rng, [104, 68, 34, 255], 10, false),
                Tex::FloorBlue => floor_blue(&mut c, &mut rng),
                Tex::FloorGrate => floor_grate(&mut c, &mut rng),
                Tex::CeilDark => ceiling(&mut c, &mut rng, [46, 46, 50, 255], [30, 30, 34, 255]),
                Tex::CeilLight => ceiling(&mut c, &mut rng, [92, 92, 88, 255], [68, 68, 64, 255]),
                Tex::CeilWood => planks(&mut c, &mut rng, [74, 48, 24, 255], 12, false),
            }
        }

        Textures { atlas }
    }

    /// The engine tile for a texture, for a given image id inside the engine's world
    pub fn tile_with_image(&self, image_id: usize, tex: Tex) -> raycaster::Tile {
        raycaster::Tile::textured(image_id, self.atlas.tex_rect(tex.index()))
    }
}

fn shade(color: [u8; 4], factor: f32) -> [u8; 4] {
    [
        (color[0] as f32 * factor).clamp(0.0, 255.0) as u8,
        (color[1] as f32 * factor).clamp(0.0, 255.0) as u8,
        (color[2] as f32 * factor).clamp(0.0, 255.0) as u8,
        color[3],
    ]
}

/// Running-bond brickwork, the look of most Wolfenstein walls
fn masonry(
    c: &mut Canvas,
    rng: &mut Rng,
    base: [u8; 4],
    mortar: [u8; 4],
    bw: i32,
    bh: i32,
    jitter: f32,
    offset_rows: bool,
) {
    c.fill(mortar);

    for row in 0..(c.height / bh + 2) {
        let y = row * bh;
        let start_x = if offset_rows && row % 2 == 1 {
            -bw / 2
        } else {
            0
        };
        let mut x = start_x;

        while x < c.width {
            let color = shade(base, 1.0 + rng.range_f32(-jitter, jitter));
            c.rect(x + 1, y + 1, bw - 2, bh - 2, color);
            c.hline(x + 1, y + 1, bw - 2, shade(color, 1.16));
            c.vline(x + 1, y + 1, bh - 2, shade(color, 1.12));
            c.hline(x + 1, y + bh - 2, bw - 2, shade(color, 0.80));
            c.vline(x + bw - 2, y + 1, bh - 2, shade(color, 0.86));
            x += bw;
        }
    }

    c.noise(rng, 0.12, false);
}

/// Irregular rubble masonry, for the blue and grey dungeon walls
fn rough_stone(
    c: &mut Canvas,
    rng: &mut Rng,
    base: [u8; 4],
    mortar: [u8; 4],
    row_h: i32,
    min_w: i32,
    max_w: i32,
    jitter: f32,
) {
    c.fill(mortar);

    let mut y = -rng.range(0, row_h);
    while y < c.height {
        let h = row_h + rng.range(-2, 3);
        let mut x = -rng.range(0, max_w);

        while x < c.width {
            let w = rng.range(min_w, max_w + 1);
            let color = shade(base, 1.0 + rng.range_f32(-jitter, jitter));
            c.rect(x + 1, y + 1, w - 2, h - 2, color);
            c.hline(x + 1, y + 1, w - 2, shade(color, 1.18));
            c.hline(x + 1, y + h - 2, w - 2, shade(color, 0.80));
            c.vline(x + 1, y + 1, h - 2, shade(color, 1.08));
            c.vline(x + w - 2, y + 1, h - 2, shade(color, 0.84));
            x += w;
        }

        y += h;
    }

    c.noise(rng, 0.14, false);
}

/// Wooden boards with grain, used for walls, floors and ceilings
fn planks(c: &mut Canvas, rng: &mut Rng, base: [u8; 4], plank: i32, vertical: bool) {
    c.fill(shade(base, 0.72));

    let span = if vertical { c.width } else { c.height };
    let mut at = 0;
    while at < span {
        let color = shade(base, 1.0 + rng.range_f32(-0.14, 0.14));
        if vertical {
            c.rect(at, 0, plank - 1, c.height, color);
            c.vline(at + plank - 1, 0, c.height, shade(color, 0.6));
        } else {
            c.rect(0, at, c.width, plank - 1, color);
            c.hline(0, at + plank - 1, c.width, shade(color, 0.6));
        }
        at += plank;
    }

    for _ in 0..90 {
        let (x, y) = (rng.range(0, c.width), rng.range(0, c.height));
        let len = rng.range(3, 12);
        let color = shade(base, rng.range_f32(0.78, 1.2));
        if vertical {
            c.vline(x, y, len, color);
        } else {
            c.hline(x, y, len, color);
        }
    }

    c.noise(rng, 0.09, false);
}

/// A horizontal beam of wood across a wall, for the brick-and-wood variant
fn wood_band(c: &mut Canvas, rng: &mut Rng, y: i32, h: i32, base: [u8; 4]) {
    c.rect(
        0,
        y,
        c.width,
        h,
        shade(base, 1.0 + rng.range_f32(-0.06, 0.06)),
    );
    c.hline(0, y, c.width, shade(base, 1.25));
    c.hline(0, y + h - 1, c.width, shade(base, 0.65));
}

/// The eagle-less emblem wall: an equal-armed iron cross in a ring. Deliberately not the
fn emblem(c: &mut Canvas, light: [u8; 4], dark: [u8; 4]) {
    let (cx, cy) = (c.width / 2, c.height / 2);

    c.circle(cx, cy, 19, shade(dark, 0.8));
    c.circle(cx, cy, 17, light);

    let arm = 6;
    let len = 15;
    c.rect(cx - arm / 2, cy - len, arm, len * 2, dark);
    c.rect(cx - len, cy - arm / 2, len * 2, arm, dark);

    for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let x = cx + dx * len - if dx > 0 { 4 } else { 0 };
        let y = cy + dy * len - if dy > 0 { 4 } else { 0 };
        c.rect(x, y, 5, 5, dark);
    }

    c.circle(cx, cy, 19, shade(dark, 1.3));
    c.set(cx, cy - 19, light);
    c.set(cx, cy + 19, light);
    c.set(cx - 19, cy, light);
    c.set(cx + 19, cy, light);
}

/// A hanging banner on a stone wall
fn banner(c: &mut Canvas) {
    let cloth = [138, 32, 32, 255];
    let cloth_dark = [96, 20, 20, 255];

    c.rect(0, 8, c.width, 3, [72, 72, 70, 255]);
    c.rect(10, 11, 44, 44, cloth);
    c.rect(10, 11, 44, 3, cloth_dark);
    for x in (12..50).step_by(6) {
        c.vline(x, 14, 41, cloth_dark);
    }
    c.vertical_shade(1.0, 0.78);

    let (cx, cy) = (32, 33);
    c.circle(cx, cy, 12, [214, 214, 206, 255]);
    c.rect(cx - 3, cy - 10, 6, 20, cloth);
    c.rect(cx - 10, cy - 3, 20, 6, cloth);
}

/// A hinged door: planked leaf, recessed panels and a handle
fn door_texture(c: &mut Canvas, rng: &mut Rng, base: [u8; 4]) {
    c.fill(shade(base, 0.7));
    planks_on(c, rng, 2, 2, c.width - 4, c.height - 4, base, 9);

    c.frame(0, 0, c.width, c.height, shade(base, 0.5));
    c.frame(1, 1, c.width - 2, c.height - 2, shade(base, 1.3));

    for (px, py) in [(9, 7), (9, 34)] {
        c.rect(px, py, 22, 23, shade(base, 0.55));
        c.rect(px + 1, py + 1, 20, 21, shade(base, 1.05));
        c.hline(px + 1, py + 1, 20, shade(base, 1.35));
        c.hline(px + 1, py + 21, 20, shade(base, 0.7));
    }
    for (px, py) in [(35, 7), (35, 34)] {
        c.rect(px, py, 22, 23, shade(base, 0.55));
        c.rect(px + 1, py + 1, 20, 21, shade(base, 1.05));
        c.hline(px + 1, py + 1, 20, shade(base, 1.35));
        c.hline(px + 1, py + 21, 20, shade(base, 0.7));
    }

    c.rect(29, 29, 6, 8, [188, 188, 182, 255]);
    c.rect(30, 30, 4, 3, [120, 120, 116, 255]);
    c.rect(30, 34, 4, 3, [120, 120, 116, 255]);
}

fn planks_on(
    c: &mut Canvas,
    rng: &mut Rng,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    base: [u8; 4],
    plank: i32,
) {
    c.rect(x, y, w, h, base);
    let mut at = x;
    while at < x + w {
        c.rect(
            at,
            y,
            (plank - 1).min(x + w - at),
            h,
            shade(base, 1.0 + rng.range_f32(-0.10, 0.10)),
        );
        c.vline(at + plank - 1, y, h, shade(base, 0.62));
        at += plank;
    }
    for _ in 0..40 {
        let (sx, sy) = (rng.range(x, x + w), rng.range(y, y + h));
        c.vline(
            sx,
            sy,
            rng.range(2, 8),
            shade(base, rng.range_f32(0.8, 1.15)),
        );
    }
}

/// Riveted steel plating
fn metal(c: &mut Canvas, rng: &mut Rng) {
    let base = [104, 112, 122, 255];
    c.fill(base);

    for row in 0..2 {
        for col in 0..2 {
            let (x, y) = (col * 32, row * 32);
            let color = shade(base, 1.0 + rng.range_f32(-0.07, 0.07));
            c.rect(x + 1, y + 1, 30, 30, color);
            c.hline(x + 1, y + 1, 30, shade(color, 1.2));
            c.vline(x + 1, y + 1, 30, shade(color, 1.15));
            c.hline(x + 1, y + 30, 30, shade(color, 0.7));
            c.vline(x + 30, y + 1, 30, shade(color, 0.75));

            for (rx, ry) in [
                (x + 4, y + 4),
                (x + 27, y + 4),
                (x + 4, y + 27),
                (x + 27, y + 27),
            ] {
                c.circle(rx, ry, 2, shade(color, 1.35));
                c.set(rx, ry, shade(color, 0.6));
            }
        }
    }

    for _ in 0..10 {
        let x = rng.range(0, c.width);
        c.vline(
            x,
            rng.range(0, c.height / 2),
            rng.range(8, 30),
            [78, 80, 76, 255],
        );
    }

    c.noise(rng, 0.08, false);
}

/// The elevator call switch that ends a floor
fn elevator_switch(c: &mut Canvas, rng: &mut Rng) {
    rough_stone(
        c,
        rng,
        [112, 112, 108, 255],
        [52, 52, 50, 255],
        16,
        12,
        22,
        0.08,
    );

    c.rect(16, 14, 32, 36, [86, 92, 100, 255]);
    c.frame(16, 14, 32, 36, [58, 62, 68, 255]);
    c.hline(17, 15, 30, [140, 146, 152, 255]);

    c.rect(28, 22, 8, 22, [46, 48, 52, 255]);
    c.rect(24, 18, 16, 8, [176, 62, 48, 255]);
    c.rect(24, 18, 16, 2, [220, 96, 76, 255]);

    c.circle(32, 46, 5, [52, 54, 58, 255]);
    c.circle(32, 46, 3, [212, 176, 68, 255]);
    c.set(31, 45, [250, 230, 150, 255]);
}

fn floor_stone(c: &mut Canvas, rng: &mut Rng) {
    let base = [96, 96, 94, 255];
    c.fill(shade(base, 0.78));

    for row in 0..=2 {
        for col in 0..=2 {
            let (x, y) = (col * 32, row * 32);
            let color = shade(base, 1.0 + rng.range_f32(-0.12, 0.12));
            c.rect(x + 1, y + 1, 30, 30, color);
            c.hline(x + 1, y + 1, 30, shade(color, 1.1));
            c.hline(x + 1, y + 30, 30, shade(color, 0.8));
        }
    }

    for _ in 0..26 {
        let (x, y) = (rng.range(0, c.width), rng.range(0, c.height));
        c.rect(
            x,
            y,
            rng.range(2, 6),
            rng.range(1, 3),
            shade(base, rng.range_f32(0.7, 1.25)),
        );
    }

    c.noise(rng, 0.16, false);
}

fn floor_blue(c: &mut Canvas, rng: &mut Rng) {
    let base = [64, 74, 102, 255];
    c.fill(shade(base, 0.8));

    for row in 0..=3 {
        for col in 0..=3 {
            let (x, y) = (col * 16, row * 16);
            let color = shade(base, 1.0 + rng.range_f32(-0.14, 0.16));
            c.rect(x + 1, y + 1, 14, 14, color);
            c.hline(x + 1, y + 1, 14, shade(color, 1.12));
            c.hline(x + 1, y + 14, 14, shade(color, 0.78));
        }
    }

    c.noise(rng, 0.14, false);
}

fn floor_grate(c: &mut Canvas, rng: &mut Rng) {
    c.fill([38, 38, 42, 255]);
    for x in (0..c.width).step_by(8) {
        c.vline(x, 0, c.height, [96, 100, 106, 255]);
        c.vline(x + 1, 0, c.height, [70, 72, 78, 255]);
    }
    for y in (0..c.height).step_by(32) {
        c.hline(0, y, c.width, [104, 108, 114, 255]);
    }
    c.noise(rng, 0.1, false);
}

fn ceiling(c: &mut Canvas, rng: &mut Rng, base: [u8; 4], grid: [u8; 4]) {
    c.fill(base);
    for x in (0..c.width).step_by(16) {
        c.vline(x, 0, c.height, grid);
    }
    for y in (0..c.height).step_by(16) {
        c.hline(0, y, c.width, grid);
    }
    for _ in 0..40 {
        let (x, y) = (rng.range(0, c.width), rng.range(0, c.height));
        c.set(x, y, shade(base, rng.range_f32(0.8, 1.2)));
    }
    c.noise(rng, 0.12, false);
}
