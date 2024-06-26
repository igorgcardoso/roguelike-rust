pub mod dungeon;
mod theme;
mod tile_type;

pub use self::{
    dungeon::{freeze_level_entities, level_transition, thaw_level_entities, MasterDungeonMap},
    theme::*,
    tile_type::{get_tile_cost, is_tile_opaque, is_tile_walkable, TileType},
};
use super::*;
use rltk::{Algorithm2D, BaseMap, Point};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,
    pub name: String,
    pub outdoors: bool,
    pub light: Vec<rltk::RGB>,
}

impl Map {
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> Map {
        let map_tile_count = (width * height) as usize;
        crate::spatial::set_size(map_tile_count);
        Map {
            tiles: vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            name: name.to_string(),
            outdoors: true,
            light: vec![rltk::RGB::from_f32(0.0, 0.0, 0.0); map_tile_count],
        }
    }
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !crate::spatial::is_blocked(idx)
    }

    pub fn populate_blocked(&mut self) {
        crate::spatial::populate_blocked_from_map(self);
    }

    pub fn clear_content_index(&mut self) {
        crate::spatial::clear();
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx > 0 && idx < self.tiles.len() {
            is_tile_opaque(self.tiles[idx]) || self.view_blocked.contains(&idx)
        } else {
            true
        }
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let width_u = self.width as usize;
        let tile_type = self.tiles[idx];

        // cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, get_tile_cost(tile_type)))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, get_tile_cost(tile_type)))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - width_u, get_tile_cost(tile_type)))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + width_u, get_tile_cost(tile_type)))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - width_u) - 1, get_tile_cost(tile_type) * 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - width_u) + 1, get_tile_cost(tile_type) * 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + width_u) - 1, get_tile_cost(tile_type) * 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + width_u) + 1, get_tile_cost(tile_type) * 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
