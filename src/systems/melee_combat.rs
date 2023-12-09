use specs::prelude::*;

use crate::{
    components::{CombatStats, Damage, DesiresMelee, Name},
    log::GameLog,
};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, DesiresMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Damage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut desires_melee, names, combat_stats, mut damage) = data;

        for (_entity, desires_melee, name, stats) in
            (&entities, &desires_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(desires_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(desires_melee.target).unwrap();
                    let dmg = i32::max(0, stats.power - target_stats.defense);
                    if dmg == 0 {
                        log.entries.push(format!(
                            "{} did 0 damage to {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {} for {} hp",
                            &name.name, &target_name.name, dmg
                        ));
                        Damage::new_damage(&mut damage, desires_melee.target, dmg);
                    }
                }
            }
        }

        desires_melee.clear();
    }
}
