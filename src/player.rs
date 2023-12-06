use rltk::{self};
use specs::prelude::*;
use std::cmp::{max, min};

use crate::{
    components::{Player, Position, Viewshed},
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    state::State,
};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        if map.tiles[dest_idx] != TileType::Wall {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut rltk::Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            rltk::VirtualKeyCode::Left
            | rltk::VirtualKeyCode::Numpad4
            | rltk::VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            rltk::VirtualKeyCode::Right
            | rltk::VirtualKeyCode::Numpad6
            | rltk::VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            rltk::VirtualKeyCode::Up | rltk::VirtualKeyCode::Numpad8 | rltk::VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            rltk::VirtualKeyCode::Down
            | rltk::VirtualKeyCode::Numpad2
            | rltk::VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            _ => {}
        },
    }
}
