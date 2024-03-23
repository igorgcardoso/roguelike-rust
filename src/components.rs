use std::collections::HashMap;

use rltk::RGB;
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use specs::error::NoError;
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::*;

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Monster {}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BlocksTile {}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct SufferDamage {
    pub amount: Vec<(i32, bool)>,
}

impl SufferDamage {
    pub fn new_damage(
        store: &mut WriteStorage<SufferDamage>,
        victim: Entity,
        amount: i32,
        from_player: bool,
    ) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push((amount, from_player));
        } else {
            let dmg = SufferDamage {
                amount: vec![(amount, from_player)],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Item {}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Consumable {}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: super::map::MasterDungeonMap,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Head,
    Torso,
    Legs,
    Feet,
    Hands,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Might,
    Quickness,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Bystander {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Vendor {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill {
    Melee,
    Defense,
    Magic,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct NaturalAttackDefense {
    pub armor_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub table: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Carnivore {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Herbivore {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub color: RGB,
    pub range: i32,
}
