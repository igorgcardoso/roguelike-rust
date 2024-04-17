use crate::{Attributes, Initiative, MyTurn, Position, RunState};
use specs::prelude::*;

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, rltk::Point>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut initiatives,
            positions,
            mut turns,
            entities,
            mut rng,
            attributes,
            mut runstate,
            player,
            player_pos,
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Clear any remaining MyTurn we left by mistake
        turns.clear();

        // Roll initiative
        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut my_turn = true;

                // Re-roll
                initiative.current = 6 + rng.roll_dice(1, 6);

                // Give a bonus for quickness
                if let Some(attributes) = attributes.get(entity) {
                    initiative.current -= attributes.quickness.bonus;
                }

                // TODO: More initiative granting boosts/penalties

                // If it's the player, we want to go to an AwaitingInput state
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                } else {
                    let distance = rltk::DistanceAlg::Pythagoras
                        .distance2d(*player_pos, rltk::Point::new(pos.x, pos.y));
                    if distance > 20.0 {
                        my_turn = false;
                    }
                }

                // It's my turn!
                if my_turn {
                    turns
                        .insert(entity, MyTurn {})
                        .expect("Unable to insert turn");
                }
            }
        }
    }
}
