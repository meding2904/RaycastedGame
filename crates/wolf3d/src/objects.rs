//! The things lying around a floor: pickups, decorations and what happens when they are touched.

use crate::assets::sprites::{ObjectKind, WeaponKind};
use crate::difficulty::Difficulty;
use crate::level::{Key, Thing};

/// The height of the 3D view in pixels. Sprites are billboarded into it, so the anchoring maths
pub const VIEW_HEIGHT: f32 = 160.0;

#[derive(Clone, Debug)]
pub struct MapObject {
    pub x: i32,
    pub y: i32,
    pub thing: Thing,
    /// Pickups are removed from the floor once taken
    pub taken: bool,
}

impl MapObject {
    pub fn new(x: i32, y: i32, thing: Thing) -> Self {
        Self {
            x,
            y,
            thing,
            taken: false,
        }
    }

    pub fn sprite(&self) -> Option<ObjectKind> {
        sprite_for(self.thing)
    }

    pub fn is_solid(&self) -> bool {
        self.thing == Thing::Pillar || self.thing == Thing::Barrel || self.thing == Thing::Table
    }
}

/// The atlas cell for a thing, if it has one. Actors and pure markers do not.
pub fn sprite_for(thing: Thing) -> Option<ObjectKind> {
    Some(match thing {
        Thing::HealthSmall => ObjectKind::HealthSmall,
        Thing::HealthLarge => ObjectKind::HealthLarge,
        Thing::AmmoBox => ObjectKind::AmmoBox,
        Thing::AmmoClip => ObjectKind::AmmoClip,
        Thing::TreasureCross => ObjectKind::TreasureCross,
        Thing::TreasureChalice => ObjectKind::TreasureChalice,
        Thing::TreasureChest => ObjectKind::TreasureChest,
        Thing::TreasureCrown => ObjectKind::TreasureCrown,
        Thing::KeyGold => ObjectKind::KeyGold,
        Thing::KeySilver => ObjectKind::KeySilver,
        Thing::ExtraLife => ObjectKind::ExtraLife,
        Thing::Barrel => ObjectKind::Barrel,
        Thing::Table => ObjectKind::Table,
        Thing::Pillar => ObjectKind::Pillar,
        Thing::Lamp => ObjectKind::Lamp,
        Thing::Chandelier => ObjectKind::Chandelier,
        Thing::Plant => ObjectKind::Plant,
        Thing::Blood => ObjectKind::Puddle,
        Thing::MachineGun | Thing::Chaingun => ObjectKind::AmmoBox,
        Thing::Guard
        | Thing::AmbushGuard
        | Thing::Ss
        | Thing::Officer
        | Thing::Dog
        | Thing::Boss => return None,
    })
}

/// Small pickups are drawn half a tile high, furniture fills a tile
pub fn sprite_shrink(thing: Thing) -> i32 {
    match thing {
        Thing::Barrel
        | Thing::Table
        | Thing::Pillar
        | Thing::Plant
        | Thing::TreasureChest
        | Thing::TreasureCrown => 1,
        Thing::Lamp => 1,
        _ => 2,
    }
}

/// The vertical offset that puts the bottom of a sprite on the floor line, in view pixels per
pub fn sprite_move_y(thing: Thing) -> f32 {
    let shrink = sprite_shrink(thing) as f32;
    let on_floor = VIEW_HEIGHT / 2.0 - VIEW_HEIGHT / 2.0 / shrink;

    match thing {
        Thing::Chandelier => -on_floor,
        _ => on_floor,
    }
}

/// What walking over a thing does to the player
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pickup {
    Health(i32),
    Ammo(i32),
    Weapon(WeaponKind, i32),
    Treasure(i32),
    Key(Key),
    ExtraLife,
}

impl Pickup {
    /// The score awarded immediately, for things that are worth points as well as their effect
    pub fn score(self) -> i32 {
        match self {
            Pickup::Treasure(points) => points,
            Pickup::ExtraLife => 400,
            _ => 0,
        }
    }
}

/// The effect of picking something up, scaled by the difficulty. Base amounts follow the
pub fn pickup_for(thing: Thing, difficulty: Difficulty) -> Option<Pickup> {
    let ammo = |base: i32| Pickup::Ammo((base as f32 * difficulty.ammo_scale()).round() as i32);
    let health =
        |base: i32| Pickup::Health((base as f32 * difficulty.health_scale()).round() as i32);

    Some(match thing {
        Thing::HealthSmall => health(25),
        Thing::HealthLarge => health(50),
        Thing::AmmoClip => ammo(8),
        Thing::AmmoBox => ammo(25),
        Thing::MachineGun => Pickup::Weapon(WeaponKind::MachineGun, ammo(6).amount()),
        Thing::Chaingun => Pickup::Weapon(WeaponKind::Chaingun, ammo(6).amount()),
        Thing::TreasureCross => Pickup::Treasure(Thing::TreasureCross.score()),
        Thing::TreasureChalice => Pickup::Treasure(Thing::TreasureChalice.score()),
        Thing::TreasureChest => Pickup::Treasure(Thing::TreasureChest.score()),
        Thing::TreasureCrown => Pickup::Treasure(Thing::TreasureCrown.score()),
        Thing::KeyGold => Pickup::Key(Key::Gold),
        Thing::KeySilver => Pickup::Key(Key::Silver),
        Thing::ExtraLife => Pickup::ExtraLife,
        _ => return None,
    })
}

impl Pickup {
    fn amount(self) -> i32 {
        match self {
            Pickup::Ammo(amount) => amount,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pickup_has_a_sprite() {
        for thing in [
            Thing::HealthSmall,
            Thing::HealthLarge,
            Thing::AmmoBox,
            Thing::AmmoClip,
            Thing::TreasureCrown,
            Thing::KeyGold,
            Thing::KeySilver,
            Thing::ExtraLife,
            Thing::Barrel,
            Thing::Pillar,
        ] {
            assert!(sprite_for(thing).is_some(), "{thing:?} has no sprite");
        }
    }

    #[test]
    fn enemies_are_not_objects() {
        assert!(sprite_for(Thing::Guard).is_none());
        assert!(sprite_for(Thing::Boss).is_none());
    }

    #[test]
    fn decorations_are_not_pickups() {
        for thing in [
            Thing::Barrel,
            Thing::Table,
            Thing::Pillar,
            Thing::Lamp,
            Thing::Plant,
            Thing::Blood,
        ] {
            assert!(
                pickup_for(thing, Difficulty::Normal).is_none(),
                "{thing:?} should not be picked up"
            );
        }
    }

    #[test]
    fn easy_difficulties_give_more_ammo() {
        let baby = pickup_for(Thing::AmmoClip, Difficulty::Baby);
        let hard = pickup_for(Thing::AmmoClip, Difficulty::Hard);
        assert_eq!(
            baby,
            Some(Pickup::Ammo(16)),
            "a clip is eight rounds, doubled on the easiest tier"
        );
        assert_eq!(hard, Some(Pickup::Ammo(6)));
    }

    #[test]
    fn treasure_is_worth_what_the_original_says() {
        assert_eq!(
            pickup_for(Thing::TreasureCrown, Difficulty::Normal),
            Some(Pickup::Treasure(5000))
        );
        assert_eq!(
            pickup_for(Thing::TreasureCross, Difficulty::Normal),
            Some(Pickup::Treasure(100))
        );
    }

    #[test]
    fn everything_that_stands_on_the_floor_has_its_bottom_there() {
        assert_eq!(sprite_move_y(Thing::Pillar), 0.0);
        assert_eq!(sprite_move_y(Thing::AmmoClip), VIEW_HEIGHT / 4.0);
        assert!(sprite_move_y(Thing::Chandelier) < 0.0);
    }

    #[test]
    fn only_furniture_blocks_the_player() {
        assert!(MapObject::new(0, 0, Thing::Pillar).is_solid());
        assert!(!MapObject::new(0, 0, Thing::AmmoClip).is_solid());
        assert!(!MapObject::new(0, 0, Thing::Blood).is_solid());
    }
}
