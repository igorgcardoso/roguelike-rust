use super::{CombatStats, Map, Player, Position, RunState, State, Viewshed, WantsToMelee};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<super::WantsToMelee>();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 0 || pos.x + delta_x > map.width - 1 {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::A | VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::S | VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::D | VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}
