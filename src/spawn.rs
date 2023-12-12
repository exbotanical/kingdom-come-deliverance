use rltk::RandomNumberGenerator;
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{
    components::{
        AreaOfEffect, BlocksCell, CombatStats, Consumable, Enemy, InflictsDamage, Item, Name,
        Player, Position, ProvidesHealing, Ranged, Renderable, SerializeOnSave, StatusEffect,
        StatusEffectType, Viewshed,
    },
    geometry::Rect,
    map::MAP_WIDTH,
};

const MAX_ENEMIES: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: rltk::RGB::named(rltk::YELLOW),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Viewshed {
            visible_cells: Vec::new(),
            range: 8,
            dirty: true,
        })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build()
}

pub fn room(ecs: &mut World, room: &Rect) {
    let enemy_spawn_points: Vec<usize>;
    let item_spawn_points: Vec<usize>;

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        let num_enemies = rng.roll_dice(1, MAX_ENEMIES + 2) - 3;
        enemy_spawn_points = generate_spawn_points(&mut rng, room, num_enemies);

        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        item_spawn_points = generate_spawn_points(&mut rng, room, num_items);
    }

    spawn_in_room(ecs, enemy_spawn_points, spawn_random_enemy);
    spawn_in_room(ecs, item_spawn_points, spawn_random_item);
}

fn generate_spawn_points(rng: &mut RandomNumberGenerator, room: &Rect, num: i32) -> Vec<usize> {
    let mut spawn_points = Vec::new();

    for _i in 0..num {
        let mut added = false;
        while !added {
            let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
            let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;

            let idx = (y * MAP_WIDTH as usize) + x;
            if !spawn_points.contains(&idx) {
                spawn_points.push(idx);
                added = true;
            }
        }
    }

    spawn_points
}

fn spawn_in_room<F>(ecs: &mut World, points: Vec<usize>, cb: F)
where
    F: Fn(&mut World, i32, i32),
{
    for idx in points.iter() {
        let x = *idx as i32 % MAP_WIDTH;
        let y = *idx as i32 / MAP_WIDTH;
        cb(ecs, x, y);
    }
}

fn spawn_random_enemy(ecs: &mut World, x: i32, y: i32) {
    let roll;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        roll = rng.roll_dice(1, 2);
    }

    match roll {
        1 => spawn_goblin(ecs, x, y),
        _ => spawn_orc(ecs, x, y),
    };
}

fn spawn_orc(ecs: &mut World, x: i32, y: i32) {
    spawn_enemy(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

fn spawn_goblin(ecs: &mut World, x: i32, y: i32) {
    spawn_enemy(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn spawn_enemy<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: rltk::RGB::named(rltk::RED),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_cells: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Enemy {})
        .with(Name {
            name: name.to_string(),
        })
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(BlocksCell {})
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();
}

fn spawn_random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }

    match roll {
        1 => spawn_health_potion(ecs, x, y),
        2 => spawn_fireball_scroll(ecs, x, y),
        3 => spawn_confusion_scroll(ecs, x, y),
        _ => spawn_missile_scroll(ecs, x, y),
    }
}

fn spawn_health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: rltk::RGB::named(rltk::MAGENTA),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();
}

fn spawn_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: rltk::RGB::named(rltk::CYAN),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magick Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();
}

fn spawn_fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: rltk::RGB::named(rltk::ORANGE),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();
}

fn spawn_confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: rltk::RGB::named(rltk::PINK),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(StatusEffect {
            effect: StatusEffectType::Confusion,
            turns: 4,
            print_as: "confusing".to_string(),
        })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();
}
