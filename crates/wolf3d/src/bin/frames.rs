#![forbid(unsafe_code)]
//! Headless verification harness.

use std::path::PathBuf;
use wolf3d::assets::font;
use wolf3d::assets::sprites::WeaponKind;
use wolf3d::assets::{save_png, sfx, sprites, textures, Assets, Atlas, Canvas};
use wolf3d::difficulty::Difficulty;
use wolf3d::game::{Game, Mode};
use wolf3d::hud;
use wolf3d::player::Player;
use wolf3d::render::{Renderer, FRAME_HEIGHT, FRAME_WIDTH};

/// The workspace root, resolved at compile time so the harness works from any directory
fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("gallery")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = arg_value(&args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);

    let assets_only = args.iter().any(|a| a == "--assets");
    let ascii = args.iter().any(|a| a == "--ascii");
    let maps = args.iter().any(|a| a == "--maps");
    let frames = args.iter().any(|a| a == "--frames");

    if let Some(path) = arg_value(&args, "--png") {
        match wolf3d::assets::load_png(std::path::Path::new(&path)) {
            Some((data, width, height)) => {
                println!("{path}: {width}x{height}");
                let (mut x0, mut y0, mut crop_w, mut crop_h) = (0, 0, width, height);
                if let Some(crop) = arg_value(&args, "--crop") {
                    let parts: Vec<i32> = crop
                        .split(',')
                        .filter_map(|part| part.trim().parse().ok())
                        .collect();
                    if parts.len() == 4 {
                        x0 = parts[0].clamp(0, width - 1);
                        y0 = parts[1].clamp(0, height - 1);
                        crop_w = parts[2].clamp(1, width - x0);
                        crop_h = parts[3].clamp(1, height - y0);
                        println!("crop {x0},{y0} {crop_w}x{crop_h}");
                    }
                }

                let mut crop = Vec::with_capacity((crop_w * crop_h * 4) as usize);
                for y in y0..y0 + crop_h {
                    for x in x0..x0 + crop_w {
                        let off = ((y * width + x) * 4) as usize;
                        crop.extend_from_slice(&data[off..off + 4]);
                    }
                }

                let step = arg_value(&args, "--step")
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or_else(|| (crop_w / 100).max(1));
                print!("{}", ascii_frame(&crop, crop_w, crop_h, step));
            }
            None => eprintln!("could not read {path}"),
        }
        return;
    }

    if args.iter().any(|argument| argument == "--names") {
        // cannot drift from the code that reads it
        println!("# textures -> assets/cc0/<name>.png (64x64)");
        for tex in textures::Tex::ALL {
            println!("{}", tex.name());
        }
        println!("# objects -> assets/cc0/<name>.png (64x64)");
        for kind in sprites::ObjectKind::ALL {
            println!("{}", kind.name());
        }
        println!(
            "# actors -> assets/cc0/<kind>_<view>_<slot>.png (64x64, view 0-7, slot per pose)"
        );
        for kind in sprites::ActorKind::ALL {
            println!("{}_0_0", kind.name().replace(' ', "_"));
        }
        return;
    }

    if args.iter().any(|argument| argument == "--font") {
        print_font();
        return;
    }

    if maps {
        print_maps();
        return;
    }

    dump_assets(&out);

    if ascii {
        preview_art();
    }

    if frames {
        render_scenes(&out, ascii);
    }

    if !assets_only && !ascii && !frames {
        println!("nothing to do: pass --assets, --ascii, --frames or --maps");
    }
}

/// Draws the whole character set through `draw_text` and prints it, which is the only way to
fn print_font() {
    let mut data = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT * 4];
    let characters: Vec<char> = ('A'..='Z')
        .chain('0'..='9')
        .chain([
            ':', '.', ',', '-', '!', '?', '%', '/', '+', '(', ')', '>', '<', '\'', '"',
        ])
        .collect();

    let per_row = 12;
    let mut y = 4;
    for chunk in characters.chunks(per_row) {
        let mut x = 4;
        {
            let mut canvas = Canvas::new(
                &mut data,
                FRAME_WIDTH as i32,
                0,
                0,
                FRAME_WIDTH as i32,
                FRAME_HEIGHT as i32,
            );
            for ch in chunk {
                canvas.rect(x - 2, y - 2, 9, 11, [40, 40, 48, 255]);
                font::draw_text(&mut canvas, x, y, &ch.to_string(), 1, [236, 232, 220, 255]);
                x += 12;
            }
        }
        let line_y = y as usize;
        for row in 0..7usize {
            let line: String = (4..4 + per_row * 12)
                .map(|px| {
                    let off = ((line_y + row) * FRAME_WIDTH + px) * 4;
                    if data[off] > 100 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect();
            println!("{line}");
        }
        println!();
        y += 14;
    }

    println!("legend: rows 1-2 of each block are A..L, then M..X, then Y,Z and punctuation");
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn dump_atlas(out: &std::path::Path, atlas: &Atlas, file: &str) {
    let path = out.join(file);
    match save_png(&path, &atlas.data, atlas.width as u32, atlas.height as u32) {
        Ok(()) => println!(
            "wrote {} ({}x{})",
            path.display(),
            atlas.width,
            atlas.height
        ),
        Err(err) => eprintln!("could not write {}: {err}", path.display()),
    }
}

fn dump_assets(out: &std::path::Path) {
    let textures = textures::Textures::generate();
    let actors = sprites::ActorSprites::generate();
    let objects = sprites::ObjectSprites::generate();
    let weapons = sprites::WeaponSprites::generate();
    let sounds = sfx::Sounds::generate();

    dump_atlas(out, &textures.atlas, "atlas_textures.png");
    dump_atlas(out, &actors.atlas, "atlas_actors.png");
    dump_atlas(out, &objects.atlas, "atlas_objects.png");
    dump_atlas(out, &weapons.atlas, "atlas_weapons.png");

    if let Err(err) = write_wav(&out.join("music.wav"), &sounds.music, sfx::SAMPLE_RATE) {
        eprintln!("could not write music.wav: {err}");
    }
}

const RAMP: &[u8] = b" .:-=+*#%@";

/// Averages an area of an RGBA image into one character of the ramp. Fully transparent areas
fn ascii_image(data: &[u8], width: i32, height: i32, step: i32) -> String {
    let mut out = String::new();

    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let (mut luma, mut covered, mut count) = (0.0f32, 0, 0);
            for dy in 0..step {
                for dx in 0..step {
                    let (px, py) = (x + dx, y + dy);
                    if px >= width || py >= height {
                        continue;
                    }
                    let off = ((py * width + px) * 4) as usize;
                    if off + 4 > data.len() {
                        continue;
                    }
                    count += 1;
                    if data[off + 3] > 24 {
                        covered += 1;
                        luma += 0.299 * data[off] as f32
                            + 0.587 * data[off + 1] as f32
                            + 0.114 * data[off + 2] as f32;
                    }
                }
            }

            if count == 0 || covered == 0 {
                out.push(' ');
            } else {
                let luma = luma / covered as f32 / 255.0;
                let index = ((luma * (RAMP.len() - 1) as f32).round() as usize).min(RAMP.len() - 1);
                out.push(RAMP[index] as char);
            }
            x += step;
        }
        out.push('\n');
        y += step;
    }

    out
}

/// Like `ascii_image`, but stretches the contrast across the whole ramp first. Retro shooters are
fn ascii_frame(data: &[u8], width: i32, height: i32, step: i32) -> String {
    let mut blocks: Vec<Vec<(i32, i32, f32)>> = Vec::new();

    let mut y = 0;
    while y < height {
        let mut row = Vec::new();
        let mut x = 0;
        while x < width {
            let (mut luma, mut count) = (0.0f32, 0);
            for dy in 0..step {
                for dx in 0..step {
                    let (px, py) = (x + dx, y + dy);
                    if px >= width || py >= height {
                        continue;
                    }
                    let off = ((py * width + px) * 4) as usize;
                    if off + 4 > data.len() {
                        continue;
                    }
                    count += 1;
                    luma += 0.299 * data[off] as f32
                        + 0.587 * data[off + 1] as f32
                        + 0.114 * data[off + 2] as f32;
                }
            }
            let value = if count == 0 {
                0.0
            } else {
                luma / count as f32 / 255.0
            };
            row.push((x / step, y / step, value));
            x += step;
        }
        blocks.push(row);
        y += step;
    }

    let mut sorted: Vec<f32> = blocks.iter().flatten().map(|block| block.2).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let low = sorted[sorted.len() / 50];
    let high = sorted[sorted.len() - 1 - sorted.len() / 50].max(low + 0.001);

    let mut out = String::new();
    for row in blocks {
        for (_, _, luma) in row {
            let t = ((luma - low) / (high - low)).clamp(0.0, 1.0);
            let index = (t * (RAMP.len() - 1) as f32).round() as usize;
            out.push(RAMP[index] as char);
        }
        out.push('\n');
    }
    out
}

fn cell_of(atlas: &Atlas, index: usize, cell: i32) -> Vec<u8> {
    let x = (index as i32 % atlas.cols) * atlas.cell;
    let y = (index as i32 / atlas.cols) * atlas.cell;
    let mut out = Vec::with_capacity((cell * cell * 4) as usize);

    for row in 0..cell {
        for col in 0..cell {
            let off = (((y + row) * atlas.width) + x + col) as usize * 4;
            if off + 4 <= atlas.data.len() {
                out.extend_from_slice(&atlas.data[off..off + 4]);
            } else {
                out.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    out
}

fn preview_art() {
    let textures = textures::Textures::generate();
    println!("\n=== wall textures (64x64, sampled every 4th pixel) ===");
    for tex in textures::Tex::ALL {
        println!("\n-- {} --", tex.name());
        print!(
            "{}",
            ascii_image(&cell_of(&textures.atlas, tex.index(), 64), 64, 64, 4)
        );
    }

    let objects = sprites::ObjectSprites::generate();
    println!("\n=== objects (64x64, sampled every 4th pixel) ===");
    for kind in sprites::ObjectKind::ALL {
        println!("\n-- {} --", kind.name());
        print!(
            "{}",
            ascii_image(&cell_of(&objects.atlas, kind.index(), 64), 64, 64, 4)
        );
    }

    let weapons = sprites::WeaponSprites::generate();
    let cell = wolf3d::assets::weapons_art::WEAPON_CELL;
    println!("\n=== weapons ({cell}x{cell}, sampled every 4th pixel) ===");
    for kind in sprites::WeaponKind::ALL {
        for frame in 0..kind.frames() {
            println!("\n-- {} frame {frame} --", kind.name());
            print!(
                "{}",
                ascii_image(&weapon_cell(&weapons, kind, frame), cell, cell, 4)
            );
        }
    }

    let actors = sprites::ActorSprites::generate();
    println!("\n=== actors: standing, walking, firing, wounded, dying ===");
    for kind in sprites::ActorKind::ALL {
        for view in [0usize, 2, 4] {
            for slot in [0usize, 1, 3, 5, 6, 7, 8, 9, 13] {
                println!("\n-- {} view {view} slot {slot} --", kind.name());
                let index = actors.cell(kind, view, slot);
                print!(
                    "{}",
                    ascii_image(&cell_of(&actors.atlas, index, 64), 64, 64, 2)
                );
            }
        }
    }
}

fn weapon_cell(
    weapons: &sprites::WeaponSprites,
    kind: sprites::WeaponKind,
    frame: usize,
) -> Vec<u8> {
    let rect = weapons.tex_rect(kind, frame);
    let cell = wolf3d::assets::weapons_art::WEAPON_CELL as usize;
    let x = rect.0 / 4;
    let y = rect.1 / 4 / weapons.atlas.width as usize;
    let mut out = Vec::with_capacity(cell * cell * 4);

    for row in 0..cell as i32 {
        for col in 0..cell as i32 {
            let off = ((y as i32 + row) * weapons.atlas.width + x as i32 + col) as usize * 4;
            if off + 4 <= weapons.atlas.data.len() {
                out.extend_from_slice(&weapons.atlas.data[off..off + 4]);
            } else {
                out.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    out
}

fn write_wav(path: &std::path::Path, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let data_len = (samples.len() * 2) as u32;
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_len).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?; // PCM
    file.write_all(&1u16.to_le_bytes())?; // mono
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * 2).to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_len.to_le_bytes())?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        file.write_all(&value.to_le_bytes())?;
    }

    Ok(())
}

/// Draws a finished frame exactly the way the window does: view, weapon, then the HUD
fn draw_game_frame(assets: &Assets, renderer: &mut Renderer, game: &Game) -> Vec<u8> {
    let mut frame = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT * 4];
    renderer.enter_floor(game, assets);
    renderer.sync(game, assets);
    Renderer::clear(&mut frame);
    renderer.draw_view(game, &mut frame);
    hud::draw(&mut frame, assets, renderer, game);
    frame
}

fn scene(name: &str, out: &std::path::Path, ascii: bool, assets: &Assets, game: &Game) {
    let mut renderer = Renderer::new(assets);
    let frame = draw_game_frame(assets, &mut renderer, game);

    let path = out.join(format!("game_{name}.png"));
    match save_png(&path, &frame, FRAME_WIDTH as u32, FRAME_HEIGHT as u32) {
        Ok(()) => println!("wrote {} ({})", path.display(), name),
        Err(err) => eprintln!("could not write {}: {err}", path.display()),
    }

    // not render at all, which a PNG alone would not tell us from a screenshot
    let view = &frame[..(FRAME_WIDTH * 160) * 4];
    let lit = view
        .chunks_exact(4)
        .filter(|pixel| pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32 > 60)
        .count();
    println!(
        "  {}: {}% of the view is lit",
        name,
        lit * 100 / (FRAME_WIDTH * 160)
    );

    if ascii {
        println!("\n-- frame: {name} --");
        print!(
            "{}",
            ascii_frame(&frame, FRAME_WIDTH as i32, FRAME_HEIGHT as i32, 4)
        );
    }
}

fn started(difficulty: Difficulty) -> Game {
    let mut game = Game::new();
    game.difficulty = difficulty;
    game.player = Player::new(2.5, 2.5, 0.0);
    game.load_floor(0);
    game.skip_intro();
    game
}

fn render_scenes(out: &std::path::Path, ascii: bool) {
    if let Err(err) = std::fs::create_dir_all(out) {
        eprintln!("could not create {}: {err}", out.display());
        return;
    }
    let assets = Assets::generate();

    let mut title = started(Difficulty::Normal);
    title.mode = Mode::DifficultySelect;
    scene("title", out, ascii, &assets, &title);

    let spawn = started(Difficulty::Normal);
    scene("spawn", out, ascii, &assets, &spawn);

    let mut fight = started(Difficulty::Normal);
    fight.player.x = 11.5;
    fight.player.y = 3.5;
    fight.player.angle = 0.0;
    fight.player.weapon.grant(WeaponKind::MachineGun);
    fight.player.weapon.select(2);
    fight.muzzle_flash = 1.0;
    scene("firefight", out, ascii, &assets, &fight);

    let mut attack = started(Difficulty::Normal);
    attack.player.x = 7.5;
    attack.player.y = 4.5;
    attack.player.angle = 0.0;
    if let Some(actor) = attack.actors.first_mut() {
        actor.x = 10.5;
        actor.y = 4.5;
        actor.angle = std::f32::consts::PI;
        actor.alerted = true;
        actor.state = wolf3d::actors::ActorState::Shoot;
        actor.frame = wolf3d::assets::sprites::FIRE_FRAME;
    }
    scene("guard_attack", out, ascii, &assets, &attack);

    let mut door = started(Difficulty::Normal);
    door.player.x = 7.5;
    door.player.y = 3.5;
    door.player.angle = 0.0;
    if let Some(first) = door.doors.first_mut() {
        first.opening = 0.45;
    }
    scene("door", out, ascii, &assets, &door);

    let mut hurt = started(Difficulty::Hard);
    hurt.player.health = 12;
    hurt.player.hurt(1, Difficulty::Hard);
    scene("hurt", out, ascii, &assets, &hurt);

    let mut tally = started(Difficulty::Normal);
    tally.mode = Mode::Intermission;
    tally.stats.kills = 7;
    tally.stats.kills_total = 9;
    tally.stats.treasure = 4;
    tally.stats.treasure_total = 11;
    tally.stats.secrets = 1;
    tally.stats.secrets_total = 2;
    tally.stats.time = 143.0;
    scene("tally", out, ascii, &assets, &tally);
}

/// Prints every floor as a text map, for working on level layouts
pub fn print_maps() {
    for level in wolf3d::levels::floors() {
        println!(
            "\n=== {} ({} actors, {} treasure, {} secret areas) ===",
            level.name,
            level.actor_count(),
            level.treasure_count(),
            level.secrets.len()
        );
        let (_, _, _) = level.start;
        let mut min_x = 63;
        let mut min_y = 63;
        let mut max_x = 0;
        let mut max_y = 0;
        for y in 0..64 {
            for x in 0..64 {
                let interesting = level.wall(x, y) != wolf3d::level::Wall::Greystone
                    || level.thing(x, y).is_some();
                if interesting {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
        let rendered = level.render_ascii();
        let rows: Vec<&str> = rendered.lines().collect();
        for y in min_y..=max_y {
            let row = rows.get(y as usize).copied().unwrap_or("");
            let chars: Vec<char> = row.chars().collect();
            let slice: String = (min_x..=max_x)
                .filter_map(|x| chars.get(x as usize).copied())
                .collect();
            println!("{y:2} {slice}");
        }
        println!("   columns {min_x}..{max_x}");
    }
}
