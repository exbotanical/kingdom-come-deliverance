use specs::prelude::*;
use specs::World;

use crate::{
    components::{Name, Position},
    map::Map,
};

pub fn draw_tooltips(ecs: &World, ctx: &mut rltk::Rltk) {
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

        if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 && map.visible_cells[idx] {
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
                let arrow_pos = rltk::Point::new(mouse_pos.0 - 2, mouse_pos.1);
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
                let arrow_pos = rltk::Point::new(mouse_pos.0 + 1, mouse_pos.1);
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
