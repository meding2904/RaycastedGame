//! Turns game state into what the raycaster engine needs: a world of wall tiles, an animated door

use raycaster::{Raycaster, Sprite, Tile, WorldMap};

use crate::assets::sprites::WeaponKind;
use crate::assets::textures::Tex;
use crate::assets::Assets;
use crate::game::Game;
use crate::objects::{sprite_move_y, sprite_shrink};

pub const FRAME_WIDTH: usize = 320;
pub const FRAME_HEIGHT: usize = 200;
pub const VIEW_WIDTH: usize = 320;
pub const VIEW_HEIGHT: usize = 160;

/// Sprites further away than this are not worth handing to the engine, the fog eats them anyway
const SPRITE_CULL_DISTANCE: f32 = 34.0;

pub struct Renderer {
    pub world: WorldMap,
    caster: Raycaster,
    image_textures: usize,
    image_actors: usize,
    image_objects: usize,
    image_weapons: usize,
    weapon_atlas_columns: usize,
    /// How many tiles the floor's pushwalls have moved, so a move can trigger a rebuild
    pushwall_tiles: i32,
}

impl Renderer {
    pub fn new(assets: &Assets) -> Self {
        let mut renderer = Self {
            world: WorldMap::new(),
            caster: Raycaster::new(),
            image_textures: 0,
            image_actors: 0,
            image_objects: 0,
            image_weapons: 0,
            weapon_atlas_columns: 1,
            pushwall_tiles: 0,
        };

        renderer.caster.set_anim_time(200);
        renderer.install_images(assets);
        renderer
    }

    /// Uploads the four atlases into a fresh world. The engine copies them, so this only has to
    fn install_images(&mut self, assets: &Assets) {
        self.world = WorldMap::new();
        self.image_textures = self.world.add_image(
            assets.textures.atlas.data.clone(),
            assets.textures.atlas.width as u32,
            assets.textures.atlas.height as u32,
        );
        self.image_actors = self.world.add_image(
            assets.actors.atlas.data.clone(),
            assets.actors.atlas.width as u32,
            assets.actors.atlas.height as u32,
        );
        self.image_objects = self.world.add_image(
            assets.objects.atlas.data.clone(),
            assets.objects.atlas.width as u32,
            assets.objects.atlas.height as u32,
        );
        self.image_weapons = self.world.add_image(
            assets.weapons.atlas.data.clone(),
            assets.weapons.atlas.width as u32,
            assets.weapons.atlas.height as u32,
        );
        self.weapon_atlas_columns = (assets.weapons.atlas.cols).max(1) as usize;

        self.world.set_fog([6, 6, 8, 255], 22.0);
    }

    /// The image ids inside the engine's world, in the order they were added
    pub fn image_ids(&self) -> (usize, usize, usize, usize) {
        (
            self.image_textures,
            self.image_actors,
            self.image_objects,
            self.image_weapons,
        )
    }

    /// Rebuilds the world for a floor: every wall tile, and the floor and ceiling of every area
    pub fn enter_floor(&mut self, game: &Game, assets: &Assets) {
        let level = game.level();
        self.install_images(assets);
        self.pushwall_tiles = game.pushwalls.iter().map(|pushwall| pushwall.moved).sum();

        let tile_of =
            |tex: Tex| -> Tile { assets.textures.tile_with_image(self.image_textures, tex) };

        self.world.set_default_floor(tile_of(level.floor));
        self.world.set_default_ceiling(tile_of(level.ceiling));

        for y in 0..crate::level::MAP_HEIGHT {
            for x in 0..crate::level::MAP_WIDTH {
                let wall = level.wall(x, y);
                if let Some(texture) = wall.tex() {
                    let rect = assets.textures.atlas.tex_rect(texture.index());
                    let tile = if wall.is_door() {
                        Tile::door(self.image_textures, rect)
                    } else {
                        Tile::textured(self.image_textures, rect)
                    };
                    self.world.set_wall(x, y, tile);
                }

                let (floor, ceiling) = level.tiles_at(x, y);
                if floor != level.floor {
                    self.world.set_floor(x, y, tile_of(floor));
                }
                if ceiling != level.ceiling {
                    self.world.set_ceiling(x, y, tile_of(ceiling));
                }
            }
        }
    }

    /// Copies the parts that change every tick: door openings, sliding walls and sprites
    pub fn sync(&mut self, game: &Game, assets: &Assets) {
        let player = &game.player;

        for door in &game.doors {
            if let Some(tile) = self.world.wall_mut(door.x, door.y) {
                tile.set_opening(door.opening);
            }
        }

        let pushwall_tiles: i32 = game.pushwalls.iter().map(|pushwall| pushwall.moved).sum();
        if pushwall_tiles != self.pushwall_tiles {
            self.pushwall_tiles = pushwall_tiles;
            self.enter_floor(game, assets);
        }

        // --- Sprites: enemies, then everything lying on the floor
        self.world.clear_sprites();

        let (px, py) = (player.x, player.y);
        let camera_angle = player.angle;
        let too_far = |x: f32, y: f32| -> bool {
            let (dx, dy) = (x - px, y - py);
            dx * dx + dy * dy > SPRITE_CULL_DISTANCE * SPRITE_CULL_DISTANCE
        };

        for actor in &game.actors {
            if too_far(actor.x, actor.y) {
                continue;
            }

            let view = crate::assets::sprites::ActorSprites::view_for(actor.angle, camera_angle);
            let cell = assets.actors.cell(actor.kind, view, actor.view_slot());
            let rect = assets.actors.tex_rect(cell);

            self.world.add_sprite(Sprite::new(
                actor.x,
                actor.y,
                Tile::textured(self.image_actors, rect),
            ));
        }

        for object in &game.objects {
            if object.taken {
                continue;
            }
            let Some(kind) = object.sprite() else {
                continue;
            };
            let (x, y) = (object.x as f32 + 0.5, object.y as f32 + 0.5);
            if too_far(x, y) {
                continue;
            }

            let rect = assets.objects.tex_rect(kind);
            let mut sprite = Sprite::new(x, y, Tile::textured(self.image_objects, rect));
            sprite.shrink = sprite_shrink(object.thing);
            sprite.move_y = sprite_move_y(object.thing);
            self.world.add_sprite(sprite);
        }
    }

    /// Renders the 3D view into the top of the frame
    pub fn draw_view(&mut self, game: &Game, frame: &mut [u8]) {
        self.caster.set_pos(game.player.x, game.player.y);
        self.caster.set_dir_angle(game.player.angle);

        self.caster.render(
            frame,
            (0, 0, VIEW_WIDTH, VIEW_HEIGHT),
            FRAME_WIDTH,
            &mut self.world,
        );
    }

    /// The weapon in the player's hands, drawn over the bottom of the view
    pub fn draw_weapon(&self, game: &Game, frame: &mut [u8]) {
        let weapon = &game.player.weapon;
        let frame_index = if weapon.firing == crate::weapons::Firing::Firing {
            weapon.frame
        } else {
            0
        };

        let Some((data, width, _)) = self.world.get_image(self.image_weapons).cloned() else {
            return;
        };

        let cell = weapon_cell(weapon.kind, frame_index);
        let columns = self.weapon_atlas_columns.max(1);
        let cell_size = crate::assets::weapons_art::WEAPON_CELL as usize;
        let atlas_width = width as usize;
        let src_x = (cell % columns) * cell_size;
        let src_y = (cell / columns) * cell_size;

        let dst_x = (VIEW_WIDTH as i32 - cell_size as i32) / 2;
        let dst_y = VIEW_HEIGHT as i32 - cell_size as i32;

        for y in 0..cell_size {
            for x in 0..cell_size {
                let src = ((src_y + y) * atlas_width + src_x + x) * 4;
                if src + 4 > data.len() || data[src + 3] == 0 {
                    continue;
                }

                let (dx, dy) = (dst_x + x as i32, dst_y + y as i32);
                if dx < 0 || dy < 0 || dx >= FRAME_WIDTH as i32 || dy >= FRAME_HEIGHT as i32 {
                    continue;
                }

                let dst = (dy as usize * FRAME_WIDTH + dx as usize) * 4;
                if dst + 4 <= frame.len() {
                    frame[dst..dst + 4].copy_from_slice(&data[src..src + 4]);
                }
            }
        }
    }

    /// Blits one atlas cell of any of the game's images, used by the status bar
    pub fn blit_cell(
        &self,
        image: usize,
        cell: usize,
        columns: usize,
        x: i32,
        y: i32,
        frame: &mut [u8],
    ) {
        let Some((data, width, _)) = self.world.get_image(image) else {
            return;
        };
        let (cell_size, atlas_width) = (64usize, *width as usize);
        let columns = columns.max(1);
        let src_x = (cell % columns) * cell_size;
        let src_y = (cell / columns) * cell_size;
        let data = data.clone();

        for sy in 0..cell_size {
            for sx in 0..cell_size {
                let src = ((src_y + sy) * atlas_width + src_x + sx) * 4;
                if src + 4 > data.len() || data[src + 3] == 0 {
                    continue;
                }
                let (dx, dy) = (x + sx as i32, y + sy as i32);
                if dx < 0 || dy < 0 || dx >= FRAME_WIDTH as i32 || dy >= FRAME_HEIGHT as i32 {
                    continue;
                }
                let dst = (dy as usize * FRAME_WIDTH + dx as usize) * 4;
                if dst + 4 <= frame.len() {
                    frame[dst..dst + 4].copy_from_slice(&data[src..src + 4]);
                }
            }
        }
    }

    /// Fills the frame with black, ready for a fresh view
    pub fn clear(frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }
    }
}

/// The weapon atlas cell for a weapon and animation frame. The layout follows the order of
pub fn weapon_cell(kind: WeaponKind, frame: usize) -> usize {
    let mut cell = 0;
    for probe in WeaponKind::ALL {
        if probe == kind {
            break;
        }
        cell += probe.frames();
    }
    cell + frame.min(kind.frames() - 1)
}

/// How many cells wide the texture atlas is, for blitting a cell
pub fn texture_columns(assets: &Assets) -> usize {
    assets.textures.atlas.cols.max(1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_cells_do_not_overlap() {
        let mut seen = Vec::new();
        for kind in WeaponKind::ALL {
            for frame in 0..kind.frames() {
                let cell = weapon_cell(kind, frame);
                assert!(!seen.contains(&cell), "cell {cell} is used twice");
                seen.push(cell);
            }
        }
    }

    #[test]
    fn a_frame_past_the_end_of_an_animation_is_clamped() {
        assert_eq!(
            weapon_cell(WeaponKind::Knife, 99),
            weapon_cell(WeaponKind::Knife, 1)
        );
    }

    #[test]
    fn the_frame_is_black_before_anything_is_drawn_into_it() {
        let mut frame = vec![255u8; FRAME_WIDTH * FRAME_HEIGHT * 4];
        Renderer::clear(&mut frame);
        assert!(frame.chunks_exact(4).all(|p| p == [0, 0, 0, 255]));
    }
}
