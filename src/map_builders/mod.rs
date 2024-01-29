mod bsp_dungeon;
mod common;
mod simple_builder;

use super::{spawner, Map, Position, Rect, TileType};
use bsp_dungeon::BspDungeonBuilder;
use common::*;
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
    let builder = rng.roll_dice(1, 2);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
