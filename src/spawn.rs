use std::collections::HashMap;

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
    random_table::RandomTable,
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

pub fn room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_ENEMIES + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;

            // Keep picking random cells until we find an empty one
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH as usize) + x;

                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    for spawn in spawn_points.iter() {
        let x = *spawn.0 as i32 % MAP_WIDTH;
        let y = *spawn.0 as i32 / MAP_WIDTH;

        match spawn.1.as_ref() {
            "Goblin" => spawn_goblin(ecs, x, y),
            "Orc" => spawn_orc(ecs, x, y),
            "Health Potion" => spawn_health_potion(ecs, x, y),
            "Fireball Scroll" => spawn_fireball_scroll(ecs, x, y),
            "Confusion Scroll" => spawn_confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => spawn_missile_scroll(ecs, x, y),
            _ => {}
        }
    }
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

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Org", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball scroll", 2 + map_depth)
        .add("Confusion scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
}
