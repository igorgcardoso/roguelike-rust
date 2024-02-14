mod area_based;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod distant_exit;
mod dla;
mod drunkward;
mod maze;
mod prefab_builder;
mod room_based;
mod simple;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;

use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use area_based::{AreaStartingPosition, CullUnreachable, XStart, YStart};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use distant_exit::DistantExit;
use dla::DLABuilder;
use drunkward::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use room_based::{RoomBasedSpawner, RoomBasedStairs, RoomBasedStartingPosition};
use simple::SimpleMapBuilder;
use specs::prelude::*;
use voronoi::VoronoiCellBuilder;
use voronoi_spawning::VoronoiSpawning;
use waveform_collapse::WaveformCollapseBuilder;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
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

fn random_initial_builder(
    rng: &mut rltk::RandomNumberGenerator,
) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);
    let result: (Box<dyn InitialMapBuilder>, bool) = match builder {
        1 => (BspDungeonBuilder::new(), true),
        2 => (BspInteriorBuilder::new(), true),
        3 => (CellularAutomataBuilder::new(), false),
        4 => (DrunkardsWalkBuilder::open_area(), false),
        5 => (DrunkardsWalkBuilder::open_halls(), false),
        6 => (DrunkardsWalkBuilder::winding_passages(), false),
        7 => (DrunkardsWalkBuilder::fat_passages(), false),
        8 => (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => (MazeBuilder::new(), false),
        10 => (DLABuilder::walk_inwards(), false),
        11 => (DLABuilder::walk_outwards(), false),
        12 => (DLABuilder::central_attractor(), false),
        13 => (DLABuilder::insectoid(), false),
        14 => (VoronoiCellBuilder::pythagoras(), false),
        15 => (VoronoiCellBuilder::manhattan(), false),
        16 => (
            PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED),
            false,
        ),
        _ => (SimpleMapBuilder::new(), true),
    };
    result
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    let (random_starter, has_rooms) = random_initial_builder(rng);
    builder.start_with(random_starter);
    if has_rooms {
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStairs::new());
        builder.with(RoomBasedStartingPosition::new());
    } else {
        builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
        builder.with(CullUnreachable::new());
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
        ));
    }

    builder.with(PrefabBuilder::vaults());

    builder
}
