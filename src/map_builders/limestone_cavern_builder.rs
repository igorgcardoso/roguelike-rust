use super::{
    AreaStartingPosition, BuilderChain, BuilderMap, CullUnreachable, DistantExit,
    DrunkardsWalkBuilder, MetaMapBuilder, TileType, VoronoiSpawning, XStart, YStart,
};
use rltk::RandomNumberGenerator;

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    #[allow(dead_code)]
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tile_type) in build_data.map.tiles.iter_mut().enumerate() {
            // Gravel Spawning
            if *tile_type == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                *tile_type = TileType::Gravel;
            } else if *tile_type == TileType::Floor && rng.roll_dice(1, 10) == 1 {
                // Spawn passable pools
                *tile_type = TileType::ShallowWater;
            } else if *tile_type == TileType::Wall {
                // Spawn deep pools and stalactites
                let mut neighbors = 0;
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                if x > 0 && old_map.tiles[idx - 1] == TileType::Wall {
                    neighbors += 1;
                }
                if x < old_map.width - 2 && old_map.tiles[idx + 1] == TileType::Wall {
                    neighbors += 1;
                }
                if y > 0 && old_map.tiles[idx - old_map.width as usize] == TileType::Wall {
                    neighbors += 1;
                }
                if y < old_map.height - 2
                    && old_map.tiles[idx + old_map.width as usize] == TileType::Wall
                {
                    neighbors += 1;
                }
                if neighbors > 2 {
                    *tile_type = TileType::DeepWater;
                } else if neighbors == 1 {
                    let roll = rng.roll_dice(1, 4);
                    match roll {
                        1 => *tile_type = TileType::Stalactite,
                        2 => *tile_type = TileType::Stalagmite,
                        _ => {}
                    }
                }
            }
        }

        build_data.take_snapshot();
        build_data.map.outdoors = false;
    }
}
