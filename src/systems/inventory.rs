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
        ReadStorage<'a, Name>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresAcquireItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, names, mut log, mut desires_item, mut positions, mut inventory) = data;

        for acquisition in desires_item.join() {
            inventory
                .insert(
                    acquisition.item,
                    InInventory {
                        owner: acquisition.acquired_by,
                    },
                )
                .expect("failed cx to insert into inventory");

            positions.remove(acquisition.item);

            if acquisition.acquired_by == *player {
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
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Map>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresUseItem>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Damage>,
        WriteStorage<'a, StatusEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player,
            map,
            names,
            consumables,
            heals,
            damages,
            aoe,
            mut log,
            mut desires_use,
            mut combat_stats,
            mut damage,
            mut status_effects,
        ) = data;

        for (acting_entity, use_intent) in (&entities, &desires_use).join() {
            let mut used_item = false;
            let mut targets: Vec<Entity> = Vec::new();

            match use_intent.target {
                None => {
                    targets.push(*player);
                }

                Some(target) => {
                    let maybe_area = aoe.get(use_intent.item);
                    let idx = map.xy_idx(target.x, target.y);

                    match maybe_area {
                        None => {
                            for t in map.cell_content[idx].iter() {
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

                                for t in map.cell_content[idx].iter() {
                                    targets.push(*t);
                                }
                            }
                        }
                    }
                }
            }

            let item_heals = heals.get(use_intent.item);
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
                                    &names.get(use_intent.item).unwrap().name,
                                    healer.heal_amount
                                ));
                                used_item = true;
                            }
                        }
                    }
                }
            }

            let item_damages = damages.get(use_intent.item);
            match item_damages {
                None => {}
                Some(damager) => {
                    used_item = false;

                    for target in targets.iter() {
                        Damage::new_damage(&mut damage, *target, damager.damage);

                        if acting_entity == *player {
                            let target_name = names.get(*target).unwrap();
                            let item_name = names.get(use_intent.item).unwrap();

                            log.entries.push(format!(
                                "You use {} on {}, inflicting {} hp",
                                item_name.name, target_name.name, damager.damage
                            ))
                        }

                        used_item = true;
                    }
                }
            }

            let mut affected_targets = Vec::new();
            {
                let effect = status_effects.get(use_intent.item);

                match effect {
                    None => {}
                    Some(effect) => {
                        used_item = false;

                        for target in targets.iter() {
                            affected_targets.push((*target, effect.clone()));

                            if acting_entity == *player {
                                let target_name = names.get(*target).unwrap();
                                let item_name = names.get(use_intent.item).unwrap();

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

            for target in affected_targets.iter() {
                status_effects
                    .insert(target.0, target.1.clone())
                    .expect("failed to insert effect");
            }

            if used_item {
                let consumable = consumables.get(use_intent.item);
                match consumable {
                    Some(_) => {
                        entities
                            .delete(use_intent.item)
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
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresDropItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, names, mut log, mut desires_drop, mut positions, mut inventory) =
            data;

        for (entity, drop_intent) in (&entities, &desires_drop).join() {
            let current_pos = positions.get(entity).unwrap();
            let mut drop_pos = Position { x: 0, y: 0 };

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
