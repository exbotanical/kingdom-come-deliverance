use rltk::{self, Algorithm2D, BaseMap, Point, RandomNumberGenerator, Tile};
use specs::prelude::*;
use specs::World;
use std::cmp::{max, min};

use crate::components::Player;
use crate::{components::Viewshed, geometry::Rect};

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 50;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Generates a map of rooms and connecting tunnels
    pub fn generate_map_rooms_and_tunnels() -> Map {
        const num_tiles: usize = (MAP_WIDTH * MAP_HEIGHT) as usize;

        let mut map = Map {
            tiles: vec![TileType::Wall; num_tiles],
            revealed_tiles: vec![false; num_tiles],
            visible_tiles: vec![false; num_tiles],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
        };

        // The max number of rooms to generate
        const MAX_ROOMS: i32 = 30;

        // Rand ranges
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            // Generate random width and height
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);

            // Generate the room center
            let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
            let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;

            // Given w/h and x/y, generate the room
            let new_room = Rect::new(x, y, w, h);

            // If the room doesn't overlap with any of the others we've generated...
            if !map
                .rooms
                .iter()
                .any(|other_room| new_room.intersects(other_room))
            {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();

                    // Place tunnels between this room and the previous room
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

pub fn draw_map(ecs: &World, ctx: &mut rltk::Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
        let mut x = 0;
        let mut y = 0;

        for (idx, tile) in map.tiles.iter().enumerate() {
            if map.revealed_tiles[idx] {
                let (glyph, mut fg) = match tile {
                    TileType::Floor => (rltk::to_cp437('.'), rltk::RGB::from_f32(0.0, 0.5, 0.5)),
                    TileType::Wall => (rltk::to_cp437('#'), rltk::RGB::from_f32(0., 1.0, 0.)),
                };

                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }

                ctx.set(x, y, fg, rltk::RGB::from_f32(0., 0., 0.), glyph);
            }

            // Move coords to next
            x += 1;
            if x > MAP_WIDTH - 1 {
                x = 0;
                y += 1;
            }
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> rltk::prelude::Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}