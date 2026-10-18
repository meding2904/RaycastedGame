//! Enemies: what each kind of soldier, dog and boss is made of, and the states they move through.

use crate::assets::sfx::Sfx;
use crate::assets::sprites::{
    ActorKind, ActorSprites, SHOOT_FRAMES, SLOT_PAIN, SLOT_SHOOT, SLOT_STAND,
};
use crate::difficulty::Difficulty;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActorState {
    /// Standing at the spawn point, unaware
    Idle,
    /// Walking towards the player, or towards the sound of a shot
    Chase,
    /// In melee reach, biting
    Attack,
    /// Standing still, firing
    Shoot,
    /// Staggered by a hit
    Pain,
    /// Playing the death animation
    Dying,
    /// Down for good
    Dead,
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub kind: ActorKind,
    pub x: f32,
    pub y: f32,
    /// Heading in radians, `atan2(y, x)` like the player's
    pub angle: f32,
    pub health: i32,
    pub state: ActorState,
    /// Set for actors that stand still until they spot the player, the original's ambush flag
    pub ambush: bool,
    /// Set once the actor knows where the player is
    pub alerted: bool,
    /// Time left in the current state
    pub state_timer: f32,
    pub anim_timer: f32,
    pub frame: usize,
    /// Guards and officers drop a clip when they go down
    pub drops_ammo: bool,
    /// Set once the sight sound has played, so it only plays once
    pub sighted: bool,
    /// Per actor random source, so behaviour is reproducible
    pub seed: u32,
}

impl Actor {
    pub fn new(
        kind: ActorKind,
        x: f32,
        y: f32,
        angle: f32,
        ambush: bool,
        seed: u32,
        difficulty: Difficulty,
    ) -> Self {
        let stats = stats(kind);
        let health = (stats.health as f32 * difficulty.enemy_health_scale()).round() as i32;

        Self {
            kind,
            x,
            y,
            angle,
            health,
            state: ActorState::Idle,
            ambush,
            alerted: false,
            state_timer: 0.0,
            anim_timer: 0.0,
            frame: 0,
            drops_ammo: stats.ranged && seed % 2 == 0,
            sighted: false,
            seed,
        }
    }

    pub fn stats(&self) -> ActorStats {
        stats(self.kind)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.state, ActorState::Dying | ActorState::Dead)
    }

    pub fn sight_range(&self) -> f32 {
        self.stats().sight_range
    }

    /// Enemies notice the player from further away once they are already chasing
    pub fn chase_sight_range(&self) -> f32 {
        self.sight_range() * 1.6
    }

    /// The sprite cell this actor should be drawn with
    pub fn view_slot(&self) -> usize {
        match self.state {
            ActorState::Idle => SLOT_STAND,
            ActorState::Chase => ActorSprites::walk_slot(self.frame),
            ActorState::Shoot | ActorState::Attack => SLOT_SHOOT + self.frame.min(SHOOT_FRAMES - 1),
            ActorState::Pain => SLOT_PAIN,
            ActorState::Dying => ActorSprites::die_slot(self.frame),
            ActorState::Dead => ActorSprites::die_slot(usize::MAX),
        }
    }

    /// Applies damage. Returns true when this blow killed the actor.
    pub fn hurt(&mut self, amount: i32, difficulty: Difficulty) -> bool {
        if !self.is_alive() {
            return false;
        }

        let scaled = (amount as f32 * difficulty.enemy_health_scale()).round() as i32;
        self.health -= scaled.max(1);
        self.alerted = true;

        if self.health <= 0 {
            self.health = 0;
            self.state = ActorState::Dying;
            self.state_timer = 0.0;
            self.frame = 0;
            self.anim_timer = 0.0;
            return true;
        }

        let stagger = match self.kind {
            ActorKind::Dog | ActorKind::Boss => 0.0,
            ActorKind::Ss => 0.25,
            _ => 0.5,
        };
        if (self.seed % 100) as f32 / 100.0 < stagger {
            self.state = ActorState::Pain;
            self.state_timer = 0.3;
        }

        false
    }

    /// Damage this actor deals with one blow, before the difficulty scaling
    pub fn damage(&self) -> i32 {
        self.stats().damage
    }
}

pub struct ActorStats {
    pub health: i32,
    /// Tiles per second
    pub speed: f32,
    /// How close the actor has to be to bite or fire
    pub attack_range: f32,
    /// How far the actor can see, in tiles
    pub sight_range: f32,
    pub damage: i32,
    /// Seconds between attacks
    pub attack_cooldown: f32,
    /// True for enemies that shoot, false for the dog
    pub ranged: bool,
}

pub fn stats(kind: ActorKind) -> ActorStats {
    match kind {
        ActorKind::Guard => ActorStats {
            health: 25,
            speed: 2.3,
            attack_range: 9.0,
            sight_range: 13.0,
            damage: 8,
            attack_cooldown: 1.3,
            ranged: true,
        },
        ActorKind::Ss => ActorStats {
            health: 100,
            speed: 2.6,
            attack_range: 11.0,
            sight_range: 15.0,
            damage: 14,
            attack_cooldown: 1.1,
            ranged: true,
        },
        ActorKind::Officer => ActorStats {
            health: 50,
            speed: 3.6,
            attack_range: 10.0,
            sight_range: 15.0,
            damage: 10,
            attack_cooldown: 1.0,
            ranged: true,
        },
        ActorKind::Dog => ActorStats {
            health: 20,
            speed: 4.4,
            attack_range: 1.3,
            sight_range: 11.0,
            damage: 6,
            attack_cooldown: 0.9,
            ranged: false,
        },
        ActorKind::Boss => ActorStats {
            health: 400,
            speed: 1.9,
            attack_range: 12.0,
            sight_range: 18.0,
            damage: 24,
            attack_cooldown: 0.55,
            ranged: true,
        },
    }
}

impl ActorKind {
    /// The sound this enemy makes when it spots the player
    pub fn sight_sound(self) -> Sfx {
        match self {
            ActorKind::Dog => Sfx::DogBark,
            ActorKind::Boss => Sfx::BossSight,
            _ => Sfx::GuardSight,
        }
    }

    pub fn death_sound(self) -> Sfx {
        match self {
            ActorKind::Dog => Sfx::DogAttack,
            _ => Sfx::GuardDeath,
        }
    }

    pub fn attack_sound(self) -> Sfx {
        match self {
            ActorKind::Dog => Sfx::DogAttack,
            _ => Sfx::MachineGunShot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_boss_is_the_toughest_and_the_dog_the_fastest() {
        let boss = stats(ActorKind::Boss);
        let dog = stats(ActorKind::Dog);
        for kind in ActorKind::ALL {
            if kind != ActorKind::Boss {
                assert!(stats(kind).health < boss.health);
            }
            if kind != ActorKind::Dog {
                assert!(stats(kind).speed < dog.speed);
            }
        }
    }

    #[test]
    fn harder_difficulties_make_tougher_enemies() {
        let normal = Actor::new(ActorKind::Ss, 0.0, 0.0, 0.0, false, 1, Difficulty::Normal);
        let hard = Actor::new(ActorKind::Ss, 0.0, 0.0, 0.0, false, 1, Difficulty::Hard);
        assert!(hard.health > normal.health);
    }

    #[test]
    fn a_lethal_hit_starts_the_death_animation() {
        let mut actor = Actor::new(
            ActorKind::Guard,
            0.0,
            0.0,
            0.0,
            false,
            0,
            Difficulty::Normal,
        );
        assert!(actor.hurt(1000, Difficulty::Normal));
        assert_eq!(actor.state, ActorState::Dying);
        assert!(!actor.is_alive());
        assert!(
            !actor.hurt(1000, Difficulty::Normal),
            "a corpse cannot be killed twice"
        );
    }

    #[test]
    fn a_corpse_uses_the_last_death_frame() {
        let mut actor = Actor::new(
            ActorKind::Guard,
            0.0,
            0.0,
            0.0,
            false,
            0,
            Difficulty::Normal,
        );
        actor.state = ActorState::Dead;
        assert_eq!(actor.view_slot(), ActorSprites::die_slot(usize::MAX));
    }

    #[test]
    fn only_shooting_enemies_can_drop_ammo() {
        let dog = Actor::new(ActorKind::Dog, 0.0, 0.0, 0.0, false, 2, Difficulty::Normal);
        assert!(!dog.drops_ammo);
    }
}
