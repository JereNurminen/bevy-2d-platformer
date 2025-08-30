use bevy::prelude::*;
use bevy_ecs_ldtk::{
    IntGridCell, LdtkPlugin, LdtkSettings, LdtkWorldBundle, LevelSelection, LevelSpawnBehavior,
    app::{LdtkEntityAppExt, LdtkIntCellAppExt},
};

use crate::{
    bundles::{
        physics_tile::{PhysicsTile, PhysicsTileBundle},
        player::PlayerBundle,
    },
    level_enums::*,
    states::GameState,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        println!("Building level");
        app.add_systems(OnEnter(GameState::Game), setup_level)
            .add_plugins(LdtkPlugin::default())
            .insert_resource(LevelSelection::index(0))
            .register_ldtk_entity_for_layer::<PlayerBundle>(L__ENTITIES, E__PLAYER_START)
            .register_ldtk_int_cell_for_layer::<PhysicsTileBundle>(L__LEVEL_GEOMETRY, 1)
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                ..default()
            });
    }
}

pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Setting up level");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("ldtk/project.ldtk").into(),
        ..Default::default()
    });
}
