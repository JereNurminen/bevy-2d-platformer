use bevy::{color::palettes::css, prelude::*, text::cosmic_text::ttf_parser::kern};

use avian2d::prelude::*;

use bevy_tnua::{
    math::{Float, Vector3},
    prelude::*,
};
use bevy_tnua_avian2d::*;

use crate::{
    components::Player,
    constants::{PLAYER_HEIGHT, PLAYER_WIDTH, pixels_to_world_units, times_phys_length_unit},
};

pub fn setup_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::from_xyz(times_phys_length_unit(2), times_phys_length_unit(10), 0.0),
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::capsule(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
        // This is Tnua's interface component.
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian2dSensorShape(Collider::rectangle(PLAYER_WIDTH, 0.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
        Sprite {
            color: Color::srgb(0.3, 0.7, 0.3),
            custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
    ));
}

pub fn apply_controls(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TnuaController>) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    let mut direction = Vector3::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= Vector3::X;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += Vector3::X;
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * times_phys_length_unit(10),
        acceleration: times_phys_length_unit(20),
        air_acceleration: times_phys_length_unit(20),
        float_height: PLAYER_HEIGHT / 2.0 + pixels_to_world_units(1),
        coyote_time: 0.3,
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        println!("jump");
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: times_phys_length_unit(4),
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            takeoff_extra_gravity: times_phys_length_unit(60),
            takeoff_above_velocity: 2.0,
            fall_extra_gravity: times_phys_length_unit(20),
            shorten_extra_gravity: times_phys_length_unit(5),
            input_buffer_time: 0.1,
            ..Default::default()
        });
    }
}
