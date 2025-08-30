use bevy::{color::palettes::css, prelude::*, text::cosmic_text::ttf_parser::kern};

use avian2d::prelude::*;

use bevy_tnua::{
    math::{Float, Vector3},
    prelude::*,
};
use bevy_tnua_avian2d::*;

use crate::{
    bundles::player::Player,
    constants::{PLAYER_HEIGHT, PLAYER_WIDTH, pixels_to_world_units, times_phys_length_unit},
};

pub fn apply_controls(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TnuaController>) {
    let Ok(mut controller) = query.single_mut() else {
        println!("Found {:?} player entities", query.iter().count());
        return;
    };

    let mut direction = Vector3::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= Vector3::X;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += Vector3::X;
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * times_phys_length_unit(10),
        acceleration: times_phys_length_unit(40),
        air_acceleration: times_phys_length_unit(30),
        float_height: PLAYER_HEIGHT / 2.0 + pixels_to_world_units(1),
        coyote_time: 0.3,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        println!("jump");
        controller.action(TnuaBuiltinJump {
            height: times_phys_length_unit(4),
            takeoff_extra_gravity: times_phys_length_unit(60),
            takeoff_above_velocity: 2.0,
            fall_extra_gravity: times_phys_length_unit(20),
            shorten_extra_gravity: times_phys_length_unit(5),
            input_buffer_time: 0.1,
            ..Default::default()
        });
    }
}
