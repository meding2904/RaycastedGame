//! Input as the game sees it: a bag of flags that the window backend fills in and the game reads.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    /// Shift: the player runs
    pub run: bool,
    /// Control or the left mouse button, held down for automatic weapons
    pub shoot: bool,
    /// Space: open a door, push a wall, call the elevator
    pub use_key: bool,
    /// A weapon slot was pressed this tick, 0 based
    pub weapon: Option<usize>,
    pub escape: bool,
    /// Q while paused
    pub pause_quit: bool,
    pub enter: bool,
    pub space: bool,
    pub up: bool,
    pub down: bool,
    /// Mouse movement since the last tick, in pixels
    pub mouse_dx: f32,
}

impl Input {
    /// Menus accept either key, like the original's "press any key"
    pub fn confirm(&self) -> bool {
        self.enter || self.space || self.shoot
    }

    /// Clears the flags that should only be seen by a single tick. Held keys such as movement and
    pub fn clear_edges(&mut self) {
        self.use_key = false;
        self.weapon = None;
        self.escape = false;
        self.pause_quit = false;
        self.enter = false;
        self.up = false;
        self.down = false;
        self.mouse_dx = 0.0;
    }

    pub fn clear_all(&mut self) {
        *self = Input::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_keys_survive_an_edge_clear() {
        let mut input = Input {
            forward: true,
            shoot: true,
            use_key: true,
            weapon: Some(2),
            ..Input::default()
        };
        input.clear_edges();

        assert!(input.forward, "movement is held, not tapped");
        assert!(input.shoot, "fire is held for automatic weapons");
        assert!(!input.use_key, "opening a door happens once per press");
        assert!(input.weapon.is_none());
    }

    #[test]
    fn any_confirm_key_confirms() {
        assert!(Input {
            enter: true,
            ..Input::default()
        }
        .confirm());
        assert!(Input {
            space: true,
            ..Input::default()
        }
        .confirm());
        assert!(!Input::default().confirm());
    }
}
