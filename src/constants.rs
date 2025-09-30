use avian2d::prelude::PhysicsLayer;

pub const TILE_SIZE: f32 = 16.0;

pub const fn multiply_by_tile_size(value: i64) -> f32 {
    value as f32 * TILE_SIZE
}

pub const PLAYER_WIDTH: f32 = multiply_by_tile_size(2);
pub const PLAYER_HEIGHT: f32 = multiply_by_tile_size(3);

include!(concat!(env!("OUT_DIR"), "/ldtk_constants.rs"));

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    LevelGeometry,
}
