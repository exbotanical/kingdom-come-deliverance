use specs::prelude::*;
use specs::Entity;

use crate::{
    components::{InInventory, Name},
    state::State,
};

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
