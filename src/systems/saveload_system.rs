use crate::components::*;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};
use std::convert::Infallible;
use std::fs;
use std::fs::File;
use std::path::Path;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty), *) => {
        $(
            SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
                &( $ecs.read_storage::<$type>(), ),
                &$data.0,
                &$data.1,
                &mut $ser,
            )
            .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<crate::map::Map>().unwrap().clone();
    let dungeon_master = ecs
        .get_mut::<crate::map::MasterDungeonMap>()
        .unwrap()
        .clone();

    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let save_helper2 = ecs
        .create_entity()
        .with(DMSerializationHelper {
            map: dungeon_master,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually Serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Name,
            BlocksTile,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            Door,
            BlocksVisibility,
            Quips,
            Attributes,
            Skills,
            Pools,
            NaturalAttackDefense,
            LootTable,
            OtherLevelPosition,
            DMSerializationHelper,
            LightSource,
            Initiative,
            MyTurn,
            Faction,
            WantsToApproach,
            WantsToFlee,
            MoveMode,
            Chasing,
            EquipmentChanged,
            Vendor,
            TownPortal,
            TeleportTo,
            ApplyMove,
            ApplyTeleport,
            MagicItem,
            ObfuscatedName,
            IdentifiedItem
        );
    }

    // Cleanup
    ecs.delete_entity(save_helper).expect("Crash on cleanup");
    ecs.delete_entity(save_helper2).expect("Crash on cleanup");
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty), *) => {
        $(
            DeserializeComponents::<Infallible, _>::deserialize(
                &mut ( &mut $ecs.write_storage::<$type>(), ),
                &$data.0, // entities
                &mut $data.1, // marker
                &mut $data.2, // allocator
                &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Name,
            BlocksTile,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            Door,
            BlocksVisibility,
            Quips,
            Attributes,
            Skills,
            Pools,
            NaturalAttackDefense,
            LootTable,
            OtherLevelPosition,
            DMSerializationHelper,
            LightSource,
            Initiative,
            MyTurn,
            Faction,
            WantsToApproach,
            WantsToFlee,
            MoveMode,
            Chasing,
            EquipmentChanged,
            Vendor,
            TownPortal,
            TeleportTo,
            ApplyMove,
            ApplyTeleport,
            MagicItem,
            ObfuscatedName,
            IdentifiedItem
        );
    }

    let mut deleteme: Option<Entity> = None;
    let mut deleteme2: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let helper2 = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<crate::map::Map>();
            *worldmap = h.map.clone();
            crate::spatial::set_size((worldmap.height * worldmap.width) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &helper2).join() {
            let mut dungeon_master = ecs.write_resource::<crate::map::MasterDungeonMap>();
            *dungeon_master = h.map.clone();
            deleteme2 = Some(e);
        }

        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut playerpos = ecs.write_resource::<rltk::Point>();
            *playerpos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
    ecs.delete_entity(deleteme2.unwrap())
        .expect("Unable to delete helper");
}

pub fn delete_save() {
    if does_save_exist() {
        fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}
