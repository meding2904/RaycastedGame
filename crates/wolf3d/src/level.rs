//! Level data: the 64x64 tile grid, the things on it, doors, pushwalls and secret areas.

use crate::assets::textures::Tex;

pub const MAP_WIDTH: i32 = 64;
pub const MAP_HEIGHT: i32 = 64;
pub const MAP_AREA: usize = (MAP_WIDTH * MAP_HEIGHT) as usize;

/// Wall codes. Zero means no wall at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wall {
    None,
    Cell,
    Bluestone,
    Greystone,
    Brick,
    BrickWood,
    Wood,
    WoodEmblem,
    Banner,
    Metal,
    Door,
    CellDoor,
    ElevatorSwitch,
}

impl Wall {
    pub const ALL: [Wall; 13] = [
        Wall::None,
        Wall::Cell,
        Wall::Bluestone,
        Wall::Greystone,
        Wall::Brick,
        Wall::BrickWood,
        Wall::Wood,
        Wall::WoodEmblem,
        Wall::Banner,
        Wall::Metal,
        Wall::Door,
        Wall::CellDoor,
        Wall::ElevatorSwitch,
    ];

    pub fn code(self) -> u8 {
        Wall::ALL.iter().position(|w| *w == self).unwrap_or(0) as u8
    }

    pub fn from_code(code: u8) -> Wall {
        Wall::ALL.get(code as usize).copied().unwrap_or(Wall::None)
    }

    pub fn is_solid(self) -> bool {
        self != Wall::None
    }

    /// Doors block movement until they have opened, and they animate in the renderer
    pub fn is_door(self) -> bool {
        matches!(self, Wall::Door | Wall::CellDoor)
    }

    /// The elevator switch ends the floor when used
    pub fn is_switch(self) -> bool {
        self == Wall::ElevatorSwitch
    }

    pub fn tex(self) -> Option<Tex> {
        match self {
            Wall::None => None,
            Wall::Cell => Some(Tex::Cell),
            Wall::Bluestone => Some(Tex::Bluestone),
            Wall::Greystone => Some(Tex::Greystone),
            Wall::Brick => Some(Tex::Brick),
            Wall::BrickWood => Some(Tex::BrickWood),
            Wall::Wood => Some(Tex::Wood),
            Wall::WoodEmblem => Some(Tex::WoodEmblem),
            Wall::Banner => Some(Tex::Banner),
            Wall::Metal => Some(Tex::Metal),
            Wall::Door => Some(Tex::Door),
            Wall::CellDoor => Some(Tex::CellDoor),
            Wall::ElevatorSwitch => Some(Tex::ElevatorSwitch),
        }
    }
}

/// Everything that is not a wall: enemy spawn markers, pickups and decorations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Thing {
    Guard,
    AmbushGuard,
    Ss,
    Officer,
    Dog,
    Boss,
    HealthSmall,
    HealthLarge,
    AmmoBox,
    AmmoClip,
    MachineGun,
    Chaingun,
    TreasureCross,
    TreasureChalice,
    TreasureChest,
    TreasureCrown,
    KeyGold,
    KeySilver,
    ExtraLife,
    Barrel,
    Table,
    Pillar,
    Lamp,
    Chandelier,
    Plant,
    Blood,
}

impl Thing {
    pub const ALL: [Thing; 26] = [
        Thing::Guard,
        Thing::AmbushGuard,
        Thing::Ss,
        Thing::Officer,
        Thing::Dog,
        Thing::Boss,
        Thing::HealthSmall,
        Thing::HealthLarge,
        Thing::AmmoBox,
        Thing::AmmoClip,
        Thing::MachineGun,
        Thing::Chaingun,
        Thing::TreasureCross,
        Thing::TreasureChalice,
        Thing::TreasureChest,
        Thing::TreasureCrown,
        Thing::KeyGold,
        Thing::KeySilver,
        Thing::ExtraLife,
        Thing::Barrel,
        Thing::Table,
        Thing::Pillar,
        Thing::Lamp,
        Thing::Chandelier,
        Thing::Plant,
        Thing::Blood,
    ];

    pub fn code(self) -> u8 {
        (Thing::ALL.iter().position(|t| *t == self).unwrap_or(0) + 1) as u8
    }

    pub fn from_code(code: u8) -> Option<Thing> {
        if code == 0 {
            return None;
        }
        Thing::ALL.get(code as usize - 1).copied()
    }

    pub fn is_actor(self) -> bool {
        matches!(
            self,
            Thing::Guard
                | Thing::AmbushGuard
                | Thing::Ss
                | Thing::Officer
                | Thing::Dog
                | Thing::Boss
        )
    }

    pub fn is_treasure(self) -> bool {
        matches!(
            self,
            Thing::TreasureCross
                | Thing::TreasureChalice
                | Thing::TreasureChest
                | Thing::TreasureCrown
        )
    }

    /// Score value of a pickup, straight out of the original's scoring table
    pub fn score(self) -> i32 {
        match self {
            Thing::TreasureCross => 100,
            Thing::TreasureChalice => 500,
            Thing::TreasureChest => 1000,
            Thing::TreasureCrown => 5000,
            Thing::ExtraLife => 400,
            _ => 0,
        }
    }
}

/// The two keys of the original: gold and silver. A locked door needs the matching one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Gold,
    Silver,
}

fn thing_char(thing: Thing) -> char {
    match thing {
        Thing::Guard => 'g',
        Thing::AmbushGuard => 'G',
        Thing::Ss => 's',
        Thing::Officer => 'o',
        Thing::Dog => 'k',
        Thing::Boss => 'Q',
        Thing::HealthSmall => '+',
        Thing::HealthLarge => 'H',
        Thing::AmmoBox => 'a',
        Thing::AmmoClip => 'A',
        Thing::MachineGun => 'm',
        Thing::Chaingun => 'u',
        Thing::TreasureCross => 'X',
        Thing::TreasureChalice => 'V',
        Thing::TreasureChest => 'T',
        Thing::TreasureCrown => 'R',
        Thing::KeyGold => 'K',
        Thing::KeySilver => 'L',
        Thing::ExtraLife => '1',
        Thing::Barrel => 'r',
        Thing::Table => 't',
        Thing::Pillar => 'p',
        Thing::Lamp => 'l',
        Thing::Chandelier => 'c',
        Thing::Plant => 'P',
        Thing::Blood => 'b',
    }
}

/// A rectangle of tiles, used for secret areas
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

/// A finished floor
#[derive(Clone, Debug)]
pub struct Level {
    pub name: String,
    pub walls: Vec<u8>,
    pub things: Vec<u8>,
    pub floor: Tex,
    pub ceiling: Tex,
    /// Per area floor and ceiling overrides
    pub regions: Vec<(Rect, Tex, Tex)>,
    pub secrets: Vec<Rect>,
    pub pushwalls: Vec<(i32, i32)>,
    /// Doors that need a key, and which key
    pub locked: Vec<(i32, i32, Key)>,
    pub start: (f32, f32, f32),
}

impl Level {
    pub fn wall(&self, x: i32, y: i32) -> Wall {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            // Outside the map is solid rock, which keeps rays and actors inside
            return Wall::Greystone;
        }
        Wall::from_code(self.walls[(y * MAP_WIDTH + x) as usize])
    }

    pub fn set_wall(&mut self, x: i32, y: i32, wall: Wall) {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            return;
        }
        self.walls[(y * MAP_WIDTH + x) as usize] = wall.code();
    }

    pub fn thing(&self, x: i32, y: i32) -> Option<Thing> {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            return None;
        }
        Thing::from_code(self.things[(y * MAP_WIDTH + x) as usize])
    }

    pub fn set_thing(&mut self, x: i32, y: i32, thing: Option<Thing>) {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            return;
        }
        let code = thing.map(|t| t.code()).unwrap_or(0);
        self.things[(y * MAP_WIDTH + x) as usize] = code;
    }

    pub fn is_pushwall(&self, x: i32, y: i32) -> bool {
        self.pushwalls.iter().any(|(px, py)| *px == x && *py == y)
    }

    /// The key a door needs, if it is locked
    pub fn lock(&self, x: i32, y: i32) -> Option<Key> {
        self.locked
            .iter()
            .find(|(lx, ly, _)| *lx == x && *ly == y)
            .map(|(_, _, key)| *key)
    }

    /// The floor and ceiling texture of a tile, honouring the region overrides
    pub fn tiles_at(&self, x: i32, y: i32) -> (Tex, Tex) {
        for (rect, floor, ceiling) in &self.regions {
            if rect.contains(x, y) {
                return (*floor, *ceiling);
            }
        }
        (self.floor, self.ceiling)
    }

    /// Renders the floor as text: walls as characters, things as letters, `@` for the spawn
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        let (sx, sy, _) = self.start;

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let ch = if x == sx.floor() as i32 && y == sy.floor() as i32 {
                    '@'
                } else if let Some(thing) = self.thing(x, y) {
                    thing_char(thing)
                } else if self.is_pushwall(x, y) {
                    'P'
                } else {
                    match self.wall(x, y) {
                        Wall::None => '.',
                        Wall::Cell => 'C',
                        Wall::Bluestone => '%',
                        Wall::Greystone => '#',
                        Wall::Brick => 'B',
                        Wall::BrickWood => 'x',
                        Wall::Wood => 'w',
                        Wall::WoodEmblem => 'W',
                        Wall::Banner => 'n',
                        Wall::Metal => 'M',
                        Wall::Door => 'D',
                        Wall::CellDoor => 'd',
                        Wall::ElevatorSwitch => 'E',
                    }
                };
                out.push(ch);
            }
            out.push('\n');
        }

        out
    }

    /// Total number of killable actors, used by the end of level tally
    pub fn actor_count(&self) -> i32 {
        (0..MAP_AREA)
            .filter(|i| {
                Thing::from_code(self.things[*i])
                    .map(|t| t.is_actor())
                    .unwrap_or(false)
            })
            .count() as i32
    }

    /// Total treasure value on a floor, used by the tally
    pub fn treasure_count(&self) -> i32 {
        (0..MAP_AREA)
            .filter(|i| {
                Thing::from_code(self.things[*i])
                    .map(|t| t.is_treasure())
                    .unwrap_or(false)
            })
            .count() as i32
    }
}

/// Carves rooms and corridors into a solid block of rock.
pub struct LevelBuilder {
    name: String,
    walls: Vec<u8>,
    things: Vec<u8>,
    floor: Tex,
    ceiling: Tex,
    regions: Vec<(Rect, Tex, Tex)>,
    secrets: Vec<Rect>,
    pushwalls: Vec<(i32, i32)>,
    locked: Vec<(i32, i32, Key)>,
    start: Option<(f32, f32, f32)>,
}

impl LevelBuilder {
    /// Starts with a full block of rock and an empty thing layer
    pub fn new(name: &str, floor: Tex, ceiling: Tex) -> Self {
        Self {
            name: name.to_string(),
            walls: vec![Wall::Greystone.code(); MAP_AREA],
            things: vec![0; MAP_AREA],
            floor,
            ceiling,
            regions: Vec::new(),
            secrets: Vec::new(),
            pushwalls: Vec::new(),
            locked: Vec::new(),
            start: None,
        }
    }

    /// A rectangular room: hollow inside, walled around the outside
    pub fn room(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.rect(x, y, w, h);
        self.outline(x - 1, y - 1, w + 2, h + 2);
    }

    /// Fills a rectangle with open floor
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, Wall::None);
            }
        }
    }

    /// Draws a one tile thick wall around a rectangle
    pub fn outline(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.hline(x, x + w - 1, y, Wall::Greystone);
        self.hline(x, x + w - 1, y + h - 1, Wall::Greystone);
        self.vline(x, y, y + h - 1, Wall::Greystone);
        self.vline(x + w - 1, y, y + h - 1, Wall::Greystone);
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, wall: Wall) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, wall);
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, wall: Wall) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.set(x, y, wall);
        }
    }

    /// Replaces the wall material of an already solid rectangle, for dressing up a room
    pub fn facing(&mut self, x: i32, y: i32, w: i32, h: i32, wall: Wall) {
        for yy in y..y + h {
            for xx in x..x + w {
                if self.get(xx, yy) != Wall::None {
                    self.set(xx, yy, wall);
                }
            }
        }
    }

    /// A door tile. The wall it sits in should already be there or be drawn around it.
    pub fn door(&mut self, x: i32, y: i32, wall: Wall) {
        self.set(x, y, wall);
    }

    /// Places a pushwall: a solid tile that slides two tiles when used, revealing what is behind
    pub fn pushwall(&mut self, x: i32, y: i32, wall: Wall) {
        self.set(x, y, wall);
        self.pushwalls.push((x, y));
    }

    /// A door that stays shut until the player has the matching key
    pub fn locked_door(&mut self, x: i32, y: i32, wall: Wall, key: Key) {
        self.set(x, y, wall);
        self.locked.push((x, y, key));
    }

    pub fn thing(&mut self, x: i32, y: i32, thing: Thing) {
        self.set_thing(x, y, Some(thing));
    }

    /// Places several things in a row, for corridors of guards
    pub fn things(&mut self, placements: &[(i32, i32, Thing)]) {
        for (x, y, thing) in placements {
            self.thing(*x, *y, *thing);
        }
    }

    pub fn secret(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.secrets.push(Rect { x, y, w, h });
    }

    pub fn region(&mut self, x: i32, y: i32, w: i32, h: i32, floor: Tex, ceiling: Tex) {
        self.regions.push((Rect { x, y, w, h }, floor, ceiling));
    }

    /// The player's spawn position and heading in degrees, 0 pointing east
    pub fn start(&mut self, x: f32, y: f32, angle_degrees: f32) {
        self.start = Some((x, y, angle_degrees));
    }

    fn index(x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            None
        } else {
            Some((y * MAP_WIDTH + x) as usize)
        }
    }

    fn set(&mut self, x: i32, y: i32, wall: Wall) {
        if let Some(i) = Self::index(x, y) {
            self.walls[i] = wall.code();
        }
    }

    fn get(&self, x: i32, y: i32) -> Wall {
        Self::index(x, y)
            .map(|i| Wall::from_code(self.walls[i]))
            .unwrap_or(Wall::None)
    }

    fn set_thing(&mut self, x: i32, y: i32, thing: Option<Thing>) {
        if let Some(i) = Self::index(x, y) {
            self.things[i] = thing.map(|t| t.code()).unwrap_or(0);
        }
    }

    /// Finishes the floor without checking it. Used by the map dump, which has to be able to show
    pub fn build_unchecked(self) -> Level {
        let start = self.start.unwrap_or_else(|| {
            eprintln!("warning: {} has no player start", self.name);
            (2.5, 2.5, 0.0)
        });

        Level {
            name: self.name,
            walls: self.walls,
            things: self.things,
            floor: self.floor,
            ceiling: self.ceiling,
            regions: self.regions,
            secrets: self.secrets,
            pushwalls: self.pushwalls,
            locked: self.locked,
            start,
        }
    }

    /// Finishes the floor and checks that it is playable
    pub fn build(self) -> Result<Level, String> {
        let level = self.build_unchecked();
        level.validate()?;
        Ok(level)
    }
}

impl Level {
    /// Checks that a floor can actually be finished and that its contents can be reached:
    pub fn validate(&self) -> Result<(), String> {
        let (sx, sy, _) = self.start;
        let start = (sx.floor() as i32, sy.floor() as i32);

        let open = |x: i32, y: i32| -> bool {
            let wall = self.wall(x, y);
            wall == Wall::None || wall.is_door() || wall.is_switch() || self.is_pushwall(x, y)
        };

        let mut seen = vec![false; MAP_AREA];
        let mut queue = vec![start];
        seen[(start.1 * MAP_WIDTH + start.0) as usize] = true;

        while let Some((x, y)) = queue.pop() {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= MAP_WIDTH || ny >= MAP_HEIGHT {
                    continue;
                }
                let index = (ny * MAP_WIDTH + nx) as usize;
                if seen[index] || !open(nx, ny) {
                    continue;
                }
                seen[index] = true;
                queue.push((nx, ny));
            }
        }

        let reachable = |x: i32, y: i32| -> bool {
            x >= 0
                && y >= 0
                && x < MAP_WIDTH
                && y < MAP_HEIGHT
                && seen[(y * MAP_WIDTH + x) as usize]
        };

        if !open(start.0, start.1) {
            return Err(format!("{}: the player starts inside a wall", self.name));
        }

        // The elevator switch must be reachable: check it is adjacent to reachable floor
        let mut switch_found = false;
        let mut switch_reachable = false;
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                if !self.wall(x, y).is_switch() {
                    continue;
                }
                switch_found = true;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if reachable(x + dx, y + dy) {
                        switch_reachable = true;
                    }
                }
            }
        }

        if !switch_found {
            return Err(format!("{}: no elevator switch", self.name));
        }
        if !switch_reachable {
            return Err(format!(
                "{}: the elevator switch cannot be reached",
                self.name
            ));
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let Some(thing) = self.thing(x, y) else {
                    continue;
                };
                if !reachable(x, y) {
                    return Err(format!(
                        "{}: {:?} at ({x},{y}) is walled off",
                        self.name, thing
                    ));
                }
            }
        }

        // A pushwall needs somewhere to go: two free tiles in a straight line, or it would just
        for (x, y) in &self.pushwalls {
            let free = |dx: i32, dy: i32, steps: i32| -> bool {
                (1..=steps).all(|step| self.wall(x + dx * step, y + dy * step) == Wall::None)
            };
            let any = free(1, 0, 2) || free(-1, 0, 2) || free(0, 1, 2) || free(0, -1, 2);
            if !any {
                return Err(format!(
                    "{}: pushwall at ({x},{y}) has nowhere to slide",
                    self.name
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_codes_round_trip() {
        for wall in Wall::ALL {
            assert_eq!(Wall::from_code(wall.code()), wall);
        }
    }

    #[test]
    fn thing_codes_round_trip() {
        for thing in Thing::ALL {
            assert_eq!(Thing::from_code(thing.code()), Some(thing));
        }
        assert_eq!(Thing::from_code(0), None);
    }

    #[test]
    fn treasure_scores_match_the_original() {
        assert_eq!(Thing::TreasureCross.score(), 100);
        assert_eq!(Thing::TreasureChalice.score(), 500);
        assert_eq!(Thing::TreasureChest.score(), 1000);
        assert_eq!(Thing::TreasureCrown.score(), 5000);
    }

    #[test]
    fn a_level_without_a_switch_is_rejected() {
        let mut b = LevelBuilder::new("Test", Tex::FloorStone, Tex::CeilDark);
        b.room(2, 2, 6, 6);
        b.start(4.5, 4.5, 0.0);
        assert!(b.build().is_err());
    }

    #[test]
    fn a_walled_off_switch_is_rejected() {
        let mut b = LevelBuilder::new("Test", Tex::FloorStone, Tex::CeilDark);
        b.room(2, 2, 6, 6);
        b.start(4.5, 4.5, 0.0);
        b.hline(10, 14, 10, Wall::ElevatorSwitch);
        assert!(b.build().is_err());
    }

    #[test]
    fn a_minimal_floor_is_accepted() {
        let mut b = LevelBuilder::new("Test", Tex::FloorStone, Tex::CeilDark);
        b.room(2, 2, 6, 6);
        b.start(4.5, 4.5, 0.0);
        b.hline(6, 8, 2, Wall::ElevatorSwitch);
        b.thing(3, 3, Thing::Guard);
        assert!(b.build().is_ok());
    }

    #[test]
    fn unreachable_content_is_rejected() {
        let mut b = LevelBuilder::new("Test", Tex::FloorStone, Tex::CeilDark);
        b.room(2, 2, 6, 6);
        b.start(4.5, 4.5, 0.0);
        b.hline(6, 8, 2, Wall::ElevatorSwitch);
        b.room(2, 20, 4, 4);
        b.thing(3, 21, Thing::Guard);
        assert!(b.build().is_err());
    }

    #[test]
    fn a_pushwall_needs_room_to_slide() {
        let mut b = LevelBuilder::new("Test", Tex::FloorStone, Tex::CeilDark);
        b.room(2, 2, 6, 6);
        b.start(4.5, 4.5, 0.0);
        b.hline(6, 8, 2, Wall::ElevatorSwitch);
        b.rect(12, 12, 3, 3);
        b.pushwall(13, 13, Wall::Brick);
        assert!(
            b.build().is_err(),
            "a pushwall with one free tile has nowhere to go"
        );
    }
}
