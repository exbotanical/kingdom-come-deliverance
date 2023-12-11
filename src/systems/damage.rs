use specs::prelude::*;

use crate::{
    components::{CombatStats, Damage, Name, Player},
    log,
};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, CombatStats>, WriteStorage<'a, Damage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn reap(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let entities = ecs.entities();
        let players = ecs.read_storage::<Player>();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let names = ecs.read_storage::<Name>();

        let mut log = ecs.write_resource::<log::GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let target_name = names.get(entity);
                        if let Some(n) = target_name {
                            log.entries.push(format!("{} has died", &n.name))
                        }
                        dead.push(entity)
                    }
                    Some(_) => log.entries.push("you have died... :(".to_string()),
                }
            }
        }
    }

    for target in dead {
        ecs.delete_entity(target).expect("unable to delete in reap");
    }
}
