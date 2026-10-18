//! The status bar and the full screen screens, drawn straight into the 320x200 frame.

use crate::assets::font::{draw_centered, draw_number, draw_text, text_width};
use crate::assets::sprites::{draw_face, WeaponKind};
use crate::assets::{Assets, Canvas};
use crate::game::{Game, Mode};
use crate::level::Key;
use crate::render::{Renderer, FRAME_HEIGHT, FRAME_WIDTH, VIEW_HEIGHT};

const BAR_TOP: i32 = VIEW_HEIGHT as i32;
const BAR_HEIGHT: i32 = (FRAME_HEIGHT as i32) - BAR_TOP;

const BAR_BROWN: [u8; 4] = [104, 82, 58, 255];
const BAR_DARK: [u8; 4] = [54, 42, 30, 255];
const BAR_LIGHT: [u8; 4] = [140, 114, 82, 255];
/// Where the difficulty list starts and how far apart its rows are
const SELECTOR_TOP: i32 = 118;
const SELECTOR_ROW: i32 = 17;

const LABEL: [u8; 4] = [196, 176, 140, 255];
const VALUE: [u8; 4] = [236, 232, 220, 255];
const GOLD: [u8; 4] = [214, 176, 72, 255];

/// Draws the weapon overlay, the status bar and whatever screen the game is showing
pub fn draw(frame: &mut [u8], _assets: &Assets, renderer: &Renderer, game: &Game) {
    if matches!(game.mode, Mode::Playing | Mode::Paused | Mode::Dying) {
        renderer.draw_weapon(game, frame);
    }

    if game.muzzle_flash > 0.0 && matches!(game.mode, Mode::Playing | Mode::Paused) {
        let strength = (game.muzzle_flash * 90.0) as u8;
        for y in 0..VIEW_HEIGHT {
            for x in 0..FRAME_WIDTH {
                let offset = (y * FRAME_WIDTH + x) * 4;
                frame[offset] = frame[offset].saturating_add(strength);
                frame[offset + 1] = frame[offset + 1].saturating_add(strength);
            }
        }
    }

    let mut canvas = Canvas::new(
        frame,
        FRAME_WIDTH as i32,
        0,
        0,
        FRAME_WIDTH as i32,
        FRAME_HEIGHT as i32,
    );

    if !matches!(
        game.mode,
        Mode::DifficultySelect | Mode::Victory | Mode::GameOver
    ) {
        draw_status_bar(&mut canvas, game);
    }

    match game.mode {
        Mode::DifficultySelect => draw_difficulty_screen(&mut canvas, game),
        Mode::Intermission => draw_intermission(&mut canvas, game),
        Mode::GameOver => draw_game_over(&mut canvas, game),
        Mode::Victory => draw_victory(&mut canvas, game),
        Mode::Paused => draw_paused(&mut canvas),
        Mode::Dying => draw_death_fade(&mut canvas, game),
        _ => {}
    }

    if matches!(game.mode, Mode::Playing | Mode::Paused) {
        if game.intro_timer > 0.0 {
            draw_intro(&mut canvas, game);
        }
        if let Some((message, _)) = &game.message {
            let width = text_width(message, 2) + 16;
            let x = (FRAME_WIDTH as i32 - width) / 2;
            canvas.rect(x, 60, width, 18, [16, 12, 10, 220]);
            canvas.frame(x, 60, width, 18, GOLD);
            draw_centered(&mut canvas, 66, message, 2, VALUE);
        }
    }
}

fn draw_status_bar(c: &mut Canvas, game: &Game) {
    let player = &game.player;

    c.rect(0, BAR_TOP, FRAME_WIDTH as i32, BAR_HEIGHT, BAR_BROWN);
    c.hline(0, BAR_TOP, FRAME_WIDTH as i32, BAR_LIGHT);
    c.hline(0, FRAME_HEIGHT as i32 - 1, FRAME_WIDTH as i32, BAR_DARK);
    c.vline(0, BAR_TOP, BAR_HEIGHT, BAR_LIGHT);
    c.vline(FRAME_WIDTH as i32 - 1, BAR_TOP, BAR_HEIGHT, BAR_DARK);

    // --- The face, which reacts to health, to firing and to being hit
    let (health, face_x, face_y) = (player.health, 4, BAR_TOP + 4);
    draw_face(
        c,
        face_x,
        face_y,
        health,
        player.face_look,
        player.face_mood(),
    );

    let label = |c: &mut Canvas, x: i32, text: &str| draw_text(c, x, BAR_TOP + 2, text, 1, LABEL);
    let value_y = BAR_TOP + 10;

    label(c, 32, "SCORE");
    draw_number(c, 32, value_y, 66, player.score, 2, VALUE);

    label(c, 104, "LIVES");
    draw_number(c, 104, value_y, 28, player.lives, 2, VALUE);

    label(c, 140, "FLOOR");
    draw_number(c, 140, value_y, 30, game.floor_index as i32 + 1, 2, VALUE);

    label(c, 178, "HEALTH");
    draw_number(
        c,
        178,
        value_y,
        44,
        player.health,
        2,
        if health <= 25 {
            [230, 90, 70, 255]
        } else {
            VALUE
        },
    );

    label(c, 232, "AMMO");
    draw_number(c, 232, value_y, 36, player.weapon.ammo, 2, VALUE);

    // --- Key slots, one for each key of the original
    label(c, 278, "KEYS");
    let mut slot = 280;
    for key in [Key::Gold, Key::Silver] {
        let owned = player.has_key(key);
        let color = if owned {
            match key {
                Key::Gold => GOLD,
                Key::Silver => [198, 202, 210, 255],
            }
        } else {
            [92, 76, 58, 255]
        };

        let (kx, ky) = (slot, BAR_TOP + 10);
        c.circle(kx + 4, ky + 1, 3, color);
        c.circle(kx + 4, ky + 1, 1, BAR_BROWN);
        c.rect(kx + 3, ky + 4, 2, 9, color);
        c.rect(kx + 5, ky + 8, 3, 2, color);
        c.rect(kx + 5, ky + 11, 2, 2, color);

        slot += 14;
    }

    // --- The four weapon boxes, with the one in hand highlighted
    let mut x = 32;
    for kind in WeaponKind::ALL {
        let owned = player.weapon.owned[kind.index()];
        let selected = player.weapon.kind == kind;
        let (width, height) = (40, 12);

        c.rect(
            x,
            BAR_TOP + 26,
            width,
            height,
            if owned {
                [48, 62, 84, 255]
            } else {
                [58, 48, 38, 255]
            },
        );
        c.frame(
            x,
            BAR_TOP + 26,
            width,
            height,
            if selected { GOLD } else { BAR_DARK },
        );

        let name = short_name(kind);
        let color = if !owned {
            [104, 92, 78, 255]
        } else if selected {
            GOLD
        } else {
            VALUE
        };
        draw_text(c, x + 2, BAR_TOP + 28, name, 1, color);

        x += width + 3;
    }
}

fn short_name(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Knife => "KNIFE",
        WeaponKind::Pistol => "PISTOL",
        WeaponKind::MachineGun => "MCHN",
        WeaponKind::Chaingun => "CHAIN",
    }
}

fn draw_intro(c: &mut Canvas, game: &Game) {
    c.rect(0, 52, FRAME_WIDTH as i32, 48, [10, 8, 8, 210]);
    draw_centered(c, 58, game.floor_name(), 3, VALUE);
    draw_centered(c, 84, "GET READY", 1, LABEL);
}

fn draw_difficulty_screen(c: &mut Canvas, game: &Game) {
    c.fill([8, 8, 12, 255]);

    // Two lines, because the whole title does not fit across 320 pixels at this size
    draw_centered(c, 12, "RAYCASTED", 5, GOLD);
    draw_centered(c, 48, "WOLF", 4, [168, 132, 52, 255]);
    draw_centered(c, 84, "A WOLFENSTEIN 3D CLONE IN RUST", 1, LABEL);

    draw_centered(c, 100, "HOW TOUGH DO YOU WANT IT?", 2, VALUE);

    let mut y = SELECTOR_TOP;
    for (index, choice) in crate::difficulty::Difficulty::ALL.iter().enumerate() {
        let name = choice.name();
        let selected = index == game.difficulty_index();
        let color = if selected { GOLD } else { [150, 146, 138, 255] };

        if selected {
            let width = text_width(name, 2) + 16;
            let left = (FRAME_WIDTH as i32 - width) / 2;
            c.rect(left, y - 2, width, 18, [42, 36, 30, 255]);
            c.frame(left, y - 2, width, 18, [92, 76, 44, 255]);
            draw_text(c, left + 4, y + 4, ">", 1, GOLD);
        }
        draw_centered(c, y, name, 2, color);
        y += SELECTOR_ROW;
    }

    draw_centered(c, 186, "UP AND DOWN TO CHOOSE, ENTER TO START", 1, LABEL);
}

fn draw_intermission(c: &mut Canvas, game: &Game) {
    c.fill([16, 16, 34, 255]);

    draw_centered(c, 8, game.floor_name(), 3, GOLD);
    draw_centered(c, 34, "FLOOR COMPLETE", 2, VALUE);

    c.hline(30, 54, 260, [40, 40, 66, 255]);

    let stats = &game.stats;
    let rows = [
        (
            "TIME",
            format_time(stats.time),
            format!("PAR {}", format_time(stats.par_time)),
        ),
        (
            "KILLS",
            format!("{}%", stats.kill_percent()),
            format!("{}/{}", stats.kills, stats.kills_total),
        ),
        (
            "SECRETS",
            format!("{}%", stats.secret_percent()),
            format!("{}/{}", stats.secrets, stats.secrets_total),
        ),
        (
            "TREASURE",
            format!("{}%", stats.treasure_percent()),
            format!("{}/{}", stats.treasure, stats.treasure_total),
        ),
    ];

    let mut y = 62;
    for (label, value, detail) in rows {
        draw_text(c, 36, y, label, 2, LABEL);
        draw_text(c, 150, y, &value, 2, VALUE);
        draw_text(c, 236, y + 3, &detail, 1, [148, 144, 136, 255]);
        y += 18;
    }

    c.hline(30, y + 2, 260, [40, 40, 66, 255]);

    draw_text(c, 36, y + 8, "BONUS", 2, LABEL);
    draw_text(c, 150, y + 8, &format!("{:>6}", game.stats.bonus), 2, GOLD);
    draw_text(c, 36, y + 28, "SCORE", 2, LABEL);
    draw_text(
        c,
        150,
        y + 28,
        &format!("{:>6}", game.player.score),
        2,
        GOLD,
    );

    draw_centered(c, 186, "PRESS ENTER TO TAKE THE ELEVATOR", 1, LABEL);
}

fn draw_game_over(c: &mut Canvas, game: &Game) {
    c.fill([8, 8, 12, 255]);
    draw_centered(c, 52, "GAME OVER", 5, [200, 60, 50, 255]);
    draw_centered(c, 100, "THE GUARDS WIN THIS TIME", 2, VALUE);
    draw_centered(
        c,
        122,
        &format!("FINAL SCORE {}", game.player.score),
        2,
        GOLD,
    );
    draw_centered(c, 168, "PRESS ENTER FOR ANOTHER GO", 1, LABEL);
}

fn draw_victory(c: &mut Canvas, game: &Game) {
    c.fill([12, 16, 20, 255]);
    draw_centered(c, 36, "YOU ESCAPED", 4, GOLD);
    draw_centered(c, 76, "ALL THREE FLOORS CLEARED", 2, VALUE);
    draw_centered(
        c,
        100,
        &format!("FINAL SCORE {}", game.player.score),
        2,
        GOLD,
    );
    draw_centered(c, 124, &format!("ON {}", game.difficulty.name()), 1, LABEL);
    draw_centered(c, 168, "PRESS ENTER TO PLAY AGAIN", 1, LABEL);
}

fn draw_paused(c: &mut Canvas) {
    c.rect(86, 58, 148, 46, [10, 8, 8, 220]);
    c.frame(86, 58, 148, 46, GOLD);
    draw_centered(c, 64, "PAUSED", 3, VALUE);
    draw_centered(c, 88, "ESC RESUMES, Q QUITS", 1, LABEL);
}

fn draw_death_fade(c: &mut Canvas, game: &Game) {
    let strength = (game.player.death_timer / 2.0).min(1.0);
    let dark = (strength * 150.0) as u8;
    for y in 0..VIEW_HEIGHT {
        for x in 0..FRAME_WIDTH {
            let offset = (y * FRAME_WIDTH + x) * 4;
            if offset + 4 > c.data.len() {
                continue;
            }
            c.data[offset] = c.data[offset].saturating_sub(dark / 3);
            c.data[offset + 1] = c.data[offset + 1].saturating_sub(dark);
            c.data[offset + 2] = c.data[offset + 2].saturating_sub(dark);
        }
    }

    if strength > 0.5 {
        draw_centered(c, 70, "YOU DIED", 4, [200, 60, 50, 255]);
    }
}

fn format_time(seconds: f32) -> String {
    let total = seconds.max(0.0) as i32;
    format!("{}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::font::text_height;

    /// The fixed strings the full screens draw, with the scale each is drawn at. Sizes are part of
    const LINES: [(&str, i32); 17] = [
        ("RAYCASTED", 5),
        ("WOLF", 4),
        ("A WOLFENSTEIN 3D CLONE IN RUST", 1),
        ("HOW TOUGH DO YOU WANT IT?", 2),
        ("UP AND DOWN TO CHOOSE, ENTER TO START", 1),
        ("FLOOR COMPLETE", 2),
        ("PRESS ENTER TO TAKE THE ELEVATOR", 1),
        ("GAME OVER", 5),
        ("THE GUARDS WIN THIS TIME", 2),
        ("PRESS ENTER FOR ANOTHER GO", 1),
        ("YOU ESCAPED", 4),
        ("ALL THREE FLOORS CLEARED", 2),
        ("PRESS ENTER TO PLAY AGAIN", 1),
        ("PAUSED", 3),
        ("ESC RESUMES, Q QUITS", 1),
        ("YOU DIED", 4),
        ("GET READY", 1),
    ];

    #[test]
    fn no_full_screen_line_runs_off_the_frame() {
        for (text, scale) in LINES {
            let width = text_width(text, scale);
            assert!(
                width <= FRAME_WIDTH as i32 - 8,
                "{text:?} at scale {scale} is {width} pixels wide, the frame is {FRAME_WIDTH}"
            );
        }
    }

    #[test]
    fn the_difficulty_names_fit_the_selector() {
        for choice in crate::difficulty::Difficulty::ALL {
            let width = text_width(choice.name(), 2) + 16;
            assert!(
                width <= FRAME_WIDTH as i32 - 16,
                "{} at scale 2 is {width} wide with its highlight frame",
                choice.name()
            );
        }
    }

    #[test]
    fn the_selector_frame_clears_the_text_and_the_row_below() {
        let options = crate::difficulty::Difficulty::ALL.len() as i32;
        let list_bottom = SELECTOR_TOP + (options - 1) * SELECTOR_ROW + 16;
        assert!(
            list_bottom <= 186,
            "the difficulty list ends at {list_bottom}, on top of the footer line"
        );
        assert!(
            SELECTOR_ROW >= text_height(2) + 2,
            "rows are {} apart and the text is {} tall",
            SELECTOR_ROW,
            text_height(2)
        );
    }

    #[test]
    fn the_face_labels_clear_the_value_row_and_the_weapon_boxes() {
        let label_bottom = BAR_TOP + 2 + text_height(1);
        assert!(
            label_bottom <= BAR_TOP + 10,
            "the status bar labels overlap their numbers"
        );

        let value_bottom = BAR_TOP + 10 + text_height(2);
        assert!(
            value_bottom <= BAR_TOP + 26,
            "the status bar numbers overlap the weapon boxes"
        );

        let box_bottom = BAR_TOP + 26 + 12;
        assert!(
            box_bottom <= FRAME_HEIGHT as i32,
            "the weapon boxes hang off the bottom of the frame"
        );
    }

    #[test]
    fn the_centre_column_of_text_is_actually_centred() {
        for (text, scale) in LINES {
            let width = text_width(text, scale);
            let left = (FRAME_WIDTH as i32 - width) / 2;
            let right = left + width;
            let right_margin = FRAME_WIDTH as i32 - right;
            assert!(
                (left - right_margin).abs() <= 1,
                "{text:?} sits {left} from the left and {right_margin} from the right"
            );
        }
    }
}
