//! Orientation of the rendered view: which world direction each side of the screen shows.
use raycaster::{Raycaster, Tile, WorldMap};

const WIDTH: usize = 64;
const HEIGHT: usize = 40;

fn solid_image(world: &mut WorldMap, color: [u8; 4]) -> usize {
    let mut data = Vec::new();
    for _ in 0..16 {
        data.extend_from_slice(&color);
    }
    world.add_image(data, 4, 4)
}

fn render_north() -> Vec<u8> {
    let mut world = WorldMap::new();
    world.set_fog([255, 255, 255, 255], 500.0);

    let red = solid_image(&mut world, [255, 0, 0, 255]);
    let blue = solid_image(&mut world, [0, 0, 255, 255]);
    let grey = solid_image(&mut world, [60, 60, 60, 255]);

    // North wall at y = 0: red to the west, blue to the east
    for x in 0..10 {
        let image = if x < 5 { red } else { blue };
        world.set_wall(x, 0, Tile::textured(image, (0, 0, 4, 4)));
    }
    for i in 0..10 {
        world.set_wall(i, 9, Tile::textured(grey, (0, 0, 4, 4)));
        world.set_wall(0, i, Tile::textured(grey, (0, 0, 4, 4)));
        world.set_wall(9, i, Tile::textured(grey, (0, 0, 4, 4)));
    }

    let mut caster = Raycaster::new();
    caster.set_pos(4.5, 5.5);
    caster.set_dir_angle(-std::f32::consts::FRAC_PI_2); // facing north, -y

    let mut frame = vec![0u8; WIDTH * HEIGHT * 4];
    caster.render(&mut frame, (0, 0, WIDTH, HEIGHT), WIDTH, &mut world);
    frame
}

fn mean_x_of(frame: &[u8], target: [u8; 3]) -> Option<f32> {
    let mut sum = 0.0;
    let mut count = 0.0;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let off = (y * WIDTH + x) * 4;
            let (r, g, b) = (frame[off], frame[off + 1], frame[off + 2]);
            let near = (r as i32 - target[0] as i32).abs() < 60
                && (g as i32 - target[1] as i32).abs() < 60
                && (b as i32 - target[2] as i32).abs() < 60;
            if near {
                sum += x as f32;
                count += 1.0;
            }
        }
    }
    (count > 0.0).then_some(sum / count)
}

#[test]
fn west_is_on_the_left_when_facing_north() {
    let frame = render_north();
    let red = mean_x_of(&frame, [255, 0, 0]).expect("the red half of the north wall is visible");
    let blue = mean_x_of(&frame, [0, 0, 255]).expect("the blue half of the north wall is visible");

    assert!(
        red < WIDTH as f32 / 2.0 && blue > WIDTH as f32 / 2.0,
        "facing north, west (red) must be left of east (blue); \
         red centred at {red:.1}, blue at {blue:.1}"
    );
}

fn render_at(angle: f32) -> Vec<u8> {
    let mut world = WorldMap::new();
    world.set_fog([255, 255, 255, 255], 500.0);
    let red = solid_image(&mut world, [255, 0, 0, 255]);
    let grey = solid_image(&mut world, [60, 60, 60, 255]);
    for x in 0..10 {
        world.set_wall(
            x,
            0,
            Tile::textured(if x < 3 { red } else { grey }, (0, 0, 4, 4)),
        );
    }
    for i in 0..10 {
        world.set_wall(i, 9, Tile::textured(grey, (0, 0, 4, 4)));
        world.set_wall(0, i, Tile::textured(grey, (0, 0, 4, 4)));
        world.set_wall(9, i, Tile::textured(grey, (0, 0, 4, 4)));
    }
    let mut caster = Raycaster::new();
    caster.set_pos(4.5, 5.5);
    caster.set_dir_angle(angle);
    let mut frame = vec![0u8; WIDTH * HEIGHT * 4];
    caster.render(&mut frame, (0, 0, WIDTH, HEIGHT), WIDTH, &mut world);
    frame
}

#[test]
fn increasing_the_angle_turns_right() {
    let before = mean_x_of(&render_at(-std::f32::consts::FRAC_PI_2), [255, 0, 0]).unwrap();
    let after = mean_x_of(&render_at(-std::f32::consts::FRAC_PI_2 + 0.2), [255, 0, 0]).unwrap();
    // Turning right slides the scene left, so the landmark moves towards x = 0
    assert!(
        after < before,
        "a larger angle must turn right; the landmark went from x={before:.1} to x={after:.1}"
    );
}

#[test]
fn a_sprite_east_of_the_camera_lands_on_the_right() {
    use raycaster::Sprite;

    let mut world = WorldMap::new();
    world.set_fog([255, 255, 255, 255], 500.0);
    let green = solid_image(&mut world, [0, 255, 0, 255]);
    let grey = solid_image(&mut world, [60, 60, 60, 255]);

    // A wall ahead of the camera so the z-buffer has something to compare against
    for x in 0..10 {
        world.set_wall(x, 0, Tile::textured(grey, (0, 0, 4, 4)));
    }
    // The sprite sits east of the camera, which faces north
    world.add_sprite(Sprite::new(5.5, 3.0, Tile::textured(green, (0, 0, 4, 4))));

    let mut caster = Raycaster::new();
    caster.set_pos(4.5, 5.5);
    caster.set_dir_angle(-std::f32::consts::FRAC_PI_2);

    let mut frame = vec![0u8; WIDTH * HEIGHT * 4];
    caster.render(&mut frame, (0, 0, WIDTH, HEIGHT), WIDTH, &mut world);

    let green_x = mean_x_of(&frame, [0, 255, 0]).expect("the sprite is visible");
    assert!(
        green_x > WIDTH as f32 / 2.0,
        "facing north, a sprite to the east must be right of centre, got x={green_x:.1}"
    );
}
