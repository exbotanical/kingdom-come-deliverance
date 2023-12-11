use specs::prelude::*;

use crate::{
    components::{BlocksCell, Position},
    map::Map,
};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksCell>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, blockers, mut map) = data;

        map.populate_blocked();
        map.clear_content_idx();

        for (entity, pos) in (&entities, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            let maybe_blocks_cell = blockers.get(entity);
            if let Some(_blocks_cell) = maybe_blocks_cell {
                map.blocked[idx] = true;
            }

            // Copy entity into the indexed slot
            map.cell_content[idx].push(entity);
        }
    }
}
