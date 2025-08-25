use bevy::prelude::*;

use crate::constants::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct IsGrounded(pub bool);

impl Default for IsGrounded {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Component)]
pub struct KinematicVelocity {
    pub velocity: Vec2,
}

impl Default for KinematicVelocity {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct PlayerController {
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub jump_buffer_time: f32,
    pub coyote_time: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: PLAYER_MOVE_SPEED,
            jump_force: PLAYER_JUMP_FORCE,
            gravity: PLAYER_GRAVITY,
            max_fall_speed: PLAYER_MAX_FALL_SPEED,
            jump_buffer_time: PLAYER_JUMP_BUFFER_TIME,
            coyote_time: PLAYER_COYOTE_TIME,
        }
    }
}

#[derive(Component)]
pub struct JumpState {
    pub jump_buffer_timer: f32,
    pub coyote_timer: f32,
    pub was_grounded_last_frame: bool,
}

impl Default for JumpState {
    fn default() -> Self {
        Self {
            jump_buffer_timer: 0.0,
            coyote_timer: 0.0,
            was_grounded_last_frame: false,
        }
    }
}

#[derive(Component)]
pub struct CollisionInfo {
    pub vertical_distance: Option<f32>,
    pub horizontal_distance: Option<f32>,
}

impl Default for CollisionInfo {
    fn default() -> Self {
        Self {
            vertical_distance: None,
            horizontal_distance: None,
        }
    }
}
