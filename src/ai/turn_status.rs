use crate::{Confusion, MyTurn, RunState};
use specs::prelude::*;

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut confusion, entities, run_state) = data;

        if *run_state != RunState::Ticking {
            return;
        }

        let mut not_my_turn: Vec<Entity> = Vec::new();
        let mut not_confused: Vec<Entity> = Vec::new();

        for (entity, _turn, confused) in (&entities, &mut turns, &mut confusion).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
            } else {
                not_my_turn.push(entity);
            }
        }

        for entity in not_my_turn {
            turns.remove(entity);
        }

        for entity in not_confused {
            confusion.remove(entity);
        }
    }
}
