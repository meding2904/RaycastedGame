#![forbid(unsafe_code)]
//! Vendored copy of <https://github.com/markusmoenig/raycaster> (MIT), commit 561d503, plus the

// This crate is vendored upstream code, kept as close to `markusmoenig/raycaster` as possible so a
// rebase stays cheap (see PATCHES.md). Upstream does not satisfy clippy with warnings denied, so
// its style lints are silenced here rather than fixed one by one; the game crate is linted strictly.
#![allow(clippy::all)]

pub mod light;
pub mod math;
pub mod raycaster;
pub mod sprite;
pub mod tile;
pub mod worldmap;

pub use crate::light::Light;
pub use crate::raycaster::Raycaster;
pub use crate::sprite::Sprite;
pub use crate::tile::Tile;
pub use crate::worldmap::WorldMap;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab,
}

pub mod prelude {
    pub use crate::light::Light;
    pub use crate::math::vec2;
    pub use crate::raycaster::Raycaster;
    pub use crate::sprite::Sprite;
    pub use crate::tile::Tile;
    pub use crate::worldmap::WorldMap;
    pub use crate::WidgetKey;
    pub use rustc_hash::FxHashMap;
}
