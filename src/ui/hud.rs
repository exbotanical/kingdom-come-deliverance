use specs::prelude::*;

use crate::{
    components::{CombatStats, Player},
    log,
    map::Map,
    ui::tooltip::draw_tooltips,
};

pub fn draw_ui(ecs: &World, ctx: &mut rltk::Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        rltk::RGB::named(rltk::WHITE),
        rltk::RGB::named(rltk::BLACK),
    );

    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(
        2,
        43,
        rltk::RGB::named(rltk::YELLOW),
        rltk::RGB::named(rltk::BLACK),
        &depth,
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
