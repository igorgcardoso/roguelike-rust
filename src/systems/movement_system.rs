use crate::{
    ApplyMove, ApplyTeleport, EntityMoved, Map, OtherLevelPosition, Position, RunState, Viewshed,
};
use specs::prelude::*;

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Position>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        WriteStorage<'a, ApplyTeleport>,
        WriteStorage<'a, OtherLevelPosition>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Viewshed>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut positions,
            entities,
            mut apply_move,
            mut apply_teleport,
            mut other_level_position,
            mut entity_moved,
            mut viewsheds,
            player_entity,
            mut runstate,
        ) = data;

        // Apply teleports
        for (entity, teleport) in (&entities, &apply_teleport).join() {
            if teleport.destination_depth == map.depth {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            destination_idx: map
                                .xy_idx(teleport.destination_x, teleport.destination_y),
                        },
                    )
                    .expect("Unable to insert");
            } else if entity == *player_entity {
                *runstate = RunState::TeleportingToOtherLevel {
                    x: teleport.destination_x,
                    y: teleport.destination_y,
                    depth: teleport.destination_depth,
                }
            } else if let Some(pos) = positions.get(entity) {
                let idx = map.xy_idx(pos.x, pos.y);
                let destination_idx = map.xy_idx(teleport.destination_x, teleport.destination_y);
                crate::spatial::move_entity(entity, idx, destination_idx);
                other_level_position
                    .insert(
                        entity,
                        OtherLevelPosition {
                            x: teleport.destination_x,
                            y: teleport.destination_y,
                            depth: teleport.destination_depth,
                        },
                    )
                    .expect("Unable to insert");
                positions.remove(entity);
            }
        }
        apply_teleport.clear();

        // Apply broad movement
        for (entity, movement, pos) in (&entities, &apply_move, &mut positions).join() {
            let start_idx = map.xy_idx(pos.x, pos.y);
            let destination_idx = movement.destination_idx;
            crate::spatial::move_entity(entity, start_idx, destination_idx);
            pos.x = destination_idx as i32 % map.width;
            pos.y = destination_idx as i32 / map.width;
            if let Some(viewshed) = viewsheds.get_mut(entity) {
                viewshed.dirty = true;
            }
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert");
        }
        apply_move.clear();
    }
}
