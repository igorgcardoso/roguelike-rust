use super::{BuilderMap, MetaMapBuilder, Position};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let start = rooms[0].center();
            build_data.starting_position = Some(Position {
                x: start.0,
                y: start.1,
            });
        } else {
            panic!("Room Based Starting Position only works after rooms have been created");
        }
    }
}
