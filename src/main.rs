// Cargo.toml dependencies:
// [dependencies]
// bevy = "0.16"
// avian2d = "0.1"

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod bundles;
mod components;
mod constants;
mod level_enums;
mod plugins;
mod states;
mod tile_merger;

use bevy_tnua::{TnuaUserControlsSystemSet, prelude::TnuaControllerPlugin};
use bevy_tnua_avian2d::TnuaAvian2dPlugin;
//use bundles::spawn_point::{PlayerSpawnBundle, PlayerSpawnPlugin, RespawnPlayer};
pub use constants::times_phys_length_unit;
use level::LevelPlugin;
use level_enums::*;
use player::apply_controls;
use plugins::*;
use states::GameState;

pub use constants::{entities, enums, layers, levels};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default().with_length_unit(constants::TILE_SIZE),
            TnuaAvian2dPlugin::new(FixedUpdate),
            PhysicsDebugPlugin::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            CameraPlugin,
            GamePlugin,
            LevelPlugin,
        ))
        .add_systems(
            FixedUpdate,
            apply_controls.in_set(TnuaUserControlsSystemSet),
        )
        .insert_resource(Gravity(Vec2::NEG_Y * times_phys_length_unit(10)))
        //.add_systems(Startup, setup_player)
        .init_state::<GameState>()
        .run();
}
