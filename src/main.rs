// Cargo.toml dependencies:
// [dependencies]
// bevy = "0.16"
// avian2d = "0.1"

use avian2d::prelude::*;
use bevy::prelude::*;

mod bundles;
mod components;
mod constants;
mod plugins;
mod states;

use bevy_tnua::{TnuaUserControlsSystemSet, prelude::TnuaControllerPlugin};
use bevy_tnua_avian2d::TnuaAvian2dPlugin;
use constants::times_phys_length_unit;
use player::{apply_controls, setup_player};
use plugins::*;
use states::GameState;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(constants::PHYSICS_LENGTH_UNIT),
            TnuaAvian2dPlugin::new(FixedUpdate),
            PhysicsDebugPlugin::default(),
            TnuaControllerPlugin::new(FixedUpdate),
        ))
        .init_state::<GameState>()
        .add_plugins((CameraPlugin, MenuPlugin, GamePlugin))
        .add_systems(Startup, setup_player)
        .add_systems(
            FixedUpdate,
            apply_controls.in_set(TnuaUserControlsSystemSet),
        )
        .insert_resource(Gravity(Vec2::NEG_Y * times_phys_length_unit(10)))
        .run();
}
