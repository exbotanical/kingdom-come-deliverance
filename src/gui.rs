use rltk::Point;
use specs::prelude::*;

use crate::{
    components::{CombatStats, InInventory, Name, Player, Position, Viewshed},
    log,
    map::Map,
    state::{RunState, State},
};

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn draw_ui(ecs: &World, ctx: &mut rltk::Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP: {}/{}", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            rltk::RGB::named(rltk::YELLOW),
            rltk::RGB::named(rltk::BLACK),
            &health,
        );
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            rltk::RGB::named(rltk::RED),
            rltk::RGB::named(rltk::BLACK),
        );

        let log = ecs.fetch::<log::GameLog>();

        let mut y = 44;
        for s in log.entries.iter().rev() {
            if y < 49 {
                ctx.print(2, y, s);
            }
            y += 1;
        }
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, rltk::RGB::named(rltk::MAGENTA));

    draw_tooltips(&ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut rltk::Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 > map.width || mouse_pos.1 > map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, pos) in (&names, &positions).join() {
        let idx = map.xy_idx(pos.x, pos.y);

        if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
            width += 3;

            if mouse_pos.0 > 40 {
                let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
                let left_x = mouse_pos.0 - width;
                let mut y = mouse_pos.1;

                for s in tooltip.iter() {
                    ctx.print_color(
                        left_x,
                        y,
                        rltk::RGB::named(rltk::WHITE),
                        rltk::RGB::named(rltk::GRAY),
                        s,
                    );

                    let padding = (width - s.len() as i32) - 1;
                    for i in 0..padding {
                        ctx.print_color(
                            arrow_pos.x - i,
                            y,
                            rltk::RGB::named(rltk::WHITE),
                            rltk::RGB::named(rltk::GREY),
                            &" ".to_string(),
                        );
                    }
                    y += 1;
                }
                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    rltk::RGB::named(rltk::WHITE),
                    rltk::RGB::named(rltk::GREY),
                    &"->".to_string(),
                );
            } else {
                let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
                let left_x = mouse_pos.0 + 3;
                let mut y = mouse_pos.1;

                for s in tooltip.iter() {
                    ctx.print_color(
                        left_x + 1,
                        y,
                        rltk::RGB::named(rltk::WHITE),
                        rltk::RGB::named(rltk::GREY),
                        s,
                    );

                    let padding = (width - s.len() as i32) - 1;
                    for i in 0..padding {
                        ctx.print_color(
                            arrow_pos.x + 1 + i,
                            y,
                            rltk::RGB::named(rltk::WHITE),
                            rltk::RGB::named(rltk::GREY),
                            &" ".to_string(),
                        );
                    }
                    y += 1;
                }

                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    rltk::RGB::named(rltk::WHITE),
                    rltk::RGB::named(rltk::GREY),
                    &"<-".to_string(),
                );
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut rltk::Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let inventory_list = gs.ecs.read_storage::<InInventory>();
    let entities = gs.ecs.entities();

    let inventory = (&inventory_list, &names)
        .join()
        .filter(|item| item.0.owner == *player);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();

    let mut j = 0;
    for (entity, _inventory, name) in (&entities, &inventory_list, &names)
        .join()
        .filter(|item| item.1.owner == *player)
    {
        ctx.set(
            17,
            y,
            rltk::RGB::named(rltk::WHITE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            rltk::RGB::named(rltk::YELLOW),
            rltk::RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            rltk::RGB::named(rltk::WHITE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        y += 1;
        j += 1;

        equippable.push(entity);
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            rltk::VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut rltk::Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let inventory_list = gs.ecs.read_storage::<InInventory>();
    let entities = gs.ecs.entities();

    let inventory = (&inventory_list, &names)
        .join()
        .filter(|item| item.0.owner == *player);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _inventory, name) in (&entities, &inventory_list, &names)
        .join()
        .filter(|item| item.1.owner == *player)
    {
        ctx.set(
            17,
            y,
            rltk::RGB::named(rltk::WHITE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            rltk::RGB::named(rltk::YELLOW),
            rltk::RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            rltk::RGB::named(rltk::WHITE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            rltk::VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut rltk::Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player);

    if let Some(visible) = visible {
        // For each visible cell...
        for idx in visible.visible_tiles.iter() {
            // Calculate distance between player and said cell
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            // If the distance is within the item range...
            if distance <= range as f32 {
                // Highlight
                ctx.set_bg(idx.x, idx.y, rltk::RGB::named(rltk::BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;

    // For each cell in range...
    for idx in available_cells.iter() {
        // If the mouse is hovering on that cell, it's targeted
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, rltk::RGB::named(rltk::CYAN));
        // If clicking on the targeted, valid cell return the coords
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_pos.0, mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, rltk::RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(gs: &mut State, ctx: &mut rltk::Rltk) -> MainMenuResult {
    let run_state = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(
        15,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        "Daddle-dat Bodan",
    );

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *run_state
    {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(
                24,
                rltk::RGB::named(rltk::MAGENTA),
                rltk::RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        } else {
            ctx.print_color_centered(
                24,
                rltk::RGB::named(rltk::WHITE),
                rltk::RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        }

        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(
                25,
                rltk::RGB::named(rltk::MAGENTA),
                rltk::RGB::named(rltk::BLACK),
                "Load Game",
            );
        } else {
            ctx.print_color_centered(
                25,
                rltk::RGB::named(rltk::WHITE),
                rltk::RGB::named(rltk::BLACK),
                "Load Game",
            );
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(
                26,
                rltk::RGB::named(rltk::MAGENTA),
                rltk::RGB::named(rltk::BLACK),
                "Quit",
            );
        } else {
            ctx.print_color_centered(
                26,
                rltk::RGB::named(rltk::WHITE),
                rltk::RGB::named(rltk::BLACK),
                "Quit",
            );
        }

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                rltk::VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                rltk::VirtualKeyCode::Up => {
                    let next_selection;
                    match selection {
                        MainMenuSelection::NewGame => next_selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => next_selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => next_selection = MainMenuSelection::LoadGame,
                    }
                    return MainMenuResult::NoSelection {
                        selected: next_selection,
                    };
                }
                rltk::VirtualKeyCode::Down => {
                    let next_selection;
                    match selection {
                        MainMenuSelection::NewGame => next_selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => next_selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => next_selection = MainMenuSelection::NewGame,
                    }
                    return MainMenuResult::NoSelection {
                        selected: next_selection,
                    };
                }
                rltk::VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
