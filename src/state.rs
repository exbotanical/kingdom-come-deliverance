use rltk::{self};
use specs::prelude::*;
use specs::World;

use crate::components::DesiresUsePotion;
use crate::components::Name;
use crate::components::Position;
use crate::components::Renderable;
use crate::gui;
use crate::log::GameLog;
use crate::map::draw_map;
use crate::map::Map;
use crate::player::player_input;
use crate::systems::damage;
use crate::systems::inventory::ItemAcquisitionSystem;
use crate::systems::inventory::PotionUseSystem;
use crate::systems::DamageSystem;
use crate::systems::MapIndexingSystem;
use crate::systems::MeleeCombatSystem;
use crate::systems::MonsterAISystem;
use crate::systems::VisibilitySystem;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAISystem {};
        mob.run_now(&self.ecs);

        let mut map_idx = MapIndexingSystem {};
        map_idx.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        let mut acquisitions = ItemAcquisitionSystem {};
        acquisitions.run_now(&self.ecs);

        let mut potions = PotionUseSystem {};
        potions.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut rltk::Rltk) {
        ctx.cls();

        let mut run_state = *self.ecs.fetch::<RunState>();

        match run_state {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain(); // cleanup delete items during systems run
                run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let selected_item = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<DesiresUsePotion>();
                        let entity = self.ecs.fetch::<Entity>();

                        intent
                            .insert(
                                *entity,
                                DesiresUsePotion {
                                    potion: selected_item,
                                },
                            )
                            .expect("Failed to insert intent");
                        run_state = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = run_state;
        }

        damage::reap(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}
