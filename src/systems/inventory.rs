use specs::prelude::*;

use crate::{
    components::{
        CombatStats, DesiresAcquireItem, DesiresDropItem, DesiresUsePotion, InInventory, Name,
        Position, Potion,
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

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, DesiresUsePotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut log, entities, mut desires_potion, names, potions, mut combat_stats) =
            data;

        for (entity, consumer, stats) in (&entities, &desires_potion, &mut combat_stats).join() {
            let potion = potions.get(consumer.potion);
            match potion {
                Some(p) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + p.heal_amt);

                    if entity == *player {
                        log.entries.push(format!(
                            "You consume the {}, healing {} hp",
                            &names.get(consumer.potion).unwrap().name,
                            p.heal_amt
                        ));

                        entities
                            .delete(consumer.potion)
                            .expect("delete potion failed");
                    }
                }
                None => {}
            }
        }

        desires_potion.clear();
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
