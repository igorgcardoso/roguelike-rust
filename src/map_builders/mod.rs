mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod drunkward;
mod simple_builder;

use self::{
    bsp_interior::BspInteriorBuilder,
    drunkward::{DrunkSpawnMode, DrunkardSettings},
};
use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use bsp_dungeon::BspDungeonBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use drunkward::DrunkardsWalkBuilder;
use simple_builder::SimpleMapBuilder;
use specs::prelude::*;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 7);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
