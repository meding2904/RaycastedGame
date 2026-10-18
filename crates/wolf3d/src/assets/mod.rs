//! Asset generation.

pub mod font;
pub mod sfx;
pub mod sprites;
pub mod textures;
pub mod weapons_art;

use std::path::{Path, PathBuf};

/// Directory scanned for downloaded CC0 art. Files in here override the procedural generator
pub const CC0_DIR: &str = "assets/cc0";

/// Everything the game draws from and listens to, generated once at startup
pub struct Assets {
    pub textures: textures::Textures,
    pub actors: sprites::ActorSprites,
    pub objects: sprites::ObjectSprites,
    pub weapons: sprites::WeaponSprites,
    pub sounds: sfx::Sounds,
}

impl Assets {
    pub fn generate() -> Self {
        Self {
            textures: textures::Textures::generate(),
            actors: sprites::ActorSprites::generate(),
            objects: sprites::ObjectSprites::generate(),
            weapons: sprites::WeaponSprites::generate(),
            sounds: sfx::Sounds::generate(),
        }
    }
}

/// A tiny deterministic xorshift64* PRNG. `rand` is used for gameplay randomness; art must not
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
    }

    /// A value in `lo..hi`, `lo` when the range is empty
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }

    /// A value in `0.0..1.0`
    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.unit() * (hi - lo)
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.unit() < probability
    }
}

/// A rectangular, stride-aware RGBA view onto a bigger image, used to draw into one atlas cell
pub struct Canvas<'a> {
    pub data: &'a mut [u8],
    pub stride: i32,
    pub width: i32,
    pub height: i32,
    pub ox: i32,
    pub oy: i32,
}

impl<'a> Canvas<'a> {
    pub fn new(data: &'a mut [u8], stride: i32, ox: i32, oy: i32, width: i32, height: i32) -> Self {
        Self {
            data,
            stride,
            width,
            height,
            ox,
            oy,
        }
    }

    pub fn set(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let off = ((self.oy + y) * self.stride + self.ox + x) as usize * 4;
        if off + 4 <= self.data.len() {
            self.data[off..off + 4].copy_from_slice(&color);
        }
    }

    pub fn get(&self, x: i32, y: i32) -> [u8; 4] {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let off = ((self.oy + y) * self.stride + self.ox + x) as usize * 4;
        if off + 4 <= self.data.len() {
            let mut c = [0u8; 4];
            c.copy_from_slice(&self.data[off..off + 4]);
            return c;
        }
        [0, 0, 0, 0]
    }

    /// Multiplies the color already at the pixel by `factor`, used for cheap shading
    pub fn darken(&mut self, x: i32, y: i32, factor: f32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let off = ((self.oy + y) * self.stride + self.ox + x) as usize * 4;
        if off + 4 <= self.data.len() {
            for i in 0..3 {
                self.data[off + i] = (self.data[off + i] as f32 * factor).clamp(0.0, 255.0) as u8;
            }
        }
    }

    pub fn fill(&mut self, color: [u8; 4]) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set(x, y, color);
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, color);
            }
        }
    }

    /// An unfilled rectangle
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for xx in x..x + w {
            self.set(xx, y, color);
            self.set(xx, y + h - 1, color);
        }
        for yy in y..y + h {
            self.set(x, yy, color);
            self.set(x + w - 1, yy, color);
        }
    }

    pub fn hline(&mut self, x: i32, y: i32, w: i32, color: [u8; 4]) {
        for xx in x..x + w {
            self.set(xx, y, color);
        }
    }

    pub fn vline(&mut self, x: i32, y: i32, h: i32, color: [u8; 4]) {
        for yy in y..y + h {
            self.set(x, yy, color);
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);

        loop {
            self.set(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn circle(&mut self, cx: i32, cy: i32, r: i32, color: [u8; 4]) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    /// A filled ellipse, used for heads, bodies and barrels
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [u8; 4]) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for y in -ry..=ry {
            for x in -rx..=rx {
                let nx = x as f32 / rx as f32;
                let ny = y as f32 / ry as f32;
                if nx * nx + ny * ny <= 1.0 {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    /// Speckles the whole cell with random noise, `amount` is a strength in 0.0..1.0
    pub fn noise(&mut self, rng: &mut Rng, amount: f32, alpha_only_opaque: bool) {
        for y in 0..self.height {
            for x in 0..self.width {
                if !rng.chance(amount) {
                    continue;
                }
                let c = self.get(x, y);
                if alpha_only_opaque && c[3] == 0 {
                    continue;
                }
                let delta = rng.range_f32(-1.0, 1.0) * 22.0;
                let shade = |v: u8| (v as f32 + delta).clamp(0.0, 255.0) as u8;
                self.set(x, y, [shade(c[0]), shade(c[1]), shade(c[2]), c[3]]);
            }
        }
    }

    /// Draws a vertical gradient over the cell from `color` to transparent, used for depth
    pub fn vertical_shade(&mut self, top: f32, bottom: f32) {
        for y in 0..self.height {
            let t = y as f32 / (self.height.max(1) - 1) as f32;
            let factor = top + (bottom - top) * t;
            for x in 0..self.width {
                self.darken(x, y, factor);
            }
        }
    }
}

/// An RGBA image split into a grid of square cells. The engine addresses textures by byte
pub struct Atlas {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub cell: i32,
    pub cols: i32,
}

impl Atlas {
    pub fn new(cols: i32, rows: i32, cell: i32) -> Self {
        let width = cols * cell;
        let height = rows * cell;
        Self {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
            cell,
            cols,
        }
    }

    pub fn from_cells(cells: usize, cell: i32) -> Self {
        let cols = cell_to_cols(cells);
        let rows = (cells as i32 + cols - 1) / cols;
        Self::new(cols, rows.max(1), cell)
    }

    pub fn cell_count(&self) -> usize {
        (self.cols * (self.height / self.cell)) as usize
    }

    /// The engine's texture rectangle for a cell: (x byte offset, y byte offset, width, height)
    pub fn tex_rect(&self, index: usize) -> (usize, usize, usize, usize) {
        let cell = self.cell as usize;
        let x = (index as i32 % self.cols) * self.cell;
        let y = (index as i32 / self.cols) * self.cell;
        (
            x as usize * 4,
            y as usize * self.width as usize * 4,
            cell,
            cell,
        )
    }

    pub fn canvas(&mut self, index: usize) -> Canvas<'_> {
        let x = (index as i32 % self.cols) * self.cell;
        let y = (index as i32 / self.cols) * self.cell;
        Canvas::new(&mut self.data, self.width, x, y, self.cell, self.cell)
    }

    /// Copies a source image into a cell, optionally mirrored horizontally
    pub fn blit(
        &mut self,
        index: usize,
        src: &[u8],
        src_width: i32,
        src_height: i32,
        mirror: bool,
    ) {
        let mut dst = self.canvas(index);
        for y in 0..src_height.min(dst.height) {
            for x in 0..src_width.min(dst.width) {
                let sx = if mirror { src_width - 1 - x } else { x };
                let off = ((y * src_width + sx) * 4) as usize;
                if off + 4 > src.len() {
                    continue;
                }
                let mut c = [0u8; 4];
                c.copy_from_slice(&src[off..off + 4]);
                dst.set(x, y, c);
            }
        }
    }

    /// Copies one cell of this atlas into another cell of the same atlas, mirrored if asked.
    pub fn copy_cell(&mut self, from: usize, to: usize, mirror: bool) {
        let cell = self.cell as usize;
        let fx = ((from as i32 % self.cols) * self.cell) as usize;
        let fy = ((from as i32 / self.cols) * self.cell) as usize;
        let width = self.width as usize;

        let mut src = Vec::with_capacity(cell * cell * 4);
        for y in 0..cell {
            for x in 0..cell {
                let off = ((fy + y) * width + fx + x) * 4;
                if off + 4 <= self.data.len() {
                    src.extend_from_slice(&self.data[off..off + 4]);
                } else {
                    src.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        self.blit(to, &src, self.cell, self.cell, mirror);
    }
}

fn cell_to_cols(cells: usize) -> i32 {
    // Keep the atlas roughly square, in powers of two, which keeps the byte offsets tidy
    let mut cols = 1;
    while (cols * cols) < cells as i32 {
        cols *= 2;
    }
    cols
}

/// Loads an RGBA image from a PNG file, returning `None` for any problem at all: a missing
pub fn load_png(path: &Path) -> Option<(Vec<u8>, i32, i32)> {
    let file = std::fs::File::open(path).ok()?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let palette = reader.info().palette.clone();
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as i32, info.height as i32);
    let bytes = &buf[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => bytes
            .chunks_exact(1)
            .flat_map(|p| [p[0], p[0], p[0], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => {
            let palette = palette.as_ref()?;
            let mut out = Vec::with_capacity(bytes.len() * 4);
            for &i in bytes {
                let base = i as usize * 3;
                if base + 2 < palette.len() {
                    out.extend_from_slice(&[
                        palette[base],
                        palette[base + 1],
                        palette[base + 2],
                        255,
                    ]);
                } else {
                    out.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
            out
        }
    };

    Some((rgba, w, h))
}

/// Looks for a downloaded CC0 image by name in the asset directory. Returns `None` when the
pub fn cc0_image(name: &str) -> Option<(Vec<u8>, i32, i32)> {
    let path: PathBuf = Path::new(CC0_DIR).join(format!("{name}.png"));
    load_png(&path)
}

/// Writes an RGBA image to a PNG, used by the headless frame harness
pub fn save_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    Ok(())
}
