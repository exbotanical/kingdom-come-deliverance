use rltk::Point;
use rltk::{self};
use specs::prelude::*;
use specs::World;

use crate::components::DesiresUseItem;
use crate::components::InInventory;
use crate::components::Player;
use crate::components::Position;
use crate::components::Ranged;
use crate::components::Renderable;
use crate::components::{DesiresDropItem, Viewshed};
use crate::log::GameLog;
use crate::map::draw_map;
use crate::map::Map;
use crate::player::player_input;
use crate::save;
use crate::spawn::{self};
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
    // Intermediate init state
    PreRun,
    // Awaiting player input to start next turn
    AwaitingInput,
    // Player's turn to do something
    PlayerTurn,
    // Monster's turn to do something
    MonsterTurn,
    // Displaying player inventory
    ShowInventory,
    // Displaying player drop menu
    ShowDropItem,
    // Ranged item targeting UI
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    // Game main menu
    MainMenu {
        menu_selection: ui::MainMenuSelection,
    },
    // Save state selected
    SaveGame,
    // Player moving across map depths
    NextLevel,
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

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let players = self.ecs.read_storage::<Player>();
        let inventory = self.ecs.read_storage::<InInventory>();
        let player = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();

        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = players.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            let item = inventory.get(entity);
            if let Some(item) = item {
                if item.owner == *player {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or player's inventory
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("unable to delete entity");
        }

        // Build new map and place player
        let map;
        {
            let mut existing_map = self.ecs.write_resource::<Map>();
            let current_depth = existing_map.depth;

            *existing_map = Map::generate_map_rooms_and_tunnels(current_depth + 1);
            map = existing_map.clone();
        }

        // Spawn enemies
        for room in map.rooms.iter().skip(1) {
            spawn::room(&mut self.ecs, room);
        }

        let (player_x, player_y) = map.rooms[0].center();
        let mut player_pos = self.ecs.write_resource::<rltk::Point>();

        *player_pos = Point::new(player_x, player_y);

        let mut positions = self.ecs.write_storage::<Position>();
        let player = self.ecs.fetch::<Entity>();

        let player_pos_comp = positions.get_mut(*player);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark player's position as dirty
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let vs = viewsheds.get_mut(*player);
        if let Some(vs) = vs {
            vs.dirty = true;
        }

        let mut log = self.ecs.fetch_mut::<GameLog>();
        log.entries
            .push("You descend to the next level".to_string());
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

            RunState::NextLevel => {
                self.goto_next_level();
                run_state = RunState::PreRun;
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = run_state;
        }

        damage::reap(&mut self.ecs);
    }
}
