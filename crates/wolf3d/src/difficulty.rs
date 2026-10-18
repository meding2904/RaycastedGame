//! The four difficulty tiers of the original, and what they scale.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    /// "Can I play, Daddy?" — enemies hit for a quarter, pickups are generous
    Baby,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub const ALL: [Difficulty; 4] = [
        Difficulty::Baby,
        Difficulty::Easy,
        Difficulty::Normal,
        Difficulty::Hard,
    ];

    pub fn index(self) -> usize {
        Difficulty::ALL.iter().position(|d| *d == self).unwrap_or(0)
    }

    /// The names printed on the difficulty screen
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Baby => "CAN I PLAY, DADDY?",
            Difficulty::Easy => "DON'T HURT ME.",
            Difficulty::Normal => "BRING 'EM ON!",
            Difficulty::Hard => "I AM DEATH INCARNATE!",
        }
    }

    /// How hard the enemies hit, the original's headline difference between the tiers
    pub fn enemy_damage_scale(self) -> f32 {
        match self {
            Difficulty::Baby => 0.25,
            Difficulty::Easy => 0.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.5,
        }
    }

    pub fn enemy_health_scale(self) -> f32 {
        match self {
            Difficulty::Baby => 1.0,
            Difficulty::Easy => 1.0,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.25,
        }
    }

    /// Ammo pickups give less on the hard tiers
    pub fn ammo_scale(self) -> f32 {
        match self {
            Difficulty::Baby => 2.0,
            Difficulty::Easy => 1.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.75,
        }
    }

    /// Health pickups are more generous on the easy tiers
    pub fn health_scale(self) -> f32 {
        match self {
            Difficulty::Baby => 1.5,
            Difficulty::Easy => 1.25,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harder_tiers_hurt_more() {
        let scales: Vec<f32> = Difficulty::ALL
            .iter()
            .map(|d| d.enemy_damage_scale())
            .collect();
        assert!(
            scales.windows(2).all(|pair| pair[1] > pair[0]),
            "{scales:?}"
        );
    }

    #[test]
    fn easier_tiers_hand_out_more_ammo() {
        let scales: Vec<f32> = Difficulty::ALL.iter().map(|d| d.ammo_scale()).collect();
        assert!(
            scales.windows(2).all(|pair| pair[1] < pair[0]),
            "{scales:?}"
        );
    }

    #[test]
    fn every_tier_has_a_name() {
        for difficulty in Difficulty::ALL {
            assert!(!difficulty.name().is_empty());
        }
    }
}
