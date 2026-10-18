//! The game: one fixed timestep simulation of a floor, its doors, its garrison and its loot.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::actors::{Actor, ActorState};
use crate::assets::sfx::Sfx;
use crate::assets::sprites;
use crate::assets::sprites::WeaponKind;
use crate::difficulty::Difficulty;
use crate::input::Input;
use crate::level::{Key, Level, Rect, Thing, Wall, MAP_HEIGHT, MAP_WIDTH};
use crate::objects::{pickup_for, sprite_for, MapObject, Pickup};
use crate::player::{Player, PLAYER_RADIUS, RUN_SPEED, TURN_SPEED, WALK_SPEED};

/// The original's game loop ran at seventy ticks a second, and all the tuning below is for that
pub const TICK_RATE: f32 = 70.0;
pub const TICK_DT: f32 = 1.0 / TICK_RATE;

/// How far a shot travels before it hits nothing at all
const MAX_SHOT_RANGE: f32 = 32.0;

/// The radius an actor occupies, a little smaller than the player's so they fit through doors
const ACTOR_RADIUS: f32 = 0.28;

/// How far a shout or a shot carries, in tiles
const ALERT_RADIUS_SHOT: f32 = 14.0;
const ALERT_RADIUS_SIGHT: f32 = 9.0;

/// The player can slip through a door once it is this far open, as in the original
const DOOR_PASSABLE: f32 = 0.75;
const DOOR_SPEED: f32 = 1.5;
const DOOR_AUTO_CLOSE: f32 = 5.0;

/// A pushwall travels two tiles, taking this long over each of them
const PUSHWALL_TILE_SECONDS: f32 = 0.7;

/// One frame of an enemy's attack animation. Raise, aim, fire, recover: a guard needs about a
const ATTACK_FRAME_SECONDS: f32 = 0.11;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[derive(Clone, Debug)]
pub struct Door {
    pub x: i32,
    pub y: i32,
    pub wall: Wall,
    /// Doors that need a key
    pub key: Option<Key>,
    pub state: DoorState,
    pub opening: f32,
    /// Counts down while the door stands open, then it closes again
    pub timer: f32,
}

impl Door {
    pub fn is_passable(&self) -> bool {
        self.opening >= DOOR_PASSABLE
    }
}

#[derive(Clone, Debug)]
pub struct Pushwall {
    /// Where the wall is right now, it moves as it slides
    pub x: i32,
    pub y: i32,
    pub wall: Wall,
    pub dir: Option<(i32, i32)>,
    /// Tiles travelled so far, 0.0 to 2.0, used for the animation
    pub progress: f32,
    /// Whole tiles the wall has actually moved, which is what changes the map
    pub moved: i32,
    pub moving: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// The difficulty screen the original opens with
    DifficultySelect,
    Playing,
    Paused,
    /// The player is dying, and the view drops to the floor
    Dying,
    /// The floor has been finished and the screen is fading out
    FloorComplete,
    /// The end of floor tally
    Intermission,
    GameOver,
    Victory,
}

impl Mode {
    /// Whether the mouse is steering the view. The window hides and grabs the cursor exactly here,
    /// so the pointer is out of the way in a firefight and back for the menus and the tally.
    pub fn captures_mouse(self) -> bool {
        matches!(self, Mode::Playing | Mode::Dying)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub kills: i32,
    pub kills_total: i32,
    pub secrets: i32,
    pub secrets_total: i32,
    pub treasure: i32,
    pub treasure_total: i32,
    pub time: f32,
    pub par_time: f32,
    /// Points awarded for the tally, added to the score as it is shown
    pub bonus: i32,
}

impl Stats {
    pub fn kill_percent(&self) -> i32 {
        percent(self.kills, self.kills_total)
    }

    pub fn secret_percent(&self) -> i32 {
        percent(self.secrets, self.secrets_total)
    }

    pub fn treasure_percent(&self) -> i32 {
        percent(self.treasure, self.treasure_total)
    }
}

fn percent(part: i32, total: i32) -> i32 {
    if total <= 0 {
        return 100;
    }
    (part as f32 / total as f32 * 100.0).round() as i32
}

pub struct Game {
    floors: Vec<Level>,
    pub floor_index: usize,
    /// A pristine copy of the current floor, used when the player dies and the floor restarts
    level: Level,
    pub doors: Vec<Door>,
    pub pushwalls: Vec<Pushwall>,
    pub actors: Vec<Actor>,
    pub objects: Vec<MapObject>,
    pub player: Player,
    pub difficulty: Difficulty,
    pub stats: Stats,
    pub mode: Mode,
    pub mode_timer: f32,
    pub message: Option<(String, f32)>,
    /// Counts down while the "Floor 1" card is on screen at the start of a floor
    pub intro_timer: f32,
    /// Sounds the game wants played this tick
    pub sounds: Vec<Sfx>,
    /// Set on the frame a shot is fired, so the renderer can flash the screen
    pub muzzle_flash: f32,
    pub rng: StdRng,
    /// Door and pushwall lookups, so collision does not have to search a list per step
    door_at: Vec<u16>,
    pushwall_at: Vec<u16>,
    /// Which secrets have been found
    secrets_found: Vec<bool>,
    difficulty_choice: usize,
}

impl Game {
    pub fn new() -> Self {
        let floors = crate::levels::floors();
        let mut game = Self {
            floors,
            floor_index: 0,
            level: Level {
                name: String::new(),
                walls: vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize],
                things: vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize],
                floor: crate::assets::textures::Tex::FloorStone,
                ceiling: crate::assets::textures::Tex::CeilDark,
                regions: Vec::new(),
                secrets: Vec::new(),
                pushwalls: Vec::new(),
                locked: Vec::new(),
                start: (2.5, 2.5, 0.0),
            },
            doors: Vec::new(),
            pushwalls: Vec::new(),
            actors: Vec::new(),
            objects: Vec::new(),
            player: Player::new(2.5, 2.5, 0.0),
            difficulty: Difficulty::Normal,
            stats: Stats::default(),
            mode: Mode::DifficultySelect,
            mode_timer: 0.0,
            message: None,
            intro_timer: 0.0,
            sounds: Vec::new(),
            muzzle_flash: 0.0,
            rng: StdRng::seed_from_u64(0x5EED_1234),
            door_at: vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            pushwall_at: vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            secrets_found: Vec::new(),
            difficulty_choice: 0,
        };

        game.load_floor(0);

        game.mode = Mode::DifficultySelect;
        game.intro_timer = 0.0;
        game
    }

    /// Rebuilds the floor's contents from the pristine copy: used at startup, on a new floor, and
    pub fn load_floor(&mut self, index: usize) {
        self.floor_index = index.min(self.floors.len().saturating_sub(1));
        self.level = self.floors[self.floor_index].clone();

        let (sx, sy, angle) = self.level.start;
        self.player.enter_floor(sx, sy, angle);

        self.stats = Stats {
            kills_total: self.level.actor_count(),
            treasure_total: self.level.treasure_count(),
            secrets_total: self.level.secrets.len() as i32,
            par_time: match index {
                0 => 210.0,
                1 => 300.0,
                _ => 360.0,
            },
            ..Stats::default()
        };

        self.secrets_found = vec![false; self.level.secrets.len()];
        self.doors.clear();
        self.pushwalls.clear();
        self.door_at = vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize];
        self.pushwall_at = vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize];

        for (x, key) in self.collect_doors() {
            let wall = self.level.wall(x.0, x.1);
            self.doors.push(Door {
                x: x.0,
                y: x.1,
                wall,
                key,
                state: DoorState::Closed,
                opening: 0.0,
                timer: 0.0,
            });
            let index = self.doors.len();
            self.door_at[(x.1 * MAP_WIDTH + x.0) as usize] = index as u16;
        }

        for (x, y) in self.level.pushwalls.clone() {
            let wall = self.level.wall(x, y);
            self.pushwalls.push(Pushwall {
                x,
                y,
                wall,
                dir: None,
                progress: 0.0,
                moved: 0,
                moving: false,
            });
            self.pushwall_at[(y * MAP_WIDTH + x) as usize] = self.pushwalls.len() as u16;
        }

        self.actors.clear();
        self.objects.clear();
        let mut seed = 1u32;
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let Some(thing) = self.level.thing(x, y) else {
                    continue;
                };
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);

                if thing.is_actor() {
                    let (kind, ambush) = actor_kind(thing);
                    let angle =
                        (self.player.y - y as f32 - 0.5).atan2(self.player.x - x as f32 - 0.5);
                    self.actors.push(Actor::new(
                        kind,
                        x as f32 + 0.5,
                        y as f32 + 0.5,
                        angle,
                        ambush,
                        seed,
                        self.difficulty,
                    ));
                } else if sprite_for(thing).is_some() {
                    self.objects.push(MapObject::new(x, y, thing));
                }
            }
        }

        self.mode = Mode::Playing;
        self.mode_timer = 0.0;
        self.message = None;
        self.intro_timer = 2.0;
        self.refresh_blocked_tiles();
    }

    fn collect_doors(&self) -> Vec<((i32, i32), Option<Key>)> {
        let mut doors = Vec::new();
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                if self.level.wall(x, y).is_door() {
                    doors.push(((x, y), self.level.lock(x, y)));
                }
            }
        }
        doors
    }

    /// Recomputes the cached lookup tables after a pushwall has moved
    fn refresh_blocked_tiles(&mut self) {
        self.pushwall_at = vec![0; MAP_WIDTH as usize * MAP_HEIGHT as usize];
        for (index, pushwall) in self.pushwalls.iter().enumerate() {
            let tile = (pushwall.y * MAP_WIDTH + pushwall.x) as usize;
            if let Some(slot) = self.pushwall_at.get_mut(tile) {
                *slot = index as u16 + 1;
            }
        }
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn difficulty_label(&self) -> &'static str {
        Difficulty::ALL[self.difficulty_choice.min(Difficulty::ALL.len() - 1)].name()
    }

    pub fn difficulty_choice(&self) -> Difficulty {
        Difficulty::ALL[self.difficulty_choice.min(Difficulty::ALL.len() - 1)]
    }

    /// The highlighted entry on the difficulty screen, 0 based
    pub fn difficulty_index(&self) -> usize {
        self.difficulty_choice
    }

    pub fn floor_name(&self) -> &str {
        &self.level.name
    }

    /// True when a tile stops movement: a wall, a door that is not open enough, or a pushwall that
    pub fn blocks_movement(&self, x: i32, y: i32) -> bool {
        match self.level.wall(x, y) {
            Wall::None => {
                if let Some(index) = self.pushwall_at.get((y * MAP_WIDTH + x) as usize).copied() {
                    if index > 0 {
                        return true;
                    }
                }

                self.objects
                    .iter()
                    .any(|o| !o.taken && o.x == x && o.y == y && o.is_solid())
            }
            wall => {
                if wall.is_door() {
                    if let Some(index) = self.door_at.get((y * MAP_WIDTH + x) as usize).copied() {
                        if index > 0 {
                            return !self.doors[(index - 1) as usize].is_passable();
                        }
                    }
                    return true;
                }
                true
            }
        }
    }

    /// True when a tile hides what is behind it
    pub fn blocks_sight(&self, x: i32, y: i32) -> bool {
        match self.level.wall(x, y) {
            Wall::None => {
                matches!(self.pushwall_at.get((y * MAP_WIDTH + x) as usize), Some(index) if *index > 0)
            }
            wall => {
                if wall.is_door() {
                    match self.door_at.get((y * MAP_WIDTH + x) as usize).copied() {
                        Some(index) if index > 0 => self.doors[(index - 1) as usize].opening < 0.95,
                        _ => true,
                    }
                } else {
                    true
                }
            }
        }
    }

    /// A box test: can something of this radius stand centred on this point?
    pub fn can_stand(&self, x: f32, y: f32, radius: f32) -> bool {
        let corners = [
            (x - radius, y - radius),
            (x + radius, y - radius),
            (x - radius, y + radius),
            (x + radius, y + radius),
        ];

        corners
            .iter()
            .all(|(cx, cy)| !self.blocks_movement(cx.floor() as i32, cy.floor() as i32))
    }

    pub fn line_of_sight(&self, from: (f32, f32), to: (f32, f32)) -> bool {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= 0.001 {
            return true;
        }

        let steps = (distance / 0.2).ceil() as i32;
        for step in 1..steps {
            let t = step as f32 / steps as f32;
            let (x, y) = (from.0 + dx * t, from.1 + dy * t);
            if self.blocks_sight(x.floor() as i32, y.floor() as i32) {
                return false;
            }
        }

        true
    }

    /// Moves a circle, sliding along walls one axis at a time
    fn slide(&self, from: (f32, f32), delta: (f32, f32), radius: f32) -> (f32, f32) {
        let mut x = from.0;
        let mut y = from.1;

        if self.can_stand(x + delta.0, y, radius) {
            x += delta.0;
        }
        if self.can_stand(x, y + delta.1, radius) {
            y += delta.1;
        }

        (x, y)
    }

    /// One tick of the game
    pub fn tick(&mut self, input: &Input) {
        self.sounds.clear();
        self.muzzle_flash = (self.muzzle_flash - TICK_DT * 6.0).max(0.0);
        if let Some((_, timer)) = &mut self.message {
            *timer -= TICK_DT;
            if *timer <= 0.0 {
                self.message = None;
            }
        }

        match self.mode {
            Mode::DifficultySelect => self.tick_difficulty_select(input),
            Mode::Playing => self.tick_playing(input),
            Mode::Paused => {
                if input.escape {
                    self.mode = Mode::Playing;
                } else if input.pause_quit {
                    self.mode = Mode::DifficultySelect;
                    self.intro_timer = 0.0;
                }
            }
            Mode::Dying => {
                self.mode_timer += TICK_DT;
                self.player.tick_timers(TICK_DT);
                self.update_actors(TICK_DT, false);
                if self.player.death_timer > 1.3 {
                    if self.player.lives > 1 {
                        self.player.lives -= 1;
                        let (x, y, angle) = self.level.start;
                        self.player.respawn(x, y, angle);
                        self.load_floor(self.floor_index);
                    } else {
                        self.player.lives = 0;
                        self.mode = Mode::GameOver;
                        self.mode_timer = 0.0;
                    }
                }
            }
            Mode::FloorComplete => {
                self.mode_timer += TICK_DT;
                if self.mode_timer > 1.6 {
                    self.finish_floor();
                }
            }
            Mode::Intermission => {
                if input.confirm() {
                    let next = self.floor_index + 1;
                    if next < self.floors.len() {
                        self.load_floor(next);
                    } else {
                        self.mode = Mode::Victory;
                        self.mode_timer = 0.0;
                    }
                }
            }
            Mode::GameOver | Mode::Victory => {
                if input.confirm() {
                    self.restart_game();
                }
            }
        }
    }

    fn tick_difficulty_select(&mut self, input: &Input) {
        let count = Difficulty::ALL.len();
        if input.down {
            self.difficulty_choice = (self.difficulty_choice + 1) % count;
        }
        if input.up {
            self.difficulty_choice = (self.difficulty_choice + count - 1) % count;
        }
        if input.confirm() {
            self.difficulty = self.difficulty_choice();
            self.player = Player::new(2.5, 2.5, 0.0);
            self.load_floor(0);
        }
    }

    fn restart_game(&mut self) {
        self.player = Player::new(2.5, 2.5, 0.0);
        self.rng = StdRng::seed_from_u64(0x5EED_1234);
        self.load_floor(0);
        self.mode = Mode::DifficultySelect;
    }

    fn tick_playing(&mut self, input: &Input) {
        if input.escape {
            self.mode = Mode::Paused;
            return;
        }

        if self.intro_timer > 0.0 {
            self.intro_timer -= TICK_DT;
            return;
        }

        self.stats.time += TICK_DT;
        self.player.tick_timers(TICK_DT);

        // --- Turning and movement. Angles run clockwise on the map, so increasing one turns right
        let mut angle = self.player.angle + input.mouse_dx * crate::player::MOUSE_SENSITIVITY;
        if input.turn_left {
            angle -= TURN_SPEED * TICK_DT;
        }
        if input.turn_right {
            angle += TURN_SPEED * TICK_DT;
        }
        self.player.angle = angle;

        let speed = if input.run { RUN_SPEED } else { WALK_SPEED };
        let (fx, fy) = self.player.facing();
        let mut delta = (0.0, 0.0);
        if input.forward {
            delta.0 += fx * speed * TICK_DT;
            delta.1 += fy * speed * TICK_DT;
        }
        if input.back {
            delta.0 -= fx * speed * TICK_DT;
            delta.1 -= fy * speed * TICK_DT;
        }
        if input.strafe_left {
            delta.0 += fy * speed * TICK_DT * 0.85;
            delta.1 += -fx * speed * TICK_DT * 0.85;
        }
        if input.strafe_right {
            delta.0 += -fy * speed * TICK_DT * 0.85;
            delta.1 += fx * speed * TICK_DT * 0.85;
        }

        if delta != (0.0, 0.0) {
            let (x, y) = self.slide((self.player.x, self.player.y), delta, PLAYER_RADIUS);
            self.player.x = x;
            self.player.y = y;
            self.player.bob += TICK_DT * if input.run { 9.0 } else { 6.0 };
        }

        // --- Weapons
        if let Some(index) = input.weapon {
            if self.player.weapon.select(index) {
                self.sounds.push(Sfx::GunEmpty);
            }
        }
        if input.shoot && self.player.weapon.begin_fire() {
            self.fire_player_weapon();
        }
        self.player.weapon.advance(TICK_DT);

        // --- Doors, switches and pushwalls
        if input.use_key {
            self.try_use();
        }
        self.update_doors(TICK_DT);
        self.update_pushwalls(TICK_DT);

        // --- Pickups and secrets
        self.check_pickups();
        self.check_secrets();

        // --- Everything else that moves
        self.update_actors(TICK_DT, true);

        if self.player.dead {
            self.mode = Mode::Dying;
            self.mode_timer = 0.0;
            self.sounds.push(Sfx::PlayerDeath);
        }
    }

    fn fire_player_weapon(&mut self) {
        let spec = self.player.weapon.spec();
        let (fx, fy) = self.player.facing();
        let from = (self.player.x, self.player.y);

        self.muzzle_flash = 1.0;
        self.sounds.push(match self.player.weapon.kind {
            WeaponKind::Knife => Sfx::KnifeSwing,
            WeaponKind::Pistol => Sfx::PistolShot,
            WeaponKind::MachineGun => Sfx::MachineGunShot,
            WeaponKind::Chaingun => Sfx::ChaingunShot,
        });

        self.alert_actors(from, ALERT_RADIUS_SHOT, false);

        let dir = (fx, fy);
        let range = spec.range.min(MAX_SHOT_RANGE);

        match self.cast_shot(from, dir, range) {
            Some(index) => {
                let damage = self.rng.random_range(spec.damage.0..=spec.damage.1);
                let killed = self.actors[index].hurt(damage, self.difficulty);
                if spec.melee {
                    self.sounds.push(Sfx::KnifeHit);
                }
                if killed {
                    self.on_actor_killed(index);
                }
            }
            None => {
                if spec.melee {
                    self.sounds.push(Sfx::KnifeHit);
                }
            }
        }
    }

    /// Steps a ray through the level and returns the first living actor it runs into
    fn cast_shot(&self, from: (f32, f32), dir: (f32, f32), range: f32) -> Option<usize> {
        let length = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        if length <= 0.0 {
            return None;
        }
        let dir = (dir.0 / length, dir.1 / length);

        let steps = (range / 0.08).ceil() as i32;
        for step in 1..steps {
            let distance = step as f32 * 0.08;
            let (x, y) = (from.0 + dir.0 * distance, from.1 + dir.1 * distance);

            if self.blocks_sight(x.floor() as i32, y.floor() as i32) {
                return None;
            }

            for (index, actor) in self.actors.iter().enumerate() {
                if !actor.is_alive() {
                    continue;
                }
                let (dx, dy) = (actor.x - x, actor.y - y);
                if dx * dx + dy * dy < 0.4 * 0.4 {
                    return Some(index);
                }
            }
        }

        None
    }

    fn on_actor_killed(&mut self, index: usize) {
        let kind = self.actors[index].kind;
        let (x, y) = (self.actors[index].x, self.actors[index].y);
        self.stats.kills += 1;
        self.sounds.push(kind.death_sound());

        if self.actors[index].drops_ammo {
            self.objects.push(MapObject::new(
                x.floor() as i32,
                y.floor() as i32,
                Thing::AmmoClip,
            ));
        }
    }

    /// Everything within earshot learns where the player is
    fn alert_actors(&mut self, origin: (f32, f32), radius: f32, require_sight: bool) {
        let player = (self.player.x, self.player.y);
        let mut woken = Vec::new();
        for (index, actor) in self.actors.iter().enumerate() {
            if !actor.is_alive() || actor.alerted {
                continue;
            }
            let (dx, dy) = (actor.x - origin.0, actor.y - origin.1);
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            if require_sight && !self.line_of_sight((actor.x, actor.y), player) {
                continue;
            }
            woken.push(index);
        }

        for index in woken {
            let actor = &mut self.actors[index];
            actor.alerted = true;
            actor.state = ActorState::Chase;
            actor.state_timer = 0.0;
        }
    }

    fn update_doors(&mut self, dt: f32) {
        for index in 0..self.doors.len() {
            let (x, y, state) = (
                self.doors[index].x,
                self.doors[index].y,
                self.doors[index].state,
            );
            match state {
                DoorState::Opening => {
                    self.doors[index].opening =
                        (self.doors[index].opening + DOOR_SPEED * dt).min(1.0);
                    if self.doors[index].opening >= 1.0 {
                        self.doors[index].state = DoorState::Open;
                        self.doors[index].timer = DOOR_AUTO_CLOSE;
                    }
                }
                DoorState::Open => {
                    self.doors[index].timer -= dt;
                    if self.doors[index].timer <= 0.0 && !self.doorway_occupied(x, y) {
                        self.doors[index].state = DoorState::Closing;
                        self.sounds.push(Sfx::DoorClose);
                    }
                }
                DoorState::Closing => {
                    self.doors[index].opening =
                        (self.doors[index].opening - DOOR_SPEED * dt).max(0.0);
                    if self.doors[index].opening <= 0.0 {
                        self.doors[index].state = DoorState::Closed;
                    }
                }
                DoorState::Closed => {}
            }
        }
    }

    fn doorway_occupied(&self, x: i32, y: i32) -> bool {
        let inside = |px: f32, py: f32| px.floor() as i32 == x && py.floor() as i32 == y;
        if inside(self.player.x, self.player.y) {
            return true;
        }
        self.actors
            .iter()
            .any(|actor| actor.is_alive() && inside(actor.x, actor.y))
    }

    fn update_pushwalls(&mut self, dt: f32) {
        for index in 0..self.pushwalls.len() {
            if !self.pushwalls[index].moving {
                continue;
            }

            let Some(dir) = self.pushwalls[index].dir else {
                continue;
            };
            self.pushwalls[index].progress =
                (self.pushwalls[index].progress + dt / PUSHWALL_TILE_SECONDS).min(2.0);

            // can express: the destination keeps the wall, the tile it left becomes a passage
            let target = (self.pushwalls[index].progress.floor() as i32).min(2);
            while self.pushwalls[index].moved < target {
                let (cx, cy) = (self.pushwalls[index].x, self.pushwalls[index].y);
                let (nx, ny) = (cx + dir.0, cy + dir.1);
                if !(0..MAP_WIDTH).contains(&nx)
                    || !(0..MAP_HEIGHT).contains(&ny)
                    || self.blocks_movement(nx, ny)
                {
                    self.pushwalls[index].moving = false;
                    self.pushwalls[index].progress = self.pushwalls[index].moved as f32;
                    break;
                }

                self.level.set_wall(cx, cy, Wall::None);
                self.level.set_wall(nx, ny, self.pushwalls[index].wall);
                self.pushwalls[index].x = nx;
                self.pushwalls[index].y = ny;
                self.pushwalls[index].moved += 1;
                self.crush_actors(nx, ny);
            }

            if self.pushwalls[index].progress >= 2.0 {
                self.pushwalls[index].moving = false;
                self.pushwalls[index].progress = 2.0;
            }
        }

        self.refresh_blocked_tiles();
    }

    /// Anything standing in the way of a sliding wall is crushed, as in the original
    fn crush_actors(&mut self, x: i32, y: i32) {
        for index in 0..self.actors.len() {
            if !self.actors[index].is_alive() {
                continue;
            }
            if self.actors[index].x.floor() as i32 != x || self.actors[index].y.floor() as i32 != y
            {
                continue;
            }
            self.actors[index].hurt(1000, Difficulty::Normal);
            self.on_actor_killed(index);
        }

        if self.player.x.floor() as i32 == x && self.player.y.floor() as i32 == y {
            self.player.hurt(20, self.difficulty);
        }
    }

    /// The use key: open a door, push a wall, or call the elevator
    fn try_use(&mut self) {
        let (fx, fy) = self.player.facing();
        let targets = [
            (self.player.x + fx * 0.9, self.player.y + fy * 0.9),
            (self.player.x + fx * 0.4, self.player.y + fy * 0.4),
        ];

        for (tx, ty) in targets {
            let (x, y) = (tx.floor() as i32, ty.floor() as i32);

            if self.level.wall(x, y).is_switch() {
                self.mode = Mode::FloorComplete;
                self.mode_timer = 0.0;
                self.sounds.push(Sfx::ElevatorSwitch);
                return;
            }

            if let Some(index) = self.door_at.get((y * MAP_WIDTH + x) as usize).copied() {
                if index > 0 {
                    let index = (index - 1) as usize;
                    self.open_door(index);
                    return;
                }
            }

            if let Some(index) = self.pushwall_at.get((y * MAP_WIDTH + x) as usize).copied() {
                if index > 0 && self.start_pushwall((index - 1) as usize) {
                    return;
                }
            }
        }
    }

    fn open_door(&mut self, index: usize) {
        let door = &mut self.doors[index];

        if let Some(key) = door.key {
            if !self.player.has_key(key) {
                let name = match key {
                    Key::Gold => "YOU NEED A GOLD KEY",
                    Key::Silver => "YOU NEED A SILVER KEY",
                };
                self.message = Some((name.to_string(), 2.5));
                self.sounds.push(Sfx::DoorLocked);
                return;
            }
        }

        match door.state {
            DoorState::Closed | DoorState::Closing => {
                door.state = DoorState::Opening;
                self.sounds.push(Sfx::DoorOpen);

                self.alert_actors((self.player.x, self.player.y), 6.0, false);
            }
            DoorState::Open => {
                door.state = DoorState::Closing;
                self.sounds.push(Sfx::DoorClose);
            }
            DoorState::Opening => {}
        }
    }

    /// Starts a pushwall sliding. Returns false when it has nowhere to go or is already moving.
    fn start_pushwall(&mut self, index: usize) -> bool {
        if self.pushwalls[index].moving || self.pushwalls[index].progress >= 2.0 {
            return false;
        }

        let (x, y) = (self.pushwalls[index].x, self.pushwalls[index].y);
        let (fx, fy) = self.player.facing();
        // The wall gives way in the direction it is pushed, otherwise along one of the other axes
        let candidates = [
            (fx.round() as i32, fy.round() as i32),
            (fy.round() as i32, -fx.round() as i32),
            (-fy.round() as i32, fx.round() as i32),
            (-(fx.round() as i32), -(fy.round() as i32)),
        ];

        let free = |dir: (i32, i32), level: &Level| -> bool {
            (1..=2).all(|step| {
                let (nx, ny) = (x + dir.0 * step, y + dir.1 * step);
                nx >= 0
                    && ny >= 0
                    && nx < MAP_WIDTH
                    && ny < MAP_HEIGHT
                    && level.wall(nx, ny) == Wall::None
            })
        };

        let Some(dir) = candidates
            .iter()
            .copied()
            .find(|dir| free(*dir, &self.level))
        else {
            return false;
        };

        self.pushwalls[index].dir = Some(dir);
        self.pushwalls[index].moving = true;
        self.sounds.push(Sfx::Pushwall);
        self.alert_actors((x as f32 + 0.5, y as f32 + 0.5), 8.0, false);
        true
    }

    fn check_pickups(&mut self) {
        for index in 0..self.objects.len() {
            if self.objects[index].taken {
                continue;
            }

            let (ox, oy) = (
                self.objects[index].x as f32 + 0.5,
                self.objects[index].y as f32 + 0.5,
            );
            let (dx, dy) = (self.player.x - ox, self.player.y - oy);
            if dx * dx + dy * dy > 0.6 * 0.6 {
                continue;
            }

            let thing = self.objects[index].thing;
            let Some(pickup) = pickup_for(thing, self.difficulty) else {
                continue;
            };

            match pickup {
                Pickup::Health(amount) => {
                    if self.player.health >= crate::player::MAX_HEALTH {
                        continue;
                    }
                    self.player.heal(amount);
                    self.sounds.push(Sfx::PickupHealth);
                }
                Pickup::Ammo(amount) => {
                    self.player.weapon.add_ammo(amount);
                    self.sounds.push(Sfx::PickupAmmo);
                }
                Pickup::Weapon(kind, amount) => {
                    self.player.weapon.grant(kind);
                    self.player.weapon.add_ammo(amount);
                    self.player.weapon.select(kind.index());
                    self.sounds.push(Sfx::PickupAmmo);
                }
                Pickup::Treasure(points) => {
                    self.player.score += points;
                    self.stats.treasure += 1;
                    self.sounds.push(Sfx::PickupTreasure);
                }
                Pickup::Key(key) => {
                    self.player.give_key(key);
                    self.sounds.push(Sfx::PickupKey);
                }
                Pickup::ExtraLife => {
                    self.player.lives += 1;
                    self.player.score += pickup.score();
                    self.sounds.push(Sfx::ExtraLife);
                }
            }

            self.objects[index].taken = true;
        }
    }

    fn check_secrets(&mut self) {
        let (px, py) = (self.player.x.floor() as i32, self.player.y.floor() as i32);
        for (index, secret) in self.level.secrets.clone().iter().enumerate() {
            if self.secrets_found.get(index).copied().unwrap_or(true) {
                continue;
            }
            if secret.contains(px, py) {
                if let Some(found) = self.secrets_found.get_mut(index) {
                    *found = true;
                }
                self.stats.secrets += 1;
            }
        }
    }

    fn update_actors(&mut self, dt: f32, can_attack: bool) {
        let player = (self.player.x, self.player.y);
        let player_alive = !self.player.dead;

        for index in 0..self.actors.len() {
            let mut actor = self.actors[index].clone();
            actor.anim_timer += dt;

            match actor.state {
                ActorState::Dying => {
                    if actor.anim_timer > 0.14 {
                        actor.anim_timer = 0.0;
                        actor.frame += 1;
                        if actor.frame >= 4 {
                            actor.state = ActorState::Dead;
                            actor.frame = 0;
                        }
                    }
                    self.actors[index] = actor;
                    continue;
                }
                ActorState::Dead => continue,
                ActorState::Chase => {
                    if actor.anim_timer > 0.18 {
                        actor.anim_timer = 0.0;
                        actor.frame = (actor.frame + 1) % 4;
                    }
                }
                ActorState::Shoot | ActorState::Attack => {
                    if actor.anim_timer > ATTACK_FRAME_SECONDS {
                        actor.anim_timer = 0.0;
                        actor.frame += 1;

                        if actor.frame == sprites::FIRE_FRAME {
                            self.strike(&mut actor, player, player_alive);
                        }

                        if actor.frame >= sprites::SHOOT_FRAMES {
                            actor.frame = 0;
                            actor.state = ActorState::Chase;
                        }
                    }
                    self.actors[index] = actor;
                    continue;
                }
                _ => {}
            }

            if actor.state == ActorState::Pain {
                actor.state_timer -= dt;
                if actor.state_timer <= 0.0 {
                    actor.state = ActorState::Chase;
                }
                self.actors[index] = actor;
                continue;
            }

            let stats = actor.stats();
            let distance = ((player.0 - actor.x).powi(2) + (player.1 - actor.y).powi(2)).sqrt();
            let sighted = player_alive && self.line_of_sight((actor.x, actor.y), player);
            let sees_player = sighted
                && distance
                    <= if actor.alerted {
                        actor.chase_sight_range()
                    } else {
                        stats.sight_range
                    };

            if !actor.alerted && sees_player {
                actor.alerted = true;
                actor.state = ActorState::Chase;
                if !actor.sighted {
                    actor.sighted = true;
                    self.sounds.push(actor.kind.sight_sound());
                    self.alert_actors((actor.x, actor.y), ALERT_RADIUS_SIGHT, true);
                }
            }

            if !actor.alerted || !player_alive {
                if actor.state != ActorState::Idle {
                    actor.state = ActorState::Idle;
                }
                self.actors[index] = actor;
                continue;
            }

            actor.state_timer -= dt;
            let ready = actor.state_timer <= 0.0;

            if can_attack && ready && sees_player && distance <= stats.attack_range {
                actor.angle = (player.1 - actor.y).atan2(player.0 - actor.x);
                actor.state = if stats.ranged {
                    ActorState::Shoot
                } else {
                    ActorState::Attack
                };
                actor.frame = 0;
                actor.anim_timer = 0.0;
                actor.state_timer = stats.attack_cooldown;
                self.actors[index] = actor;
                continue;
            }

            // Otherwise walk towards the player
            actor.state = ActorState::Chase;
            let (dx, dy) = (player.0 - actor.x, player.1 - actor.y);
            let length = (dx * dx + dy * dy).sqrt().max(0.001);
            let target = (dx / length, dy / length);
            actor.angle = target.1.atan2(target.0);

            let turns = [0.0f32, 0.6, -0.6, 1.2, -1.2];
            for turn in turns {
                let (sin, cos) = turn.sin_cos();
                let dir = (
                    target.0 * cos - target.1 * sin,
                    target.0 * sin + target.1 * cos,
                );
                let step = stats.speed * dt;
                let (nx, ny) = (actor.x + dir.0 * step, actor.y + dir.1 * step);
                if self.can_stand(nx, ny, ACTOR_RADIUS) {
                    actor.x = nx;
                    actor.y = ny;
                    break;
                }
            }

            self.actors[index] = actor;
        }
    }

    /// The moment an attack connects: a bullet if the enemy shoots, a bite if it does not. The
    fn strike(&mut self, actor: &mut Actor, player: (f32, f32), player_alive: bool) {
        if !player_alive {
            return;
        }

        let stats = actor.stats();
        let distance = ((player.0 - actor.x).powi(2) + (player.1 - actor.y).powi(2)).sqrt();
        if distance > stats.attack_range * 1.2 {
            return;
        }
        if !self.line_of_sight((actor.x, actor.y), player) {
            return;
        }

        let damage = if stats.ranged {
            (stats.damage as f32 * (1.0 - distance / (stats.attack_range + 6.0)).max(0.35)) as i32
        } else {
            (stats.damage as f32 * 0.8) as i32
        };

        self.player.hurt(damage.max(1), self.difficulty);
        self.sounds.push(actor.kind.attack_sound());
    }

    /// Called when the player calls the elevator: works out the rating and the bonus
    fn finish_floor(&mut self) {
        let time_bonus =
            ((self.stats.par_time - self.stats.time).max(0.0) / 10.0).round() as i32 * 100;
        let clean_bonus = if self.stats.kill_percent() == 100 {
            5000
        } else {
            0
        };
        let secret_bonus = self.stats.secret_percent() * 50;
        let treasure_bonus = self.stats.treasure_percent() * 40;

        self.stats.bonus = time_bonus + clean_bonus + secret_bonus + treasure_bonus;
        self.player.score += self.stats.bonus;
        self.mode = Mode::Intermission;
        self.mode_timer = 0.0;
    }

    /// Where a secret area lies, for the map dump and the tests
    pub fn secret_at(&self, index: usize) -> Option<Rect> {
        self.level.secrets.get(index).copied()
    }

    /// Skips the intro card, used by the tests and by the frame harness
    pub fn is_paused(&self) -> bool {
        self.mode == Mode::Paused
    }

    pub fn skip_intro(&mut self) {
        self.intro_timer = 0.0;
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn actor_kind(thing: Thing) -> (crate::assets::sprites::ActorKind, bool) {
    use crate::assets::sprites::ActorKind;
    match thing {
        Thing::Guard => (ActorKind::Guard, false),
        Thing::AmbushGuard => (ActorKind::Guard, true),
        Thing::Ss => (ActorKind::Ss, false),
        Thing::Officer => (ActorKind::Officer, false),
        Thing::Dog => (ActorKind::Dog, false),
        Thing::Boss => (ActorKind::Boss, false),
        _ => (ActorKind::Guard, false),
    }
}

/// A convenience for the tests and the harness: run the game for a while with no input
pub fn idle_input() -> Input {
    Input::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started_game(difficulty: Difficulty) -> Game {
        let mut game = Game::new();
        game.difficulty = difficulty;
        game.player = Player::new(2.5, 2.5, 0.0);
        game.load_floor(0);
        game.skip_intro();
        game
    }

    #[test]
    fn the_game_opens_on_the_difficulty_screen() {
        let game = Game::new();
        assert_eq!(game.mode, Mode::DifficultySelect);
    }

    #[test]
    fn the_cursor_is_captured_only_while_the_player_is_in_the_floor() {
        assert!(Mode::Playing.captures_mouse());
        assert!(Mode::Dying.captures_mouse());
        for mode in [
            Mode::DifficultySelect,
            Mode::Paused,
            Mode::FloorComplete,
            Mode::Intermission,
            Mode::GameOver,
            Mode::Victory,
        ] {
            assert!(!mode.captures_mouse(), "{mode:?} must hand the cursor back");
        }
    }

    #[test]
    fn a_floor_loads_its_garrison_and_its_loot() {
        let game = started_game(Difficulty::Normal);
        assert_eq!(game.mode, Mode::Playing);
        assert!(game.actors.len() > 5, "floor 1 should have a garrison");
        assert!(game.objects.len() > 5, "floor 1 should have things on it");
        assert_eq!(game.stats.kills_total, game.actors.len() as i32);
        assert!(game.stats.treasure_total > 0);
        assert!(game.stats.secrets_total > 0);
    }

    #[test]
    fn the_player_starts_on_open_floor() {
        let game = started_game(Difficulty::Normal);
        assert!(game.can_stand(game.player.x, game.player.y, PLAYER_RADIUS));
    }

    #[test]
    fn walls_stop_the_player() {
        let game = started_game(Difficulty::Normal);
        assert!(game.blocks_movement(1, 4));
        assert!(!game.can_stand(1.2, 4.5, PLAYER_RADIUS));
    }

    #[test]
    fn walking_forward_moves_the_player_and_walls_hold() {
        let mut game = started_game(Difficulty::Normal);
        let start = (game.player.x, game.player.y);
        let input = Input {
            forward: true,
            ..Input::default()
        };

        for _ in 0..40 {
            game.tick(&input);
        }
        assert!(
            game.player.x > start.0 + 0.5,
            "the player should have walked east"
        );

        for _ in 0..600 {
            game.tick(&input);
        }
        assert!(
            game.player.x < 10.0,
            "the player should be stopped by the cell wall, at {}",
            game.player.x
        );
    }

    #[test]
    fn a_left_key_turns_left_and_a_right_key_turns_right() {
        let mut game = started_game(Difficulty::Normal);
        game.player.angle = 0.0;
        game.tick(&Input {
            turn_left: true,
            ..Input::default()
        });
        assert!(
            game.player.angle < 0.0,
            "turn_left must swing the heading left, got {}",
            game.player.angle
        );

        let mut game = started_game(Difficulty::Normal);
        game.player.angle = 0.0;
        game.tick(&Input {
            turn_right: true,
            ..Input::default()
        });
        assert!(
            game.player.angle > 0.0,
            "turn_right must swing the heading right, got {}",
            game.player.angle
        );
    }

    #[test]
    fn moving_the_mouse_right_turns_right() {
        let mut game = started_game(Difficulty::Normal);
        game.player.angle = 0.0;
        game.tick(&Input {
            mouse_dx: 40.0,
            ..Input::default()
        });
        assert!(
            game.player.angle > 0.0,
            "a mouse pushed right must turn right, got {}",
            game.player.angle
        );
    }

    #[test]
    fn strafing_left_moves_to_the_left_of_where_the_player_faces() {
        let mut game = started_game(Difficulty::Normal);
        game.player.angle = 0.0; // facing east, so the player's left is north, -y
        let (x, y) = (game.player.x, game.player.y);
        game.tick(&Input {
            strafe_left: true,
            ..Input::default()
        });
        assert!(
            game.player.y < y && (game.player.x - x).abs() < 0.2,
            "strafing left from ({} , {}) went to ({}, {})",
            x,
            y,
            game.player.x,
            game.player.y
        );
    }

    #[test]
    fn doors_start_shut_and_open_when_used() {
        let mut game = started_game(Difficulty::Normal);
        assert!(game.blocks_movement(10, 4), "the door should start closed");

        game.player.x = 9.5;
        game.player.y = 4.5;
        game.player.angle = 0.0;

        let input = Input {
            use_key: true,
            ..Input::default()
        };
        game.tick(&input);
        assert!(game.sounds.contains(&Sfx::DoorOpen));

        let idle = Input::default();
        for _ in 0..90 {
            game.tick(&idle);
        }
        assert!(
            !game.blocks_movement(10, 4),
            "the door should be passable once open"
        );
    }

    #[test]
    fn a_locked_door_refuses_to_open_without_the_key() {
        let mut game = started_game(Difficulty::Normal);
        game.player.x = 18.5;
        game.player.y = 17.5;
        game.player.angle = std::f32::consts::FRAC_PI_2;

        let input = Input {
            use_key: true,
            ..Input::default()
        };
        game.tick(&input);

        assert!(game.sounds.contains(&Sfx::DoorLocked));
        assert!(game.blocks_movement(18, 18));
        assert!(
            game.message.is_some(),
            "the player should be told a key is missing"
        );

        game.player.give_key(Key::Gold);
        game.doors.iter_mut().for_each(|d| d.timer = 0.0);
        for _ in 0..5 {
            game.tick(&input);
        }
        let idle = Input::default();
        for _ in 0..90 {
            game.tick(&idle);
        }
        assert!(
            !game.blocks_movement(18, 18),
            "the gold door should open with the gold key"
        );
    }

    #[test]
    fn a_pushwall_slides_two_tiles_and_opens_a_passage() {
        let mut game = started_game(Difficulty::Normal);
        game.player.x = 5.5;
        game.player.y = 7.5;
        game.player.angle = std::f32::consts::FRAC_PI_2;

        assert!(game.blocks_movement(5, 8));
        let input = Input {
            use_key: true,
            ..Input::default()
        };
        game.tick(&input);
        assert!(
            game.sounds.contains(&Sfx::Pushwall),
            "pushing a wall should rumble"
        );

        let idle = Input::default();
        for _ in 0..140 {
            game.tick(&idle);
        }

        assert!(
            !game.blocks_movement(5, 8),
            "the wall should have left its old tile"
        );
        assert!(
            game.blocks_movement(5, 10),
            "the wall should have landed two tiles down"
        );
        assert_eq!(game.pushwalls[0].progress, 2.0);
    }

    #[test]
    fn walking_into_the_stash_counts_a_secret() {
        let mut game = started_game(Difficulty::Normal);
        let before = game.stats.secrets;
        game.player.x = 5.5;
        game.player.y = 10.5;
        game.tick(&Input::default());
        assert!(
            game.stats.secrets > before,
            "walking into the hidden room should count a secret"
        );
    }

    #[test]
    fn shooting_a_guard_in_front_of_you_kills_it_and_scores_a_kill() {
        let mut game = started_game(Difficulty::Normal);
        game.player.x = 13.5;
        game.player.y = 4.5;
        game.player.angle = 0.0;
        game.objects.clear();
        game.actors.retain(|a| (a.x - 15.5).abs() < 0.6);
        assert_eq!(game.actors.len(), 1);

        let before = game.stats.kills;
        for _ in 0..120 {
            game.player.weapon.ammo = 8;
            game.player.weapon.cooldown = 0.0;
            let shoot = Input {
                shoot: true,
                ..Input::default()
            };
            game.tick(&shoot);
            if game.actors[0].state == ActorState::Dying {
                break;
            }
        }

        assert!(
            game.actors[0].health <= 0,
            "the guard should have been shot down"
        );
        assert_eq!(game.stats.kills, before + 1);
    }

    #[test]
    fn the_weapon_cannot_fire_across_the_map() {
        let game = started_game(Difficulty::Normal);
        let hit = game.cast_shot((2.5, 4.5), (1.0, 0.0), 40.0);
        assert!(
            hit.is_none(),
            "the cell wall should stop a shot down the corridor"
        );
    }

    #[test]
    fn a_guard_notices_the_player_and_comes_after_them() {
        let mut game = started_game(Difficulty::Normal);
        game.actors.clear();
        game.actors.push(Actor::new(
            crate::assets::sprites::ActorKind::Guard,
            15.5,
            4.5,
            0.0,
            false,
            1,
            Difficulty::Normal,
        ));
        game.player.x = 11.5;
        game.player.y = 4.5;
        game.player.angle = 0.0;

        let idle = Input::default();
        for _ in 0..60 {
            game.tick(&idle);
        }

        assert!(
            game.actors[0].alerted,
            "the guard should have spotted the player"
        );
        assert!(game.actors[0].x < 15.5, "the guard should be closing in");
    }

    #[test]
    fn an_ambush_guard_waits_until_it_sees_the_player() {
        let mut game = started_game(Difficulty::Normal);
        game.actors.clear();
        game.actors.push(Actor::new(
            crate::assets::sprites::ActorKind::Guard,
            22.5,
            13.5,
            0.0,
            true,
            3,
            Difficulty::Normal,
        ));
        game.player.x = 4.5;
        game.player.y = 4.5;

        let idle = Input::default();
        for _ in 0..100 {
            game.tick(&idle);
        }

        assert!(
            !game.actors[0].alerted,
            "an ambusher should stay put across the floor"
        );
        assert_eq!(game.actors[0].state, ActorState::Idle);
    }

    #[test]
    fn enemies_hurt_the_player_more_on_hard() {
        let damage_on = |difficulty: Difficulty| {
            let mut game = started_game(difficulty);
            game.actors.clear();
            game.actors.push(Actor::new(
                crate::assets::sprites::ActorKind::Officer,
                6.5,
                4.5,
                0.0,
                false,
                0,
                difficulty,
            ));
            game.player.x = 4.5;
            game.player.y = 4.5;
            for _ in 0..200 {
                game.tick(&Input::default());
            }
            crate::player::MAX_HEALTH - game.player.health
        };

        let baby = damage_on(Difficulty::Baby);
        let hard = damage_on(Difficulty::Hard);
        assert!(
            baby > 0,
            "an officer in the same room should hit the player"
        );
        assert!(hard > baby, "hard should hurt more: {baby} vs {hard}");
    }

    #[test]
    fn picking_up_ammo_and_health_and_treasure_works() {
        let mut game = started_game(Difficulty::Normal);
        game.objects.clear();
        game.player.weapon.ammo = 0;
        game.player.health = 50;
        game.objects.push(MapObject::new(4, 4, Thing::AmmoClip));
        game.objects.push(MapObject::new(4, 5, Thing::HealthSmall));
        game.objects
            .push(MapObject::new(5, 4, Thing::TreasureChalice));

        for _ in 0..3 {
            game.player.x = 4.5;
            game.player.y = 4.5;
            game.tick(&Input::default());
            game.player.x = 4.5;
            game.player.y = 5.5;
            game.tick(&Input::default());
            game.player.x = 5.5;
            game.player.y = 4.5;
            game.tick(&Input::default());
        }

        assert_eq!(game.player.weapon.ammo, 8, "a clip is eight rounds");
        assert_eq!(game.player.health, 75, "a first aid kit is twenty five");
        assert_eq!(game.stats.treasure, 1);
        assert_eq!(game.player.score, 500, "a chalice is five hundred points");
        assert!(game.objects.iter().all(|o| o.taken));
    }

    #[test]
    fn the_elevator_switch_ends_the_floor() {
        let mut game = started_game(Difficulty::Normal);
        game.player.x = 23.5;
        game.player.y = 25.5;
        game.player.angle = 0.0;

        let input = Input {
            use_key: true,
            ..Input::default()
        };
        game.tick(&input);

        assert_eq!(game.mode, Mode::FloorComplete);
        assert!(game.sounds.contains(&Sfx::ElevatorSwitch));

        for _ in 0..130 {
            game.tick(&Input::default());
        }
        assert_eq!(game.mode, Mode::Intermission);
    }

    #[test]
    fn the_tally_leads_to_the_next_floor() {
        let mut game = started_game(Difficulty::Normal);
        game.mode = Mode::Intermission;
        let confirm = Input {
            enter: true,
            ..Input::default()
        };
        game.tick(&confirm);
        assert_eq!(game.floor_index, 1, "the tally should move to floor 2");
        assert_eq!(game.mode, Mode::Playing);
    }

    #[test]
    fn the_last_floor_ends_in_a_victory() {
        let mut game = started_game(Difficulty::Normal);
        game.load_floor(game.floors.len() - 1);
        game.mode = Mode::Intermission;
        let confirm = Input {
            enter: true,
            ..Input::default()
        };
        game.tick(&confirm);
        assert_eq!(game.mode, Mode::Victory);
    }

    #[test]
    fn dying_costs_a_life_and_restarts_the_floor() {
        let mut game = started_game(Difficulty::Normal);
        let lives = game.player.lives;
        game.player.health = 1;
        game.player.hurt(50, Difficulty::Normal);

        for _ in 0..24 * 4 {
            game.tick(&Input::default());
        }

        assert_eq!(game.player.lives, lives - 1);
        assert_eq!(game.player.health, crate::player::MAX_HEALTH);
        assert!(!game.player.dead);
        assert_eq!(game.mode, Mode::Playing);
        assert_eq!(game.stats.kills, 0, "a restarted floor resets the tally");
    }

    #[test]
    fn running_out_of_lives_is_game_over() {
        let mut game = started_game(Difficulty::Normal);
        game.player.lives = 1;
        game.player.health = 1;
        game.player.hurt(50, Difficulty::Normal);

        for _ in 0..24 * 5 {
            game.tick(&Input::default());
        }
        assert_eq!(game.mode, Mode::GameOver);
    }

    #[test]
    fn a_guard_raises_its_weapon_before_the_shot_lands() {
        use crate::assets::sprites::ActorKind;

        let mut game = started_game(Difficulty::Normal);
        game.actors.clear();
        game.player.x = 4.5;
        game.player.y = 4.5;
        game.player.angle = 0.0;

        let mut guard = crate::actors::Actor::new(
            ActorKind::Guard,
            6.5,
            4.5,
            std::f32::consts::PI,
            false,
            7,
            Difficulty::Normal,
        );
        guard.alerted = true;
        guard.state = ActorState::Chase;
        guard.state_timer = 0.0;
        game.actors.push(guard);

        let health = game.player.health;
        let mut hit_after = None;
        let mut frames = Vec::new();

        for tick in 0..60 {
            game.tick(&Input::default());
            if let Some(actor) = game.actors.first() {
                frames.push(actor.frame);
            }
            if game.player.health < health {
                hit_after = Some(tick);
                break;
            }
        }

        assert_eq!(
            frames.first().copied(),
            Some(0),
            "the attack starts on the raise frame"
        );
        let tick = hit_after.expect("the guard should hit the player eventually");
        assert!(
            tick >= 8,
            "the shot landed after only {tick} ticks: there is no windup to react to"
        );
        assert!(
            frames.contains(&1),
            "the aim frame has to be shown before the shot: {frames:?}"
        );
    }

    #[test]
    fn the_difficulty_screen_starts_the_game_on_the_chosen_tier() {
        let mut game = Game::new();
        let down = Input {
            down: true,
            ..Input::default()
        };
        game.tick(&down);
        assert_eq!(game.difficulty_index(), 1, "down moves the highlight");

        let up = Input {
            up: true,
            ..Input::default()
        };
        game.tick(&up);
        assert_eq!(game.difficulty_index(), 0, "up moves it back");

        let confirm = Input {
            enter: true,
            ..Input::default()
        };
        game.tick(&confirm);
        assert_eq!(game.mode, Mode::Playing);
        assert_eq!(
            game.difficulty,
            Difficulty::ALL[0],
            "the game starts on the tier the player picked"
        );
    }

    #[test]
    fn a_whole_floor_of_idle_ticks_is_stable() {
        // The floor should survive being watched for a while without panicking
        let mut game = started_game(Difficulty::Normal);
        let idle = Input::default();
        for _ in 0..70 * 5 {
            game.tick(&idle);
        }
        assert!(matches!(game.mode, Mode::Playing | Mode::Dying));
    }
}
