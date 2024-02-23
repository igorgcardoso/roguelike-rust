use super::{
    gamelog::GameLog, Bystander, EntityMoved, Map, Name, Point, Position, Quips, RunState, Viewshed,
};
use specs::prelude::*;

pub struct BystanderAI {}

impl<'a> System<'a> for BystanderAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Bystander>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadExpect<'a, Point>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            runstate,
            entities,
            mut viewshed,
            bystander,
            mut position,
            mut entity_moved,
            mut rng,
            player_pos,
            mut gamelog,
            mut quips,
            names,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _bystander, pos) in
            (&entities, &mut viewshed, &bystander, &mut position).join()
        {
            // Possibly quip
            let quip = quips.get_mut(entity);
            if let Some(quip) = quip {
                if !quip.available.is_empty()
                    && viewshed.visible_tiles.contains(&player_pos)
                    && rng.roll_dice(1, 6) == 1
                {
                    let name = names.get(entity);
                    let quip_index = if quip.available.len() == 1 {
                        0
                    } else {
                        (rng.roll_dice(1, quip.available.len() as i32) - 1) as usize
                    };
                    gamelog.entries.push(format!(
                        "{} says \"{}\"",
                        name.unwrap().name,
                        quip.available[quip_index]
                    ));
                    quip.available.remove(quip_index);
                }
            }

            // Try to move randomly
            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);
            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                let destination_idx = map.xy_idx(x, y);
                if !map.blocked[destination_idx] {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = x;
                    pos.y = y;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
                    map.blocked[destination_idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
