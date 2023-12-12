use specs::prelude::*;
use std::fs::{self, File};
use std::path::Path;

use specs::error::NoError;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};

use crate::{components::*, map};
use crate::{
    components::{SerializationHelper, SerializeOnSave},
    map::Map,
};

const SAVE_FILE_PATH: &str = "./savegame.json";

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeOnSave>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn has_save_file() -> bool {
    Path::new(SAVE_FILE_PATH).exists()
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    let map_cp = ecs.get_mut::<Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_cp })
        .marked::<SimpleMarker<SerializeOnSave>>()
        .build();

    {
        let writer = File::create(SAVE_FILE_PATH).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);

        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeOnSave>>(),
        );

        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Enemy,
            Name,
            BlocksCell,
            CombatStats,
            Damage,
            DesiresMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            StatusEffect,
            ProvidesHealing,
            InInventory,
            DesiresAcquireItem,
            DesiresUseItem,
            DesiresDropItem,
            SerializationHelper
        );
    }

    ecs.delete_entity(save_helper)
        .expect("crash on save cleanup");
    // let d = serde_json::to_string(&*ecs.fetch::<Map>()).unwrap();
    // println!("{}", d);
}

pub fn load_game(ecs: &mut World) {
    {
        let mut to_delete = Vec::new();

        // Add every entity in the game into a delete list (we'll iterate deletes separately to avoid invalidating the iterator)
        for entity in ecs.entities().join() {
            to_delete.push(entity);
        }

        // Delete every entity
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("delete entity failed");
        }
    }

    // Open the save file and deserialize it
    let data = fs::read_to_string(SAVE_FILE_PATH).unwrap();
    let mut deserialized = serde_json::Deserializer::from_str(&data);

    {
        // Deserialize every entity
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeOnSave>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeOnSave>>(),
        );

        deserialize_individually!(
            ecs,
            deserialized,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Enemy,
            Name,
            BlocksCell,
            CombatStats,
            Damage,
            DesiresMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            StatusEffect,
            ProvidesHealing,
            InInventory,
            DesiresAcquireItem,
            DesiresUseItem,
            DesiresDropItem,
            SerializationHelper
        );
    }

    let mut to_delete: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helpers = ecs.read_storage::<SerializationHelper>();
        let players = ecs.read_storage::<Player>();
        let positions = ecs.read_storage::<Position>();

        // Iterate all entities and build the map
        for (entity, helper) in (&entities, &helpers).join() {
            let mut map = ecs.write_resource::<Map>();
            *map = helper.map.clone();
            map.cell_content = vec![Vec::new(); map::MAP_COUNT];
            to_delete = Some(entity);
        }

        // Set the player and player position
        for (entity, _player, pos) in (&entities, &players, &positions).join() {
            let mut player_pos = ecs.write_resource::<rltk::Point>();
            *player_pos = rltk::Point::new(pos.x, pos.y);

            let mut player = ecs.write_resource::<Entity>();
            *player = entity;
        }
    }

    // Delete the helper so we don't have duplicates when next saving the game
    ecs.delete_entity(to_delete.unwrap())
        .expect("unable to delete helper");
}

pub fn delete_save() {
    if Path::new(SAVE_FILE_PATH).exists() {
        fs::remove_file(SAVE_FILE_PATH).expect("unable to delete save file");
    }
}
