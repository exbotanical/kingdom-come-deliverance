use rltk::Point;
use specs::prelude::*;

use crate::{
    components::{DesiresMelee, Enemy, Position, StatusEffect, StatusEffectType, Viewshed},
    map::Map,
    state::RunState,
};

pub struct EnemyAISystem {}

impl<'a> System<'a> for EnemyAISystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Enemy>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, DesiresMelee>,
        WriteStorage<'a, StatusEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_pos,
            player,
            run_state,
            enemy,
            mut map,
            mut viewsheds,
            mut positions,
            mut desires_melee,
            mut status_effects,
        ) = data;

        if *run_state != RunState::EnemyTurn {
            return;
        }

        for (entity, viewshed, _enemy, pos) in
            (&entities, &mut viewsheds, &enemy, &mut positions).join()
        {
            let mut can_act = true;

            let status_effect = status_effects.get_mut(entity);
            if let Some(effect) = status_effect {
                can_act = match effect.effect {
                    StatusEffectType::Confusion => false,
                };

                effect.turns -= 1;
                if effect.turns < 1 {
                    status_effects.remove(entity);
                }
            }

            if can_act {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

                if distance < 1.5 {
                    desires_melee
                        .insert(entity, DesiresMelee { target: *player })
                        .expect("unable to insert attack");
                } else if viewshed.visible_cells.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &mut *map,
                    );

                    // Check for 2+ steps (where 0 is current location) and move enemy to that location
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);

                        map.blocked[idx] = false;

                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;

                        idx = map.xy_idx(pos.x, pos.y);

                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
