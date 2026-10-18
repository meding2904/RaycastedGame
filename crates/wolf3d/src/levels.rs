//! The three hand built floors.

use crate::assets::textures::Tex;
use crate::level::{Key, Level, LevelBuilder, Thing, Wall};

/// A one tile high corridor running east, walled above and below
fn corridor_h(b: &mut LevelBuilder, x0: i32, x1: i32, y: i32, wall: Wall) {
    b.rect(x0, y, x1 - x0 + 1, 1);
    b.hline(x0, x1, y - 1, wall);
    b.hline(x0, x1, y + 1, wall);
}

/// A one tile wide corridor running south, walled left and right
fn corridor_v(b: &mut LevelBuilder, x: i32, y0: i32, y1: i32, wall: Wall) {
    b.rect(x, y0, 1, y1 - y0 + 1);
    b.vline(x - 1, y0, y1, wall);
    b.vline(x + 1, y0, y1, wall);
}

/// Every floor of the game, in order. Floors that fail validation are reported here rather than
pub fn floors() -> Vec<Level> {
    let floors = vec![floor_1(), floor_2(), floor_3()];
    for level in &floors {
        if let Err(problem) = level.validate() {
            eprintln!("level problem: {problem}");
        }
    }
    floors
}

/// Floor 1: the cell, a pushwall to a hidden stash, the guard corridor, the kennels, the shrine
pub fn floor_1() -> Level {
    let mut b = LevelBuilder::new("Floor 1", Tex::FloorStone, Tex::CeilDark);

    b.room(2, 2, 8, 6); // interior 2..9 x 2..7
    b.facing(1, 1, 10, 8, Wall::Greystone);
    b.room(2, 9, 7, 4); // the hidden room, interior 2..8 x 9..12
    b.facing(1, 8, 9, 8, Wall::Brick);
    b.pushwall(5, 8, Wall::Brick);
    b.secret(4, 8, 4, 2);
    b.secret(2, 9, 7, 4);
    b.start(4.5, 4.5, 0.0);

    b.room(28, 2, 12, 12); // interior 28..39 x 2..13
    b.facing(27, 1, 14, 14, Wall::Bluestone);
    b.hline(30, 36, 1, Wall::Banner);
    b.hline(31, 37, 7, Wall::BrickWood);
    b.vline(34, 4, 10, Wall::BrickWood);
    b.region(28, 2, 12, 12, Tex::FloorBlue, Tex::CeilDark);

    b.room(14, 11, 10, 7); // interior 14..23 x 11..17
    b.facing(13, 10, 12, 9, Wall::Brick);
    b.region(14, 11, 10, 7, Tex::FloorWood, Tex::CeilWood);

    b.room(2, 16, 8, 6); // interior 2..9 x 16..21
    b.facing(1, 15, 10, 8, Wall::Brick);

    b.room(14, 22, 10, 7); // interior 14..23 x 22..28
    b.facing(13, 21, 12, 9, Wall::Greystone);
    b.region(14, 22, 10, 7, Tex::FloorBlue, Tex::CeilLight);

    // --- Now the openings
    corridor_h(&mut b, 11, 26, 4, Wall::Brick);
    b.door(10, 4, Wall::Door); // cell -> corridor
    b.door(27, 4, Wall::Door); // corridor -> courtyard
    b.door(18, 5, Wall::Door); // corridor -> kennels
    corridor_v(&mut b, 18, 6, 10, Wall::Brick);
    corridor_h(&mut b, 10, 13, 16, Wall::Brick); // kennels -> gold key room
    b.locked_door(18, 18, Wall::Door, Key::Gold); // kennels -> elevator
    corridor_v(&mut b, 18, 19, 21, Wall::Greystone);
    b.vline(24, 24, 27, Wall::ElevatorSwitch);

    // --- Guards, dogs and the contents of the floor
    b.thing(15, 4, Thing::Guard);
    b.thing(21, 4, Thing::Guard);
    b.thing(25, 4, Thing::AmmoClip);

    b.thing(32, 5, Thing::Officer);
    b.thing(36, 9, Thing::Guard);
    b.thing(31, 11, Thing::Guard);
    b.thing(34, 3, Thing::Pillar);
    b.thing(29, 3, Thing::Pillar);
    b.thing(38, 3, Thing::Pillar);
    b.thing(29, 12, Thing::Pillar);
    b.thing(38, 12, Thing::Pillar);
    b.thing(36, 8, Thing::TreasureChalice);
    b.thing(32, 8, Thing::AmmoBox);
    b.thing(36, 5, Thing::HealthSmall);
    b.thing(30, 6, Thing::Table);
    b.thing(37, 10, Thing::Barrel);

    b.thing(16, 13, Thing::Dog);
    b.thing(21, 13, Thing::Dog);
    b.thing(18, 16, Thing::Dog);
    b.thing(15, 16, Thing::Blood);
    b.thing(22, 16, Thing::AmmoClip);
    b.thing(20, 15, Thing::HealthSmall);

    b.thing(5, 18, Thing::KeyGold);
    b.thing(7, 20, Thing::Guard);
    b.thing(4, 21, Thing::Guard);
    b.thing(7, 17, Thing::TreasureCross);
    b.thing(3, 19, Thing::Table);

    b.thing(16, 26, Thing::Ss);
    b.thing(21, 24, Thing::TreasureChest);
    b.thing(15, 23, Thing::HealthLarge);
    b.thing(22, 27, Thing::AmmoBox);
    b.thing(19, 25, Thing::Barrel);

    b.thing(3, 11, Thing::TreasureCross);
    b.thing(7, 11, Thing::HealthSmall);
    b.thing(3, 10, Thing::AmmoBox);
    b.thing(7, 9, Thing::MachineGun);
    b.thing(6, 12, Thing::TreasureChalice);

    b.build_unchecked()
}

/// Floor 2: cell blocks, a hidden armoury with the chaingun, a shrine with the silver key, and a
pub fn floor_2() -> Level {
    let mut b = LevelBuilder::new("Floor 2", Tex::FloorWood, Tex::CeilDark);

    b.room(2, 2, 6, 5); // interior 2..7 x 2..6
    b.facing(1, 1, 8, 7, Wall::Cell);
    b.start(3.5, 3.5, 0.0);

    for cell_x in [12, 17, 22] {
        b.room(cell_x, 7, 4, 3); // interior cell_x..cell_x+3 x 7..9
        b.facing(cell_x - 1, 6, 6, 5, Wall::Cell);
    }
    b.secret(12, 7, 14, 3);

    b.room(31, 2, 10, 10); // interior 31..40 x 2..11
    b.facing(30, 1, 12, 12, Wall::Bluestone);
    b.hline(33, 38, 1, Wall::WoodEmblem);
    b.region(31, 2, 10, 10, Tex::FloorBlue, Tex::CeilDark);

    b.room(16, 13, 9, 6); // interior 16..24 x 13..18
    b.facing(15, 12, 11, 8, Wall::Cell);

    b.room(22, 7, 4, 4); // interior 22..25 x 7..10
    b.facing(21, 6, 6, 6, Wall::Cell);
    b.region(22, 7, 4, 4, Tex::FloorWood, Tex::CeilWood);

    b.room(32, 17, 9, 7); // interior 32..40 x 17..23
    b.facing(31, 16, 11, 9, Wall::Bluestone);
    b.region(32, 17, 9, 7, Tex::FloorBlue, Tex::CeilLight);

    // --- Openings
    b.door(8, 4, Wall::Door);
    corridor_h(&mut b, 9, 30, 4, Wall::Cell);
    for cell_x in [12, 17, 22] {
        b.door(cell_x + 1, 6, Wall::CellDoor);
        b.rect(cell_x + 1, 5, 1, 1); // a gap between the corridor and every cell door
    }
    b.door(30, 4, Wall::Door); // corridor -> shrine
    corridor_v(&mut b, 20, 5, 11, Wall::Cell); // corridor -> hidden armoury
    b.pushwall(20, 12, Wall::Cell);
    corridor_h(&mut b, 26, 30, 8, Wall::Cell); // dormitory -> shrine
    corridor_v(&mut b, 36, 13, 16, Wall::Bluestone); // shrine -> elevator room
    b.locked_door(36, 12, Wall::Door, Key::Silver);
    b.hline(35, 39, 24, Wall::ElevatorSwitch);

    // --- Contents
    b.thing(14, 4, Thing::AmbushGuard);
    b.thing(18, 4, Thing::Guard);
    b.thing(22, 4, Thing::Guard);
    b.thing(26, 4, Thing::AmbushGuard);
    b.thing(29, 4, Thing::AmmoClip);

    b.thing(13, 8, Thing::TreasureCross);
    b.thing(18, 8, Thing::Guard);
    b.thing(14, 9, Thing::Guard);
    b.thing(23, 8, Thing::HealthSmall);

    b.thing(35, 4, Thing::Ss);
    b.thing(38, 8, Thing::Ss);
    b.thing(33, 9, Thing::Ss);
    b.thing(37, 3, Thing::KeySilver);
    b.thing(32, 4, Thing::TreasureChalice);
    b.thing(39, 4, Thing::TreasureCross);
    b.thing(34, 7, Thing::Lamp);
    b.thing(37, 6, Thing::Table);

    b.thing(20, 15, Thing::Chaingun);
    b.thing(19, 16, Thing::Guard);
    b.thing(17, 14, Thing::AmmoBox);
    b.thing(23, 14, Thing::AmmoBox);
    b.thing(18, 17, Thing::AmmoClip);
    b.thing(22, 17, Thing::AmmoClip);
    b.secret(16, 13, 9, 6);

    b.thing(23, 8, Thing::Officer);
    b.thing(24, 9, Thing::Guard);
    b.thing(25, 8, Thing::TreasureCrown);
    b.thing(23, 10, Thing::Barrel);

    b.thing(34, 21, Thing::Boss);
    b.thing(38, 19, Thing::HealthLarge);
    b.thing(33, 18, Thing::TreasureChest);
    b.thing(39, 22, Thing::AmmoBox);
    b.thing(36, 20, Thing::Pillar);

    b.build_unchecked()
}

/// Floor 3: the great hall and its boss, the kitchen wing behind a pushwall, the cellar, and the
pub fn floor_3() -> Level {
    let mut b = LevelBuilder::new("Floor 3", Tex::FloorStone, Tex::CeilDark);

    b.room(2, 2, 8, 5); // interior 2..9 x 2..6
    b.facing(1, 1, 10, 7, Wall::Greystone);
    b.start(4.5, 4.5, 0.0);

    b.room(20, 7, 20, 14); // interior 20..39 x 7..20
    b.facing(19, 6, 22, 16, Wall::Greystone);
    b.hline(22, 37, 6, Wall::Banner);
    b.region(20, 7, 20, 14, Tex::FloorBlue, Tex::CeilLight);

    b.room(2, 8, 6, 8); // interior 2..7 x 8..15
    b.facing(1, 7, 8, 10, Wall::BrickWood);
    b.region(2, 8, 6, 8, Tex::FloorWood, Tex::CeilWood);

    b.room(41, 10, 9, 9); // interior 41..49 x 10..18
    b.facing(40, 9, 11, 11, Wall::Bluestone);
    b.region(41, 10, 9, 9, Tex::FloorBlue, Tex::CeilLight);

    b.room(11, 21, 8, 6); // interior 11..18 x 21..26
    b.facing(10, 20, 10, 8, Wall::Cell);
    b.region(11, 21, 8, 6, Tex::FloorWood, Tex::CeilWood);

    // --- Openings
    b.door(10, 4, Wall::Door);
    corridor_h(&mut b, 11, 34, 4, Wall::BrickWood);
    b.door(28, 6, Wall::Door);
    b.rect(28, 5, 1, 1);
    corridor_h(&mut b, 8, 18, 12, Wall::Metal); // hall -> kitchen wing
    b.pushwall(19, 12, Wall::Metal);
    b.secret(8, 12, 11, 1);
    b.secret(2, 8, 6, 8);
    b.locked_door(40, 14, Wall::Door, Key::Gold);
    b.vline(50, 13, 16, Wall::ElevatorSwitch);
    // north wall is the row below, and a corridor helper would brick it back up. The passage
    b.rect(11, 20, 9, 1);
    b.hline(11, 19, 19, Wall::Cell);
    b.door(16, 20, Wall::CellDoor);
    b.secret(11, 20, 9, 7);

    // --- Contents
    b.thing(13, 4, Thing::Guard);
    b.thing(17, 4, Thing::Ss);
    b.thing(24, 4, Thing::Guard);
    b.thing(31, 4, Thing::AmmoBox);

    b.thing(30, 13, Thing::Boss);
    b.thing(27, 16, Thing::Ss);
    b.thing(34, 16, Thing::Ss);
    b.thing(25, 12, Thing::Guard);
    b.thing(35, 12, Thing::Guard);
    b.thing(29, 15, Thing::TreasureCrown);
    b.thing(32, 15, Thing::HealthLarge);
    b.thing(22, 19, Thing::AmmoBox);
    b.thing(37, 8, Thing::TreasureChalice);
    b.thing(21, 9, Thing::Chandelier);
    b.thing(33, 9, Thing::Chandelier);
    b.thing(26, 14, Thing::Ss);
    b.thing(33, 13, Thing::Guard);
    b.thing(24, 10, Thing::Pillar);
    b.thing(30, 10, Thing::Pillar);
    b.thing(36, 10, Thing::Pillar);
    b.thing(24, 18, Thing::Pillar);
    b.thing(30, 18, Thing::Pillar);
    b.thing(36, 18, Thing::Pillar);
    b.thing(21, 8, Thing::KeyGold);

    b.thing(3, 10, Thing::Table);
    b.thing(6, 10, Thing::Table);
    b.thing(3, 14, Thing::Barrel);
    b.thing(6, 14, Thing::Barrel);
    b.thing(4, 12, Thing::Dog);
    b.thing(6, 13, Thing::Dog);
    b.thing(3, 9, Thing::HealthSmall);
    b.thing(5, 15, Thing::TreasureChalice);

    b.thing(12, 12, Thing::AmmoBox);
    b.thing(16, 12, Thing::AmmoBox);
    b.thing(10, 12, Thing::Chaingun);
    b.thing(18, 12, Thing::HealthLarge);

    b.thing(42, 11, Thing::Guard);
    b.thing(47, 11, Thing::Guard);
    b.thing(45, 13, Thing::Officer);
    b.thing(43, 17, Thing::Ss);
    b.thing(47, 17, Thing::Ss);
    b.thing(42, 14, Thing::HealthLarge);
    b.thing(46, 15, Thing::AmmoBox);
    b.thing(44, 12, Thing::TreasureChest);
    b.thing(48, 18, Thing::TreasureChalice);
    b.thing(43, 10, Thing::Lamp);

    b.thing(13, 23, Thing::Dog);
    b.thing(18, 23, Thing::Dog);
    b.thing(15, 25, Thing::TreasureCrown);
    b.thing(18, 25, Thing::HealthLarge);
    b.thing(13, 25, Thing::AmmoBox);

    b.build_unchecked()
}

/// The kits the game validates at startup: every floor must be finishable and fully reachable.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_floor_is_valid() {
        for level in floors() {
            level
                .validate()
                .unwrap_or_else(|err| panic!("floor failed validation: {err}"));
        }
    }

    #[test]
    fn every_floor_has_an_elevator_switch() {
        for level in floors() {
            let switches = (0..64)
                .flat_map(|y| (0..64).map(move |x| (x, y)))
                .filter(|(x, y)| level.wall(*x, *y).is_switch())
                .count();
            assert!(switches > 0, "{} has no switch", level.name);
        }
    }

    #[test]
    fn every_floor_places_the_player_inside_open_space() {
        for level in floors() {
            let (x, y, _) = level.start;
            assert_eq!(
                level.wall(x.floor() as i32, y.floor() as i32),
                Wall::None,
                "{} starts the player inside a wall",
                level.name
            );
        }
    }

    #[test]
    fn floors_get_bigger_and_meaner() {
        let floors = floors();
        let actors: Vec<i32> = floors.iter().map(|l| l.actor_count()).collect();
        assert!(
            actors.iter().sum::<i32>() > 20,
            "the three floors should hold a decent garrison, got {actors:?}"
        );
        assert!(
            actors.windows(2).all(|pair| pair[1] >= pair[0]),
            "later floors should not be easier: {actors:?}"
        );
    }

    #[test]
    fn every_floor_has_treasure_and_secrets() {
        for level in floors() {
            assert!(level.treasure_count() > 0, "{} has no treasure", level.name);
            assert!(!level.secrets.is_empty(), "{} has no secrets", level.name);
        }
    }
}
