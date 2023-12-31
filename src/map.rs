use crate::components::Player;
use crate::{components::Viewshed, geometry::Rect};
use rltk::{self, Algorithm2D, BaseMap, Point, RandomNumberGenerator};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::World;
use std::cmp::{max, min};

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_COUNT: usize = (MAP_HEIGHT * MAP_WIDTH) as usize;

// The max number of rooms to generate
const MAX_ROOMS: i32 = 30;

// Rand ranges
const MIN_SIZE: i32 = 6;
const MAX_SIZE: i32 = 10;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum CellType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub cells: Vec<CellType>,
    pub revealed_cells: Vec<bool>,
    pub visible_cells: Vec<bool>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    // Is the point blocked by something?
    pub blocked: Vec<bool>,
    // The depth of the map i.e. floors
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub cell_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    /// Visits each vector in cell_content and clears it
    pub fn clear_content_idx(&mut self) {
        for content in self.cell_content.iter_mut() {
            content.clear();
        }
    }

    /// Returns a bool indicating whether the coordinates are a valid exit point (i.e. within bounds) on the map
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        // x, y inside map?
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let idx = self.xy_idx(x, y);

        !self.blocked[idx]
    }

    /// Sets blocked on a cell if its a wall
    pub fn populate_blocked(&mut self) {
        for (i, cell) in self.cells.iter_mut().enumerate() {
            self.blocked[i] = *cell == CellType::Wall;
        }
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.cells[idx] = CellType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.cells[idx] = CellType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.cells[idx] = CellType::Floor;
            }
        }
    }

    /// Generates a map of rooms and connecting tunnels
    pub fn generate_map_rooms_and_tunnels(depth: i32) -> Map {
        let mut map = Map {
            cells: vec![CellType::Wall; MAP_COUNT],
            revealed_cells: vec![false; MAP_COUNT],
            visible_cells: vec![false; MAP_COUNT],
            blocked: vec![false; MAP_COUNT],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            cell_content: vec![Vec::new(); MAP_COUNT],
            depth,
        };

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

        let stairs_pos = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_pos.0, stairs_pos.1);
        map.cells[stairs_idx] = CellType::DownStairs;

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> rltk::prelude::Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.cells[idx] == CellType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::prelude::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };

        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };

        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };

        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }

        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }

        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }

        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);

        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub fn draw_map(ecs: &World, ctx: &mut rltk::Rltk) {
    let map = ecs.fetch::<Map>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, _viewshed) in (&mut players, &mut viewsheds).join() {
        let mut x = 0;
        let mut y = 0;

        for (idx, cell) in map.cells.iter().enumerate() {
            if map.revealed_cells[idx] {
                let (glyph, mut fg) = match cell {
                    CellType::Floor => (rltk::to_cp437('.'), rltk::RGB::from_f32(0., 0.5, 0.5)),
                    CellType::Wall => (rltk::to_cp437('#'), rltk::RGB::from_f32(0., 1., 0.)),
                    CellType::DownStairs => (rltk::to_cp437('>'), rltk::RGB::from_f32(0., 1., 1.)),
                };

                if !map.visible_cells[idx] {
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
