use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Consumable, DesiresAcquireItem, DesiresDropItem, DesiresUseItem, InInventory,
        Name, Position, ProvidesHealing,
    },
    log::GameLog,
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
        Entities<'a>,
        WriteStorage<'a, DesiresUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player,
            mut log,
            entities,
            mut desires_use,
            names,
            consumables,
            heals,
            mut combat_stats,
        ) = data;

        for (entity, desires_use) in (&entities, &desires_use).join() {
            let mut used_item = false;
            let mut targets: Vec<Entity> = Vec::new();

            match desires_use.target {
                None => targets.push(*player),
                Some(t) => {}
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

                            if entity == *player {
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
