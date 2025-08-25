use avian2d::prelude::*;
use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub kinematic_velocity: KinematicVelocity,
    pub locked_axes: LockedAxes,
    pub player: Player,
    pub controller: PlayerController,
    pub is_grounded: IsGrounded,
    pub jump_state: JumpState,
    pub collision_info: CollisionInfo,
    pub game_entity: GameEntity,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.7, 0.3),
                custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, times_phys_length_unit(5), 0.0),
            rigid_body: RigidBody::Kinematic,
            collider: Collider::rectangle(PLAYER_WIDTH, PLAYER_HEIGHT),
            kinematic_velocity: KinematicVelocity::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            player: Player,
            controller: PlayerController::default(),
            is_grounded: IsGrounded::default(),
            jump_state: JumpState::default(),
            collision_info: CollisionInfo::default(),
            game_entity: GameEntity,
        }
    }
}

#[derive(Bundle)]
pub struct PlatformBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub platform: Platform,
    pub game_entity: GameEntity,
}

impl PlatformBundle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(size.x, size.y),
            platform: Platform,
            game_entity: GameEntity,
        }
    }
}

#[derive(Bundle)]
pub struct CameraBundle {
    pub camera: Camera2d,
    pub main_camera: MainCamera,
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self {
            camera: Camera2d,
            main_camera: MainCamera,
        }
    }
}
