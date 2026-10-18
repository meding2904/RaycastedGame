//! A 5x7 bitmap font for the status bar and the full screen messages.

use super::Canvas;

/// How far the pen advances per character, before scaling: five columns plus a one pixel gap
pub const ADVANCE: i32 = 6;

/// The rows of a glyph, top row first. Bit 4 is the leftmost column, bit 0 the rightmost.
pub fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b01111, 0b00001, 0b00110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        ':' => [
            0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00110, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        '%' => [
            0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011,
        ],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '>' => [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
        '<' => [
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ],
        // The difficulty names have apostrophes in them, so a blank here would read as a gap
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '"' => [
            0b01010, 0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '*' => [
            0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '#' => [
            0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000,
        ],
        _ => [0; 7],
    }
}

/// The width of a drawn string in pixels for a given scale, including the one pixel gap
pub fn text_width(text: &str, scale: i32) -> i32 {
    let chars = text.chars().count() as i32;
    if chars == 0 {
        return 0;
    }
    chars * ADVANCE * scale - scale
}

/// The height of a drawn string in pixels for a given scale
pub fn text_height(scale: i32) -> i32 {
    7 * scale
}

pub fn draw_text(c: &mut Canvas, x: i32, y: i32, text: &str, scale: i32, color: [u8; 4]) {
    let mut cursor = x;

    for ch in text.chars() {
        if ch != ' ' {
            let rows = glyph(ch);
            for (row, bits) in rows.iter().enumerate() {
                if *bits == 0 {
                    continue;
                }
                // tables are most significant bit first, so column 0 reads bit 4 -- getting this
                // backwards painted every screen as a mirror image, which is easy to miss when a
                let bit = |col: i32| 1 << (4 - col);
                let mut col = 0;
                while col < 5 {
                    if bits & bit(col) == 0 {
                        col += 1;
                        continue;
                    }
                    let start = col;
                    while col < 5 && bits & bit(col) != 0 {
                        col += 1;
                    }
                    c.rect(
                        cursor + start * scale,
                        y + row as i32 * scale,
                        (col - start) * scale,
                        scale,
                        color,
                    );
                }
            }
        }
        cursor += ADVANCE * scale;
    }
}

/// Draws text with a one pixel hard shadow, which stays readable over textures
pub fn draw_text_shadow(c: &mut Canvas, x: i32, y: i32, text: &str, scale: i32, color: [u8; 4]) {
    draw_text(c, x + scale, y + scale, text, scale, [20, 16, 14, 255]);
    draw_text(c, x, y, text, scale, color);
}

/// Draws a number, right aligned inside `width` pixels
pub fn draw_number(
    c: &mut Canvas,
    x: i32,
    y: i32,
    width: i32,
    value: i32,
    scale: i32,
    color: [u8; 4],
) {
    let text = value.to_string();
    let w = text_width(&text, scale);
    draw_text_shadow(c, x + width - w, y, &text, scale, color);
}

/// Draws centered text across the full 320 pixel frame width
pub fn draw_centered(c: &mut Canvas, y: i32, text: &str, scale: i32, color: [u8; 4]) {
    let w = text_width(text, scale);
    draw_text_shadow(c, (320 - w) / 2, y, text, scale, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_matches_its_table() {
        for ch in ('A'..='Z').chain('0'..='9') {
            let expected: Vec<String> = glyph(ch)
                .iter()
                .map(|row| {
                    (0..5)
                        .map(|col| {
                            if row & (0b10000 >> col) != 0 {
                                '#'
                            } else {
                                '.'
                            }
                        })
                        .collect()
                })
                .collect();
            assert_eq!(
                painted_bitmap(ch, 1),
                expected,
                "{ch} is not painted as written"
            );
        }
    }

    #[test]
    fn the_whole_upper_alphabet_has_a_glyph() {
        for ch in ('A'..='Z').chain('0'..='9') {
            assert!(
                glyph(ch).iter().any(|row| *row != 0),
                "{ch} is a blank glyph"
            );
        }
    }

    #[test]
    fn glyphs_never_leave_the_five_column_box() {
        for ch in ('A'..='Z').chain('0'..='9').chain([
            ':', '.', ',', '-', '!', '?', '%', '/', '+', '(', ')', '>', '<', '\'', '"', '*', '=',
            '&', '#',
        ]) {
            for row in glyph(ch) {
                assert!(row & !0b11111 == 0, "{ch} has bits outside the glyph box");
            }
        }
    }

    #[test]
    fn every_character_the_screens_draw_has_a_glyph() {
        let punctuation = [
            ':', '.', ',', '-', '!', '?', '%', '/', '+', '(', ')', '>', '<', '\'', '"', '*', '=',
            '&', '#',
        ];
        for ch in ('A'..='Z')
            .chain('a'..='z')
            .chain('0'..='9')
            .chain(punctuation)
        {
            assert!(
                glyph(ch).iter().any(|row| *row != 0),
                "{ch} would be drawn as a blank"
            );
        }
    }

    #[test]
    fn a_string_is_as_wide_as_its_glyphs_and_gaps() {
        assert_eq!(text_width("", 1), 0);
        assert_eq!(text_width("A", 1), 5);
        assert_eq!(text_width("AB", 1), 5 + 1 + 5);
        assert_eq!(text_width("AB", 3), 2 * ADVANCE * 3 - 3);
        assert_eq!(text_height(2), 14);
    }

    /// One character, drawn at the given scale, read back as rows of `#` and `.` exactly as they
    fn painted_bitmap(ch: char, scale: i32) -> Vec<String> {
        let (width, height) = (32, 16);
        let mut data = vec![0u8; width * height * 4];
        {
            let mut c = Canvas::new(&mut data, width as i32, 0, 0, width as i32, height as i32);
            draw_text(&mut c, 0, 0, &ch.to_string(), scale, [255, 255, 255, 255]);
        }
        (0..7)
            .map(|row| {
                (0..5)
                    .map(|col| {
                        let off = (((row * scale) as usize * width) + (col * scale) as usize) * 4;
                        if data[off] != 0 {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn text_is_not_mirrored() {
        // The tables are most significant bit first. Reading them the other way round mirrors
        let l = painted_bitmap('L', 1);
        for (row, line) in l.iter().take(6).enumerate() {
            assert_eq!(
                line, "#....",
                "L's stem belongs in the left column, row {row} reads {line}"
            );
        }

        let s = painted_bitmap('S', 1);
        assert_eq!(s[0], ".####", "S's top bar should close on the right");
        assert_eq!(s[6], "####.", "S's bottom bar should close on the left");

        let j = painted_bitmap('J', 1);
        assert_eq!(j[0], "..###", "J's top bar belongs on the right");

        let mut data = vec![0u8; 32 * 16 * 4];
        {
            let mut c = Canvas::new(&mut data, 32, 0, 0, 32, 16);
            draw_text(&mut c, 0, 0, "AB", 1, [255, 255, 255, 255]);
        }
        let second = ADVANCE as usize * 4;
        assert!(
            data[second] != 0 || data[second + 4 * 32] != 0,
            "the second letter of AB should start at column {ADVANCE}"
        );
        assert_eq!(
            data[second - 4],
            0,
            "the gap between letters should stay empty"
        );
    }

    #[test]
    fn text_is_not_upside_down() {
        let l = painted_bitmap('L', 1);
        assert_eq!(l[0], "#....", "the top of L is its stem, not its foot");
        assert_eq!(l[6], "#####", "L's foot belongs on the bottom row");

        let t = painted_bitmap('T', 1);
        assert_eq!(t[0], "#####", "T's bar belongs on top");
        assert_eq!(t[6], "..#..", "T's stem belongs at the bottom");
    }

    #[test]
    fn letters_are_tall_enough_to_read_at_scale_one() {
        assert!(text_height(1) >= 7);
    }
}
