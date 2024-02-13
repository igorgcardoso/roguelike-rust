mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkward;
mod maze;
mod prefab_builder;
mod simple;
mod voronoi;
mod waveform_collapse;

use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use dla::DLABuilder;
use drunkward::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use simple::SimpleMapBuilder;
use specs::prelude::*;
use voronoi::VoronoiCellBuilder;
use waveform_collapse::WaveformCollapseBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    /*let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 16);
    let mut result: Box<dyn MapBuilder>;
    match builder {
        1 => result = Box::new(BspDungeonBuilder::new(new_depth)),
        2 => result = Box::new(BspInteriorBuilder::new(new_depth)),
        3 => result = Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        7 => result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        8 => result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        9 => result = Box::new(MazeBuilder::new(new_depth)),
        10 => result = Box::new(DLABuilder::walk_inwards(new_depth)),
        11 => result = Box::new(DLABuilder::walk_outwards(new_depth)),
        12 => result = Box::new(DLABuilder::central_attractor(new_depth)),
        13 => result = Box::new(DLABuilder::insectoid(new_depth)),
        14 => result = Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
        15 => result = Box::new(VoronoiCellBuilder::manhattan(new_depth)),
        _ => result = Box::new(SimpleMapBuilder::new(new_depth)),
    }

    if rng.roll_dice(1, 3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result))
    }

    result*/
    Box::new(PrefabBuilder::new(
        new_depth,
        Some(Box::new(CellularAutomataBuilder::new(new_depth))),
    ))
}
