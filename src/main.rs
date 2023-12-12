use rltk::{self, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use whatever::components::SerializationHelper;
use whatever::{
    components::{
        AreaOfEffect, BlocksCell, CombatStats, Consumable, Damage, DesiresAcquireItem,
        DesiresDropItem, DesiresMelee, DesiresUseItem, InInventory, InflictsDamage, Item, Monster,
        Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeOnSave, StatusEffect,
        Viewshed,
    },
    log,
    map::Map,
    spawn,
    state::{RunState, State},
    ui,
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
    gs.ecs.register::<BlocksCell>();
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
    gs.ecs.register::<SimpleMarker<SerializeOnSave>>();
    gs.ecs.register::<SerializationHelper>();

    let map = Map::generate_map_rooms_and_tunnels(1);

    let (x, y) = map.rooms[0].center();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs
        .insert(SimpleMarkerAllocator::<SerializeOnSave>::new());

    // Skip room player spawned in
    for room in map.rooms.iter().skip(1) {
        spawn::room(&mut gs.ecs, room);
    }

    let player_entity = spawn::player(&mut gs.ecs, x, y);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(x, y));
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: ui::MainMenuSelection::NewGame,
    });
    gs.ecs.insert(log::GameLog {
        entries: vec!["lil boo mane - lil booney".to_string()],
    });

    rltk::main_loop(ctx, gs)
}
