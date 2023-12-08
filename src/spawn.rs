use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{
    components::{
        BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Renderable,
        Viewshed,
    },
    geometry::Rect,
    map::MAP_WIDTH,
};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: rltk::RGB::named(rltk::YELLOW),
            bg: rltk::RGB::named(rltk::BLACK),
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
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build()
}

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        roll = rng.roll_dice(1, 2);
    }

    match roll {
        1 => goblin(ecs, x, y),
        _ => orc(ecs, x, y),
    };
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: rltk::RGB::named(rltk::RED),
            bg: rltk::RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(BlocksTile {})
        .build();
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
        random_monster(ecs, x, y);
    }
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let monster_spawn_points: Vec<usize>;
    let item_spawn_points: Vec<usize>;

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        println!("num_monsters {} ", num_monsters);
        monster_spawn_points = generate_spawn_points(&mut rng, room, num_monsters);

        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        item_spawn_points = generate_spawn_points(&mut rng, room, num_items);
    }

    spawn_in_room(ecs, monster_spawn_points, random_monster);
    spawn_in_room(ecs, item_spawn_points, health_potion);
}

pub fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: rltk::RGB::named(rltk::MAGENTA),
            bg: rltk::RGB::named(rltk::BLACK),
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Potion { heal_amt: 8 })
        .build();
}
