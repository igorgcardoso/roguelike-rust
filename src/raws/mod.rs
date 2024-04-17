mod faction_structs;
mod item_structs;
mod loot_structs;
mod mob_structs;
mod prop_structs;
mod raw_master;
mod spawn_table_structs;

use faction_structs::FactionInfo;
pub use faction_structs::Reaction;
use item_structs::*;
use loot_structs::*;
use mob_structs::*;
use prop_structs::*;
pub use raw_master::*;
use serde::Deserialize;
use spawn_table_structs::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTable>,
    pub faction_table: Vec<FactionInfo>,
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();

    let raw_string =
        std::str::from_utf8(raw_data).expect("Unable to convert to a valid UTF-8 string");

    let decoder: Raws = serde_json::from_str(raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
}
