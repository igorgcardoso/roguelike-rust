mod area_based;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod distant_exit;
mod dla;
mod door_placement;
mod drunkward;
mod forest;
mod limestone_cavern;
mod maze;
mod prefab_builder;
mod room_based;
mod simple;
mod town;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;

use super::{spawner, Map, Position, Rect, TileType};
use area_based::{AreaStartingPosition, CullUnreachable, XStart, YStart};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use distant_exit::DistantExit;
use dla::DLABuilder;
use door_placement::DoorPlacement;
use drunkward::DrunkardsWalkBuilder;
use forest::forest_builder;
use limestone_cavern::{
    limestone_cavern_builder, limestone_deep_cavern_builder, limestone_transition_builder,
};
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use room_based::{
    BspCorridors, CorridorSpawner, DoglegCorridors, NearestCorridors, RoomBasedSpawner,
    RoomBasedStairs, RoomBasedStartingPosition, RoomCornerRounder, RoomDrawer, RoomExploder,
    RoomSort, RoomSorter, StraightLineCorridors,
};
use simple::SimpleMapBuilder;
use specs::prelude::*;
use town::town_builder;
use voronoi::VoronoiCellBuilder;
use voronoi_spawning::VoronoiSpawning;
use waveform_collapse::WaveformCollapseBuilder;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32,
}

impl BuilderMap {
    #[cfg(debug_assertions)]
    fn take_snapshot(&mut self) {
        let mut snapshot = self.map.clone();
        for v in snapshot.revealed_tiles.iter_mut() {
            *v = true;
        }
        self.history.push(snapshot);
    }

    #[cfg(not(debug_assertions))]
    fn take_snapshot(&mut self) {}
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height,
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => {
                self.starter = Some(starter);
            }
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, meta_builder: Box<dyn MetaMapBuilder>) {
        self.builders.push(meta_builder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for meta_builder in self.builders.iter_mut() {
            meta_builder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_start_position(rng: &mut rltk::RandomNumberGenerator) -> (XStart, YStart) {
    let x_roll = rng.roll_dice(1, 3);
    let x = match x_roll {
        1 => XStart::Left,
        2 => XStart::Center,
        _ => XStart::Right,
    };

    let y_roll = rng.roll_dice(1, 3);
    let y = match y_roll {
        1 => YStart::Bottom,
        2 => YStart::Center,
        _ => YStart::Top,
    };

    (x, y)
}

fn random_room_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()),
    }

    // BSP Interior still makes holes in the walls
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LeftMost)),
            2 => builder.with(RoomSorter::new(RoomSort::RightMost)),
            3 => builder.with(RoomSorter::new(RoomSort::TopMost)),
            4 => builder.with(RoomSorter::new(RoomSort::BottomMost)),
            _ => builder.with(RoomSorter::new(RoomSort::Central)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 4);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let corridor_spawn_roll = rng.roll_dice(1, 2);
        if corridor_spawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let builder_roll = rng.roll_dice(1, 16);
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiCellBuilder::pythagoras()),
        13 => builder.start_with(VoronoiCellBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(
            prefab_builder::prefab_levels::WFC_POPULATED,
        )),
    }

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Setup an exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder(
    new_depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height, "New Map");
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
        builder.with(CullUnreachable::new());

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and spawn mobs
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
        ));
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    builder
}

pub fn level_builder(
    new_depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_cavern_builder(new_depth, rng, width, height),
        4 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        5 => limestone_transition_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height),
    }
}
