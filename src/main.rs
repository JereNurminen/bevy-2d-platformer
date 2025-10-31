use aseprite_deserialize::Aseprite;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod aseprite_deserialize;
mod bundles;
mod components;
mod constants;
mod level_enums;
mod plugins;
mod states;
mod tile_merger;

use animation_library::AnimationLibraryPlugin;
use collision::CollisionPlugin;
pub use constants::multiply_by_tile_size;
use gravity::GravityPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;
use level::LevelPlugin;
use player::{PlayerAction, PlayerPlugin};
use plugins::*;
use projectile::ProjectilePlugin;
use states::GameState;

pub use constants::{entities, enums, layers, levels};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default().with_length_unit(constants::TILE_SIZE),
            PhysicsDebugPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            JsonAssetPlugin::<Aseprite>::new(&["json"]),
            InputManagerPlugin::<PlayerAction>::default(),
            AnimationLibraryPlugin,
            PlayerPlugin,
            CameraPlugin,
            GamePlugin,
            LevelPlugin,
            CollisionPlugin,
            GravityPlugin,
            ProjectilePlugin,
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * multiply_by_tile_size(10)))
        .init_state::<GameState>()
        .run();
}
