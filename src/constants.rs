pub const PHYSICS_LENGTH_UNIT: f32 = 16.0;

pub const fn times_phys_length_unit(value: i64) -> f32 {
    value as f32 * PHYSICS_LENGTH_UNIT
}

pub const fn pixels_to_world_units(value: i64) -> f32 {
    value as f32 / PHYSICS_LENGTH_UNIT
}

// Player constants
pub const PLAYER_WIDTH: f32 = times_phys_length_unit(1);
pub const PLAYER_HEIGHT: f32 = times_phys_length_unit(2);

// Player movement constants
pub const PLAYER_MOVE_SPEED: f32 = times_phys_length_unit(10);
pub const PLAYER_JUMP_FORCE: f32 = times_phys_length_unit(100);
pub const PLAYER_GRAVITY: f32 = times_phys_length_unit(100);
pub const PLAYER_MAX_FALL_SPEED: f32 = times_phys_length_unit(1000);
pub const PLAYER_JUMP_BUFFER_TIME: f32 = 0.15;
pub const PLAYER_COYOTE_TIME: f32 = 0.12;

// Collision detection constants
pub const GROUND_CHECK_HEIGHT: f32 = 2.0 / PHYSICS_LENGTH_UNIT; // 0.0625 world units
pub const GROUND_CHECK_OFFSET_Y: f32 = pixels_to_world_units(-1);
pub const GROUND_CHECK_DISTANCE: f32 = pixels_to_world_units(2);
pub const SLOW_UPWARD_THRESHOLD: f32 = 50.0 / PHYSICS_LENGTH_UNIT; // 1.5625 world units/sec

// Movement threshold for collision detection
pub const MIN_MOVEMENT_THRESHOLD: f32 = pixels_to_world_units(1);
