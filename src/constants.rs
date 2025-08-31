pub const TILE_SIZE: f32 = 16.0;

pub const fn multiply_by_tile_size(value: i64) -> f32 {
    value as f32 * TILE_SIZE
}

pub const fn pixels_to_world_units(value: i64) -> f32 {
    value as f32 / TILE_SIZE
}

// Player constants
pub const PLAYER_WIDTH: f32 = multiply_by_tile_size(1);
pub const PLAYER_HEIGHT: f32 = multiply_by_tile_size(2);

include!(concat!(env!("OUT_DIR"), "/ldtk_constants.rs"));
