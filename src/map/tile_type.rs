use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
}

pub fn is_tile_walkable(tile_type: TileType) -> bool {
    matches!(
        tile_type,
        TileType::Floor
            | TileType::DownStairs
            | TileType::Road
            | TileType::Grass
            | TileType::ShallowWater
            | TileType::WoodFloor
            | TileType::Bridge
            | TileType::Gravel
    )
}

pub fn is_tile_opaque(tile_type: TileType) -> bool {
    matches!(tile_type, TileType::Wall)
}

pub fn get_tile_cost(tile_type: TileType) -> f32 {
    match tile_type {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
