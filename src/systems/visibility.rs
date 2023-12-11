use rltk::{field_of_view, Point};
use specs::prelude::*;

use crate::{
    components::{Player, Position, Viewshed},
    map::Map,
};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Player>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, mut map, mut viewsheds, positions) = data;

        for (entitites, viewshed, pos) in (&entities, &mut viewsheds, &positions).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_cells.clear();
                viewshed.visible_cells =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                // Retain only cells within the bounds
                viewshed
                    .visible_cells
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                let maybe_player = player.get(entitites);
                if let Some(_player) = maybe_player {
                    for cell in map.visible_cells.iter_mut() {
                        *cell = false
                    }

                    for vis in viewshed.visible_cells.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);

                        map.revealed_cells[idx] = true;
                        map.visible_cells[idx] = true;
                    }
                }
            }
        }
    }
}
