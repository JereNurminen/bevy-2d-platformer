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

use plugins::*;
use states::GameState;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(constants::PHYSICS_LENGTH_UNIT),
            PhysicsDebugPlugin::default(),
        ))
        .init_state::<GameState>()
        .add_plugins((
            CameraPlugin,
            MenuPlugin,
            PlayerPlugin,
            GamePlugin,
            CollisionPlugin,
        ))
        .run();
}
