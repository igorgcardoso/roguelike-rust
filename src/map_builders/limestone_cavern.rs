use super::{
    area_based::{AreaEndingPosition, XEnd, YEnd},
    bsp_dungeon::BspDungeonBuilder,
    cellular_automata::CellularAutomataBuilder,
    dla::DLABuilder,
    prefab_builder::PrefabBuilder,
    room_based::{
        NearestCorridors, RoomBasedSpawner, RoomDrawer, RoomExploder, RoomSort, RoomSorter,
    },
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

pub fn limestone_deep_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Deep Limestone Caverns");
    chain.start_with(DLABuilder::central_attractor());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain.with(PrefabBuilder::sectional(
        super::prefab_builder::prefab_sections::ORC_CAMP,
    ));

    chain
}

pub fn limestone_transition_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarf Fort - Upper Reaches");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(CaveDecorator::new());
    chain.with(CaveTransition::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaEndingPosition::new(XEnd::Right, YEnd::Center));
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

pub struct CaveTransition {}

impl MetaMapBuilder for CaveTransition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveTransition {
    #[allow(dead_code)]
    pub fn new() -> Box<CaveTransition> {
        Box::new(CaveTransition {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.depth = 5;
        build_data.take_snapshot();

        // Build a BSP-based dungeon
        let mut builder = BuilderChain::new(5, build_data.width, build_data.height, "New Map");
        builder.start_with(BspDungeonBuilder::new());
        builder.with(RoomDrawer::new());
        builder.with(RoomSorter::new(RoomSort::RightMost));
        builder.with(NearestCorridors::new());
        builder.with(RoomExploder::new());
        builder.with(RoomBasedSpawner::new());
        builder.build_map(rng);

        // Add the history to our history
        for history in builder.build_data.history.iter() {
            build_data.history.push(history.clone());
        }
        build_data.take_snapshot();

        // Copy the right half of the BSP map into our map
        for x in build_data.map.width / 2..build_data.map.width {
            for y in 0..build_data.map.height {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = builder.build_data.map.tiles[idx];
            }
        }
        build_data.take_snapshot();

        // Keep Voronoi spawn data from the left half of the map
        let width = build_data.map.width;
        build_data.spawn_list.retain(|spawn| {
            let x = spawn.0 as i32 / width;
            x < width / 2
        });

        // Keep room spawn data from the right half of the map
        for spawn in builder.build_data.spawn_list.iter() {
            let x = spawn.0 as i32 / width;
            if x > width / 2 {
                build_data.spawn_list.push(spawn.clone());
            }
        }
    }
}
