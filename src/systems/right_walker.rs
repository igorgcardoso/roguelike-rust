use super::*;

pub struct RightWalker {}

impl<'a> System<'a> for RightWalker {
    type SystemData = (ReadStorage<'a, RightMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (righty, mut pos): Self::SystemData) {
        for (_righty, pos) in (&righty, &mut pos).join() {
            pos.x += 1;
            if pos.x > 79 {
                pos.x = 0;
            }
        }
    }
}
