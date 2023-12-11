use specs::prelude::*;

use crate::{
    components::{CombatStats, Damage, DesiresMelee, Name},
    log::GameLog,
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresMelee>,
        WriteStorage<'a, Damage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, names, combat_stats, mut log, mut desires_melee, mut damages) = data;

        for (_entity, melee_intent, name, stats) in
            (&entities, &desires_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(melee_intent.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(melee_intent.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);
                    if damage == 0 {
                        log.entries.push(format!(
                            "{} did 0 damage to {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {} for {} hp",
                            &name.name, &target_name.name, damage
                        ));
                        Damage::new_damage(&mut damages, melee_intent.target, damage);
                    }
                }
            }
        }

        desires_melee.clear();
    }
}
