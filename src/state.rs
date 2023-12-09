use rltk::{self};
use specs::prelude::*;
use specs::World;

use crate::components::DesiresDropItem;
use crate::components::DesiresUseItem;
use crate::components::Position;
use crate::components::Renderable;
use crate::gui;
use crate::map::draw_map;
use crate::map::Map;
use crate::player::player_input;
use crate::systems::damage;
use crate::systems::inventory::ItemAcquisitionSystem;
use crate::systems::inventory::ItemDropSystem;
use crate::systems::inventory::ItemUseSystem;
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
    ShowDropItem,
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

        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);

        let mut drops = ItemDropSystem {};
        drops.run_now(&self.ecs);

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
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::Selected => {
                        let selected_item = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<DesiresUseItem>();
                        let entity = self.ecs.fetch::<Entity>();

                        intent
                            .insert(
                                *entity,
                                DesiresUseItem {
                                    item: selected_item,
                                    target: None,
                                },
                            )
                            .expect("Failed to insert intent");
                        run_state = RunState::PlayerTurn;
                    }
                }
            }

            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::Selected => {
                        let item = result.1.unwrap();
                        let player = self.ecs.fetch::<Entity>();
                        let mut intent = self.ecs.write_storage::<DesiresDropItem>();

                        intent
                            .insert(*player, DesiresDropItem { item })
                            .expect("failed to insert drop intent");

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

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

        for (pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}
