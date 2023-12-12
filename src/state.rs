use rltk::{self};
use specs::prelude::*;
use specs::World;

use crate::components::DesiresDropItem;
use crate::components::DesiresUseItem;
use crate::components::Position;
use crate::components::Ranged;
use crate::components::Renderable;
use crate::map::draw_map;
use crate::map::Map;
use crate::player::player_input;
use crate::save;
use crate::systems::damage;
use crate::systems::inventory::ItemAcquisitionSystem;
use crate::systems::inventory::ItemDropSystem;
use crate::systems::inventory::ItemUseSystem;
use crate::systems::DamageSystem;
use crate::systems::MapIndexingSystem;
use crate::systems::MeleeCombatSystem;
use crate::systems::MonsterAISystem;
use crate::systems::VisibilitySystem;
use crate::ui;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: ui::MainMenuSelection,
    },
    SaveGame,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis_system = VisibilitySystem {};
        vis_system.run_now(&self.ecs);

        let mut monster_ai_system = MonsterAISystem {};
        monster_ai_system.run_now(&self.ecs);

        let mut map_idx_system = MapIndexingSystem {};
        map_idx_system.run_now(&self.ecs);

        let mut melee_system = MeleeCombatSystem {};
        melee_system.run_now(&self.ecs);

        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);

        let mut item_acquisition_system = ItemAcquisitionSystem {};
        item_acquisition_system.run_now(&self.ecs);

        let mut item_use_system = ItemUseSystem {};
        item_use_system.run_now(&self.ecs);

        let mut item_drop_system = ItemDropSystem {};
        item_drop_system.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut rltk::Rltk) {
        let mut run_state = *self.ecs.fetch::<RunState>();

        ctx.cls();

        match run_state {
            RunState::MainMenu { .. } => {}

            _ => {
                draw_map(&self.ecs, ctx);

                let map = self.ecs.fetch::<Map>();
                let positions = self.ecs.read_storage::<Position>();
                let renderables = self.ecs.read_storage::<Renderable>();

                let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

                for (pos, render) in data.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    if map.visible_cells[idx] {
                        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                    }
                }

                ui::draw_ui(&self.ecs, ctx);
            }
        }

        match run_state {
            RunState::PreRun => {
                self.run_systems();
                // cleanup delete items during systems run
                self.ecs.maintain();

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
                let result = ui::show_inventory(self, ctx);
                match result.0 {
                    ui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    ui::ItemMenuResult::NoResponse => {}
                    ui::ItemMenuResult::Selected => {
                        let selected_item = result.1.unwrap();
                        let ranged = self.ecs.read_storage::<Ranged>();
                        let maybe_ranged_item = ranged.get(selected_item);

                        if let Some(ranged_item) = maybe_ranged_item {
                            run_state = RunState::ShowTargeting {
                                range: ranged_item.range,
                                item: selected_item,
                            }
                        } else {
                            let entity = self.ecs.fetch::<Entity>();
                            let mut intent = self.ecs.write_storage::<DesiresUseItem>();

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
            }

            RunState::ShowDropItem => {
                let result = ui::drop_item_menu(self, ctx);
                match result.0 {
                    ui::ItemMenuResult::NoResponse => {}
                    ui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    ui::ItemMenuResult::Selected => {
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

            RunState::ShowTargeting { range, item } => {
                let target = ui::ranged_target(self, ctx, range);

                match target.0 {
                    ui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    ui::ItemMenuResult::NoResponse => {}
                    ui::ItemMenuResult::Selected => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut intent = self.ecs.write_storage::<DesiresUseItem>();

                        intent
                            .insert(
                                *player,
                                DesiresUseItem {
                                    item,
                                    target: target.1,
                                },
                            )
                            .expect("unable to insert intent");

                        run_state = RunState::PlayerTurn;
                    }
                }
            }

            RunState::MainMenu { .. } => {
                let result = ui::main_menu(self, ctx);

                match result {
                    ui::MainMenuResult::NoSelection { selected } => {
                        run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }

                    ui::MainMenuResult::Selected { selected } => match selected {
                        ui::MainMenuSelection::NewGame => run_state = RunState::PreRun,
                        ui::MainMenuSelection::LoadGame => {
                            save::load_game(&mut self.ecs);
                            run_state = RunState::PreRun;
                            // TODO: if we want perma-death
                            // save::delete_save();
                        }
                        ui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }

            RunState::SaveGame => {
                save::save_game(&mut self.ecs);

                run_state = RunState::MainMenu {
                    menu_selection: ui::MainMenuSelection::LoadGame,
                }
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = run_state;
        }

        damage::reap(&mut self.ecs);
    }
}
