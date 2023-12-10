use rltk::{self, Point};
use specs::prelude::*;
use whatever::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Consumable, Damage, DesiresAcquireItem,
        DesiresDropItem, DesiresMelee, DesiresUseItem, InInventory, InflictsDamage, Item, Monster,
        Name, Player, Position, ProvidesHealing, Ranged, Renderable, StatusEffect, Viewshed,
    },
    log,
    map::Map,
    spawn,
    state::{RunState, State},
};

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut ctx = RltkBuilder::simple80x50()
        .with_title("whatever game")
        .build()?;

    ctx.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<DesiresMelee>();
    gs.ecs.register::<Damage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InInventory>();
    gs.ecs.register::<DesiresAcquireItem>();
    gs.ecs.register::<DesiresUseItem>();
    gs.ecs.register::<DesiresDropItem>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<StatusEffect>();

    let map = Map::generate_map_rooms_and_tunnels();

    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawn::spawn_room(&mut gs.ecs, room);
    }

    let player_entity = spawn::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(log::GameLog {
        entries: vec!["lil boo mane - lil booney".to_string()],
    });

    rltk::main_loop(ctx, gs)
}
