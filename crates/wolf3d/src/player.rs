//! The player: position and heading, health, lives, keys and score.

use crate::assets::sprites::FaceMood;
use crate::difficulty::Difficulty;
use crate::weapons::WeaponState;

/// How far the player can be pushed into a wall before it stops them, in tiles. The original uses
pub const PLAYER_RADIUS: f32 = 0.30;

/// Walking and running speed, in tiles per second
pub const WALK_SPEED: f32 = 2.8;
pub const RUN_SPEED: f32 = 5.0;

/// How fast the keyboard turns the player, in radians per second
pub const TURN_SPEED: f32 = 2.4;

/// Mouse turning, in radians per pixel of mouse movement
pub const MOUSE_SENSITIVITY: f32 = 0.0035;

pub const MAX_HEALTH: i32 = 100;

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    /// Heading in radians, measured like `atan2(y, x)` on the map: 0 is east, positive turns south
    pub angle: f32,
    pub health: i32,
    pub lives: i32,
    pub score: i32,
    pub keys: [bool; 2],
    pub weapon: WeaponState,
    pub dead: bool,
    /// Counts up while the player is dying, so the view can drop to the floor
    pub death_timer: f32,
    /// Set for a moment after being hit, to play the pain face and shake the view
    pub pain_timer: f32,
    /// A step counter for the weapon bob
    pub bob: f32,
    /// Where the status bar face is looking, -2..2
    pub face_look: i32,
    pub face_look_timer: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle_degrees: f32) -> Self {
        Self {
            x,
            y,
            angle: angle_degrees.to_radians(),
            health: MAX_HEALTH,
            lives: 3,
            score: 0,
            keys: [false, false],
            weapon: WeaponState::new(),
            dead: false,
            death_timer: 0.0,
            pain_timer: 0.0,
            bob: 0.0,
            face_look: 0,
            face_look_timer: 0.0,
        }
    }

    /// Resets for a fresh floor, keeping the score, lives, weapons and keys
    pub fn enter_floor(&mut self, x: f32, y: f32, angle_degrees: f32) {
        self.x = x;
        self.y = y;
        self.angle = angle_degrees.to_radians();
        self.dead = false;
        self.death_timer = 0.0;
        self.pain_timer = 0.0;
        self.bob = 0.0;
    }

    /// Back to the start of a floor after dying: the original takes the weapons away and hands
    pub fn respawn(&mut self, x: f32, y: f32, angle_degrees: f32) {
        self.enter_floor(x, y, angle_degrees);
        self.health = MAX_HEALTH;
        self.weapon = WeaponState::new();
        self.keys = [false, false];
    }

    /// The unit vector the player is facing
    pub fn facing(&self) -> (f32, f32) {
        (self.angle.cos(), self.angle.sin())
    }

    pub fn facing_tile(&self, distance: f32) -> (i32, i32) {
        let (dx, dy) = self.facing();
        (
            (self.x + dx * distance).floor() as i32,
            (self.y + dy * distance).floor() as i32,
        )
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(MAX_HEALTH);
    }

    /// Applies damage. Returns true when this blow killed the player.
    pub fn hurt(&mut self, amount: i32, difficulty: Difficulty) -> bool {
        if self.dead || amount <= 0 {
            return false;
        }

        let scaled = (amount as f32 * difficulty.enemy_damage_scale()).round() as i32;
        self.health -= scaled.max(1);
        self.pain_timer = 0.35;

        if self.health <= 0 {
            self.health = 0;
            self.dead = true;
            self.death_timer = 0.0;
            return true;
        }

        false
    }

    pub fn has_key(&self, key: crate::level::Key) -> bool {
        match key {
            crate::level::Key::Gold => self.keys[0],
            crate::level::Key::Silver => self.keys[1],
        }
    }

    pub fn give_key(&mut self, key: crate::level::Key) {
        match key {
            crate::level::Key::Gold => self.keys[0] = true,
            crate::level::Key::Silver => self.keys[1] = true,
        }
    }

    /// The face on the status bar reacts to health and to being shot at
    pub fn face_mood(&self) -> FaceMood {
        if self.dead {
            return FaceMood::Pain;
        }
        if self.pain_timer > 0.0 {
            return FaceMood::Pain;
        }
        self.weapon.face_mood()
    }

    pub fn tick_timers(&mut self, dt: f32) {
        self.pain_timer = (self.pain_timer - dt).max(0.0);
        self.face_look_timer -= dt;
        if self.face_look_timer <= 0.0 {
            self.face_look_timer = 1.4 + (self.bob.fract() * 2.0);
            self.face_look = ((self.bob * 1.7).sin() * 2.4) as i32;
        }
        if self.dead {
            self.death_timer += dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_player_is_healthy_and_armed() {
        let player = Player::new(2.5, 3.5, 0.0);
        assert_eq!(player.health, 100);
        assert_eq!(player.lives, 3);
        assert_eq!(player.score, 0);
        assert_eq!(player.weapon.ammo, 8);
        assert!(!player.dead);
    }

    #[test]
    fn damage_scales_with_difficulty() {
        let mut baby = Player::new(0.0, 0.0, 0.0);
        let mut hard = Player::new(0.0, 0.0, 0.0);

        baby.hurt(20, Difficulty::Baby);
        hard.hurt(20, Difficulty::Hard);

        assert!(hard.health < baby.health, "hard should hurt more than easy");
        assert_eq!(baby.health, 95, "a quarter of 20 is 5");
        assert_eq!(hard.health, 70, "one and a half of 20 is 30");
    }

    #[test]
    fn a_lethal_blow_kills_and_reports_it() {
        let mut player = Player::new(0.0, 0.0, 0.0);
        player.health = 10;
        assert!(!player.hurt(5, Difficulty::Normal));
        assert!(player.hurt(50, Difficulty::Normal));
        assert!(player.dead);
        assert_eq!(player.health, 0);
        assert!(
            !player.hurt(50, Difficulty::Normal),
            "the dead cannot be killed again"
        );
    }

    #[test]
    fn healing_never_exceeds_the_maximum() {
        let mut player = Player::new(0.0, 0.0, 0.0);
        player.health = 90;
        player.heal(50);
        assert_eq!(player.health, MAX_HEALTH);
    }

    #[test]
    fn keys_are_remembered_separately() {
        use crate::level::Key;
        let mut player = Player::new(0.0, 0.0, 0.0);
        assert!(!player.has_key(Key::Gold));
        player.give_key(Key::Gold);
        assert!(player.has_key(Key::Gold));
        assert!(!player.has_key(Key::Silver));
    }

    #[test]
    fn facing_follows_the_heading() {
        let east = Player::new(0.0, 0.0, 0.0);
        assert!((east.facing().0 - 1.0).abs() < 0.001);

        let south = Player::new(0.0, 0.0, 90.0);
        assert!(
            south.facing().1 > 0.999,
            "90 degrees turns towards the bottom of the map"
        );
    }

    #[test]
    fn respawning_takes_the_weapons_away() {
        let mut player = Player::new(0.0, 0.0, 0.0);
        player
            .weapon
            .grant(crate::assets::sprites::WeaponKind::Chaingun);
        player.weapon.ammo = 99;
        player.score = 5000;
        player.lives = 2;

        player.respawn(4.5, 4.5, 0.0);

        assert_eq!(player.health, 100);
        assert_eq!(player.weapon.ammo, 8);
        assert!(!player.weapon.owned[3], "the chaingun is lost on death");
        assert_eq!(player.score, 5000, "but the score is kept");
        assert_eq!(player.lives, 2);
    }
}
