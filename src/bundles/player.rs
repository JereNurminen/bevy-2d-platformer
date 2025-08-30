use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkEntity;
use bevy_ecs_ldtk::Worldly;
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use crate::components::*;
use crate::constants::*;

#[derive(Component)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub game_entity: GameEntity,
    pub tnua_controller: TnuaController,
    sensor_shape: TnuaAvian2dSensorShape,
    #[worldly]
    worldly: Worldly,
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
            transform: Transform::from_xyz(
                times_phys_length_unit(0),
                times_phys_length_unit(0),
                0.0,
            ),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            tnua_controller: TnuaController::default(),
            game_entity: GameEntity,
            sensor_shape: TnuaAvian2dSensorShape(Collider::rectangle(PLAYER_WIDTH, 0.0)),
            worldly: Worldly::default(),
        }
    }
}
