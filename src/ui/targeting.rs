use specs::prelude::*;
use specs::Entity;

use crate::{components::Viewshed, state::State};

use super::inventory::ItemMenuResult;

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut rltk::Rltk,
    range: i32,
) -> (ItemMenuResult, Option<rltk::Point>) {
    let player = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<rltk::Point>();
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
    let maybe_visible = viewsheds.get(*player);

    if let Some(visible) = maybe_visible {
        // For each visible cell...
        for idx in visible.visible_cells.iter() {
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
                Some(rltk::Point::new(mouse_pos.0, mouse_pos.1)),
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
