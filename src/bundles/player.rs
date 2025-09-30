use avian2d::prelude::*;
use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::gravity::EntityGravity;

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub game_entity: GameEntity,
    pub gravity: EntityGravity,
    // pub tnua_controller: TnuaController,
    // pub sensor_shape: TnuaAvian2dSensorShape,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        println!("Constructing a player bundle");
        Self {
            player: Player,
            sprite: Sprite {
                color: Color::srgb(0.3, 0.7, 0.3),
                custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(multiply_by_tile_size(0), multiply_by_tile_size(0), 0.0),
            rigid_body: RigidBody::Kinematic,
            collider: Collider::capsule(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            game_entity: GameEntity,
            gravity: EntityGravity {
                gravity: 10.0,
                max_fall_speed: 10.0,
                enabled: true,
            },
            //tnua_controller: TnuaController::default(),
            //sensor_shape: TnuaAvian2dSensorShape(Collider::rectangle(PLAYER_WIDTH, 0.0)),
        }
    }
}
