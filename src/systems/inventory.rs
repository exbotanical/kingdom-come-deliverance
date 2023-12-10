use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, CombatStats, Consumable, Damage, DesiresAcquireItem, DesiresDropItem,
        DesiresUseItem, InInventory, InflictsDamage, Name, Position, ProvidesHealing, StatusEffect,
    },
    log::GameLog,
    map::Map,
};

pub struct ItemAcquisitionSystem {}

impl<'a> System<'a> for ItemAcquisitionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresAcquireItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, mut desires_item, mut positions, names, mut inventory) = data;

        for acquisition in desires_item.join() {
            positions.remove(acquisition.item);
            inventory
                .insert(
                    acquisition.item,
                    InInventory {
                        owner: acquisition.acquired_by,
                    },
                )
                .expect("failed to insert into inventory");

            if acquisition.acquired_by == *player_entity {
                log.entries.push(format!(
                    "You pick up the {}",
                    names.get(acquisition.item).unwrap().name
                ));
            }
        }

        desires_item.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, DesiresUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, StatusEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player,
            mut log,
            map,
            entities,
            mut desires_use,
            names,
            consumables,
            heals,
            damages,
            mut combat_stats,
            mut damage,
            aoe,
            mut status_effects,
        ) = data;

        for (acting_entity, desires_use) in (&entities, &desires_use).join() {
            let mut used_item = false;
            let mut targets: Vec<Entity> = Vec::new();

            match desires_use.target {
                None => {
                    targets.push(*player);
                }

                Some(target) => {
                    let area = aoe.get(desires_use.item);
                    let idx = map.xy_idx(target.x, target.y);

                    match area {
                        None => {
                            for t in map.tile_content[idx].iter() {
                                targets.push(*t);
                            }
                        }

                        Some(area) => {
                            let mut blast_cells = rltk::field_of_view(target, area.radius, &*map);
                            blast_cells.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });

                            for cell_idx in blast_cells.iter() {
                                let idx = map.xy_idx(cell_idx.x, cell_idx.y);

                                for t in map.tile_content[idx].iter() {
                                    targets.push(*t);
                                }
                            }
                        }
                    }
                }
            }

            let item_heals = heals.get(desires_use.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    used_item = false;

                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);

                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);

                            if acting_entity == *player {
                                log.entries.push(format!(
                                    "You consume the {}, healing {} hp",
                                    &names.get(desires_use.item).unwrap().name,
                                    healer.heal_amount
                                ));
                                used_item = true;
                            }
                        }
                    }
                }
            }

            let item_damages = damages.get(desires_use.item);
            match item_damages {
                None => {}
                Some(damager) => {
                    used_item = false;

                    for target in targets.iter() {
                        Damage::new_damage(&mut damage, *target, damager.damage);

                        if acting_entity == *player {
                            let target_name = names.get(*target).unwrap();
                            let item_name = names.get(desires_use.item).unwrap();

                            log.entries.push(format!(
                                "You use {} on {}, inflicting {} hp",
                                item_name.name, target_name.name, damager.damage
                            ))
                        }

                        used_item = true;
                    }
                }
            }

            let mut apply_effect = Vec::new();
            {
                let effect = status_effects.get(desires_use.item);

                match effect {
                    None => {}
                    Some(effect) => {
                        used_item = false;

                        for target in targets.iter() {
                            apply_effect.push((*target, effect.clone()));

                            if acting_entity == *player {
                                let target_name = names.get(*target).unwrap();
                                let item_name = names.get(desires_use.item).unwrap();

                                log.entries.push(format!(
                                    "You use {} on {}, {} them.",
                                    item_name.name, target_name.name, effect.print_as
                                ));
                            }

                            used_item = true;
                        }
                    }
                }
            }

            for affected in apply_effect.iter() {
                status_effects
                    .insert(affected.0, affected.1.clone())
                    .expect("failed to insert effect");
            }

            if used_item {
                let consumable = consumables.get(desires_use.item);
                match consumable {
                    Some(_) => {
                        entities
                            .delete(desires_use.item)
                            .expect("delete item failed");
                    }
                    None => {}
                }
            }
        }

        desires_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, DesiresDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut log, entities, mut desires_drop, names, mut positions, mut inventory) =
            data;

        for (entity, drop_intent) in (&entities, &desires_drop).join() {
            let mut drop_pos = Position { x: 0, y: 0 };

            let current_pos = positions.get(entity).unwrap();
            drop_pos.x = current_pos.x;
            drop_pos.y = current_pos.y;

            positions
                .insert(drop_intent.item, drop_pos)
                .expect("failed to insert drop position");

            inventory.remove(drop_intent.item);

            if entity == *player {
                log.entries.push(format!(
                    "You dropped the {}",
                    names.get(drop_intent.item).unwrap().name,
                ));
            }
        }

        desires_drop.clear();
    }
}
