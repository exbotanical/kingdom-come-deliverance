use rltk::Point;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_cells: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGB,
    pub bg: rltk::RGB,
    pub render_order: i32,
}

#[derive(Component)]
pub struct BlocksCell {}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone)]
pub struct DesiresMelee {
    pub target: Entity,
}

#[derive(Component, Debug)]
pub struct Damage {
    pub amount: Vec<i32>,
}

impl Damage {
    pub fn new_damage(store: &mut WriteStorage<Damage>, victim: Entity, amount: i32) {
        if let Some(damage) = store.get_mut(victim) {
            damage.amount.push(amount);
        } else {
            let damage = Damage {
                amount: vec![amount],
            };
            store
                .insert(victim, damage)
                .expect("unable to insert damage");
        }
    }
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug, Clone)]
pub struct InInventory {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct DesiresAcquireItem {
    pub acquired_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug)]
pub struct DesiresUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug, Clone)]
pub struct DesiresDropItem {
    pub item: Entity,
}

/// Indicates an item can be used (at which point it is destroyed)
#[derive(Component, Debug)]
pub struct Consumable {}

#[derive(Component, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusEffectType {
    Confusion,
}

#[derive(Component, Debug, Clone)]
pub struct StatusEffect {
    pub effect: StatusEffectType,
    pub print_as: String,
    pub turns: i32,
}
