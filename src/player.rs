use crate::{raws::Reaction, Attributes};

use super::{
    gamelog::GameLog, BlocksTile, BlocksVisibility, Door, EntityMoved, Faction, HungerClock,
    HungerState, Item, Map, Player, Pools, Position, Renderable, RunState, State, TileType, Vendor,
    Viewshed, WantsToMelee, WantsToPickupItem,
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Attributes>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<super::WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut block_movement = ecs.write_storage::<BlocksTile>();
    let mut rendarables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let vendors = ecs.read_storage::<Vendor>();

    let mut result = RunState::AwaitingInput;

    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 0 || pos.x + delta_x > map.width - 1 {
            return RunState::AwaitingInput;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_gamemode(
            destination_idx,
            |potential_target| {
                if let Some(_vendor) = vendors.get(potential_target) {
                    return Some(RunState::ShowVendor {
                        vendor: potential_target,
                        mode: crate::VendorMode::Sell,
                    });
                }
                let mut hostile = true;
                if combat_stats.get(potential_target).is_some() {
                    if let Some(faction) = factions.get(potential_target) {
                        let reaction = crate::raws::faction_reaction(
                            &faction.name,
                            "Player",
                            &crate::raws::RAWS.lock().unwrap(),
                        );
                        if reaction != Reaction::Attack {
                            hostile = false;
                        }
                    }
                }
                if !hostile {
                    swap_entities.push((potential_target, pos.x, pos.y));

                    // Move the player
                    pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                    pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
                    return Some(RunState::Ticking);
                } else {
                    let target = combat_stats.get(potential_target);
                    if let Some(_target) = target {
                        wants_to_melee
                            .insert(
                                entity,
                                WantsToMelee {
                                    target: potential_target,
                                },
                            )
                            .expect("Add target failed");
                        return Some(RunState::Ticking);
                    }
                }
                let door = doors.get_mut(potential_target);
                if let Some(door) = door {
                    door.open = true;
                    blocks_visibility.remove(potential_target);
                    block_movement.remove(potential_target);
                    let glyph = rendarables.get_mut(potential_target).unwrap();
                    glyph.glyph = rltk::to_cp437('/');
                    viewshed.dirty = true;
                    return Some(RunState::Ticking);
                }
                None
            },
        );
        if !crate::spatial::is_blocked(destination_idx) {
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
            let new_idx = map.xy_idx(pos.x, pos.y);
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");
            crate::spatial::move_entity(entity, old_idx, new_idx);

            viewshed.dirty = true;
            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;
            result = RunState::Ticking;
            match map.tiles[destination_idx] {
                TileType::DownStairs => {
                    result = RunState::NextLevel;
                }
                TileType::UpStairs => {
                    result = RunState::PreviousLevel;
                }
                _ => {}
            }
        }
    }

    for swap in swap_entities.iter() {
        let their_pos = positions.get_mut(swap.0);
        if let Some(their_pos) = their_pos {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = swap.1;
            their_pos.y = swap.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            crate::spatial::move_entity(swap.0, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }

    result
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut log = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => log
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut log = ecs.fetch_mut::<GameLog>();
        log.entries
            .push("There is no way down from here.".to_string());
        false
    }
}
pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStairs {
        true
    } else {
        let mut log = ecs.fetch_mut::<GameLog>();
        log.entries
            .push("There is no way up from here.".to_string());
        false
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Hotkeys
    if ctx.shift && ctx.key.is_some() {
        let key: Option<i32> = match ctx.key.unwrap() {
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None,
        };
        if let Some(key) = key {
            return use_consumable_hotkey(gs, key - 1);
        }
    }
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::W | VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
                return try_move_player(0, -1, &mut gs.ecs)
            }
            VirtualKeyCode::Q | VirtualKeyCode::Numpad7 => {
                return try_move_player(-1, -1, &mut gs.ecs)
            }
            VirtualKeyCode::A | VirtualKeyCode::Left | VirtualKeyCode::Numpad4 => {
                return try_move_player(-1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::E | VirtualKeyCode::Numpad9 => {
                return try_move_player(1, -1, &mut gs.ecs)
            }
            VirtualKeyCode::S | VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
                return try_move_player(0, 1, &mut gs.ecs)
            }
            VirtualKeyCode::Z | VirtualKeyCode::Numpad1 => {
                return try_move_player(-1, 1, &mut gs.ecs)
            }
            VirtualKeyCode::D | VirtualKeyCode::Right | VirtualKeyCode::Numpad6 => {
                return try_move_player(1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::C | VirtualKeyCode::Numpad3 => {
                return try_move_player(1, 1, &mut gs.ecs)
            }
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::B => return RunState::ShowInventory,
            VirtualKeyCode::V => return RunState::ShowDropItem,
            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,
            // Cheating
            VirtualKeyCode::Backslash => return RunState::ShowCheatMenu,
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            _ => return RunState::AwaitingInput,
        },
    }
    RunState::Ticking
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        crate::spatial::for_each_tile_content(idx, |entity_id| {
            let faction = factions.get(entity_id);
            match faction {
                None => {}
                Some(faction) => {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap(),
                    );
                    if reaction == Reaction::Attack {
                        can_heal = false;
                    }
                }
            }
        })
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hc = hunger_clocks.get(*player_entity);
    if let Some(hc) = hc {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        pools.hit_points.current = pools.hit_points.max.min(pools.hit_points.current + 1);
    }

    RunState::Ticking
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack, WantsToUseItem};

    let consumable = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();

    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumable).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        use crate::components::Ranged;
        if let Some(ranged) = gs
            .ecs
            .read_storage::<Ranged>()
            .get(carried_consumables[key as usize])
        {
            return RunState::ShowTargeting {
                range: ranged.range,
                item: carried_consumables[key as usize],
            };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                *player_entity,
                WantsToUseItem {
                    item: carried_consumables[key as usize],
                    target: None,
                },
            )
            .expect("Unable to insert intent");
    }
    RunState::Ticking
}
