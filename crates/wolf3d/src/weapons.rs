//! The four weapons and the knife, with their rates, damage and animation timing.

use crate::assets::sprites::{FaceMood, WeaponKind};

pub struct WeaponSpec {
    pub kind: WeaponKind,
    /// Damage before the difficulty scaling, the original's weapons are all in the same band
    pub damage: (i32, i32),
    /// Seconds between shots
    pub cooldown: f32,
    pub uses_ammo: bool,
    /// How far the shot reaches, in tiles
    pub range: f32,
    /// Knife only: how close the target has to be
    pub melee: bool,
}

pub fn spec(kind: WeaponKind) -> WeaponSpec {
    match kind {
        WeaponKind::Knife => WeaponSpec {
            kind,
            damage: (5, 10),
            cooldown: 0.45,
            uses_ammo: false,
            range: 1.4,
            melee: true,
        },
        WeaponKind::Pistol => WeaponSpec {
            kind,
            damage: (8, 16),
            cooldown: 0.42,
            uses_ammo: true,
            range: 30.0,
            melee: false,
        },
        WeaponKind::MachineGun => WeaponSpec {
            kind,
            damage: (8, 16),
            cooldown: 0.16,
            uses_ammo: true,
            range: 30.0,
            melee: false,
        },
        WeaponKind::Chaingun => WeaponSpec {
            kind,
            damage: (8, 16),
            cooldown: 0.09,
            uses_ammo: true,
            range: 30.0,
            melee: false,
        },
    }
}

/// What the player's hands are doing right now
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Firing {
    Idle,
    /// The weapon is mid animation, `frame` walks the sprite forward
    Firing,
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponState {
    pub kind: WeaponKind,
    pub ammo: i32,
    pub owned: [bool; 4],
    pub firing: Firing,
    pub frame: usize,
    pub anim_timer: f32,
    /// Seconds until the weapon can fire again
    pub cooldown: f32,
    /// Set for the duration of a click on an empty clip
    pub empty_click: f32,
}

impl WeaponState {
    pub fn new() -> Self {
        Self {
            kind: WeaponKind::Pistol,
            ammo: 8,
            owned: [true, true, false, false],
            firing: Firing::Idle,
            frame: 0,
            anim_timer: 0.0,
            cooldown: 0.0,
            empty_click: 0.0,
        }
    }

    pub fn spec(&self) -> WeaponSpec {
        spec(self.kind)
    }

    pub fn can_fire(&self) -> bool {
        if self.cooldown > 0.0 {
            return false;
        }
        !self.spec().uses_ammo || self.ammo > 0
    }

    /// Switches weapons if the player owns the one asked for. The knife and pistol cannot be
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.owned.len() || !self.owned[index] {
            return false;
        }
        let kind = WeaponKind::ALL[index];
        if kind == self.kind {
            return false;
        }
        self.kind = kind;
        self.firing = Firing::Idle;
        self.frame = 0;
        self.anim_timer = 0.0;
        self.cooldown = 0.25;
        true
    }

    pub fn grant(&mut self, kind: WeaponKind) {
        self.owned[kind.index()] = true;
    }

    pub fn add_ammo(&mut self, amount: i32) {
        self.ammo = (self.ammo + amount).max(0);
    }

    /// Advances the animation and tells us whether a frame ticked over
    pub fn advance(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.0);
        self.empty_click = (self.empty_click - dt).max(0.0);

        if self.firing == Firing::Firing {
            self.anim_timer += dt;
            let frame_time = self.spec().cooldown.min(0.12);
            if self.anim_timer >= frame_time {
                self.anim_timer = 0.0;
                self.frame += 1;
                if self.frame >= self.kind.frames() {
                    self.frame = 0;
                    self.firing = Firing::Idle;
                }
            }
        }
    }

    /// Starts the firing animation and returns true when a shot should actually be let loose this
    pub fn begin_fire(&mut self) -> bool {
        if !self.can_fire() {
            if self.spec().uses_ammo && self.ammo <= 0 {
                self.empty_click = 0.2;
            }
            return false;
        }

        self.firing = Firing::Firing;
        self.frame = 0;
        self.anim_timer = 0.0;
        self.cooldown = self.spec().cooldown;

        if self.spec().uses_ammo {
            self.ammo -= 1;
        }

        true
    }

    pub fn face_mood(&self) -> FaceMood {
        if self.firing == Firing::Firing {
            FaceMood::Firing
        } else {
            FaceMood::Normal
        }
    }
}

impl Default for WeaponState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_player_starts_with_a_knife_a_pistol_and_eight_bullets() {
        let state = WeaponState::new();
        assert_eq!(state.kind, WeaponKind::Pistol);
        assert_eq!(state.ammo, 8);
        assert!(state.owned[0] && state.owned[1]);
        assert!(!state.owned[2] && !state.owned[3]);
    }

    #[test]
    fn firing_uses_a_bullet_and_starts_a_cooldown() {
        let mut state = WeaponState::new();
        assert!(state.begin_fire());
        assert_eq!(state.ammo, 7);
        assert!(
            !state.can_fire(),
            "the cooldown should block an immediate second shot"
        );
        assert!(!state.begin_fire());
    }

    #[test]
    fn the_knife_never_runs_out_of_ammo() {
        let mut state = WeaponState::new();
        state.grant(WeaponKind::Knife);
        assert!(state.select(0));
        for _ in 0..20 {
            state.cooldown = 0.0;
            assert!(state.begin_fire());
        }
        assert_eq!(state.ammo, 8, "the knife must not consume bullets");
    }

    #[test]
    fn an_empty_weapon_clicks_instead_of_firing() {
        let mut state = WeaponState::new();
        state.ammo = 0;
        state.cooldown = 0.0;
        assert!(!state.begin_fire());
        assert!(state.empty_click > 0.0);
    }

    #[test]
    fn unowned_weapons_cannot_be_selected() {
        let mut state = WeaponState::new();
        assert!(
            !state.select(2),
            "the machine gun has not been picked up yet"
        );
        state.grant(WeaponKind::MachineGun);
        assert!(state.select(2));
        assert_eq!(state.kind, WeaponKind::MachineGun);
    }

    #[test]
    fn the_firing_animation_ends_by_itself() {
        let mut state = WeaponState::new();
        state.begin_fire();
        for _ in 0..40 {
            state.advance(0.05);
        }
        assert_eq!(state.firing, Firing::Idle);
        assert_eq!(state.frame, 0);
    }

    #[test]
    fn the_chaingun_is_the_fastest_weapon() {
        let rates: Vec<f32> = WeaponKind::ALL.iter().map(|k| spec(*k).cooldown).collect();
        assert!(
            rates[3] < rates[2],
            "chaingun should outpace the machine gun"
        );
        assert!(rates[2] < rates[1], "machine gun should outpace the pistol");
    }
}
