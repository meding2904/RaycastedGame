#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub color: Option<[u8; 4]>,
    pub texture: Option<(usize, (usize, usize, usize, usize))>,
    pub frames: u16,

    /// True for door tiles. Rays pass through the columns a door has slid past and
    pub door: bool,

    /// Door animation state, 0.0 fully closed .. 1.0 fully open. Regular walls stay at 0.0.
    pub opening: f32,
}

/// A tile
impl Tile {
    /// Creates a new tile for a given image rectangle
    pub fn textured(image_id: usize, rect: (usize, usize, usize, usize)) -> Self {
        Self {
            color: None,
            texture: Some((image_id, rect)),
            frames: 1,
            door: false,
            opening: 0.0,
        }
    }

    /// Creates a new tile for a given image rectangle which has multiple frames horizontally. The rect is the first frame. Frames is the total number of frames including the first frame.
    pub fn textured_anim(image_id: usize, rect: (usize, usize, usize, usize), frames: u16) -> Self {
        Self {
            color: None,
            texture: Some((image_id, rect)),
            frames: frames,
            door: false,
            opening: 0.0,
        }
    }

    /// Creates a new tile for a given color
    pub fn colored(color: [u8; 4]) -> Self {
        Self {
            color: Some(color),
            texture: None,
            frames: 1,
            door: false,
            opening: 0.0,
        }
    }

    /// Creates a door tile, closed by default
    pub fn door(image_id: usize, rect: (usize, usize, usize, usize)) -> Self {
        Self {
            color: None,
            texture: Some((image_id, rect)),
            frames: 1,
            door: true,
            opening: 0.0,
        }
    }

    /// Sets the opening state of a door tile, 0.0 closed .. 1.0 fully open
    pub fn set_opening(&mut self, opening: f32) {
        self.opening = opening.clamp(0.0, 1.0);
    }
}
