use std::collections::HashSet;

use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;

use crate::{
    bundles::level::{LevelBundle, StaticLevelData, TileCoords},
    constants::{self, GameLayer, TILE_SIZE},
    states::GameState,
    tile_merger::TileMerger,
};

use super::player::PlayerSpawnEvent;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        println!("Building level");
        app.add_systems(OnEnter(GameState::Game), setup_level);
    }
}

pub fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<PlayerSpawnEvent>,
) {
    let project = ldtk_rust::Project::new("assets/ldtk/project.ldtk");
    let level_data = project
        .levels
        .iter()
        .find(|level| level.identifier == constants::levels::LEVEL_0)
        .unwrap();

    if let Some(layers) = &level_data.layer_instances {
        for layer in layers {
            let identifier = layer.identifier.clone();
            match identifier.as_str() {
                constants::layers::LEVEL_GEOMETRY => {
                    let width = layer.c_wid as usize;

                    // Collect all solid tile positions
                    let mut tile_positions = HashSet::new();
                    for (index, &tile) in layer.int_grid_csv.iter().enumerate() {
                        if tile == 1 {
                            let x = (index % width) as i64;
                            let y = (index / width) as i64;
                            tile_positions.insert(TileCoords { x, y });
                        }
                    }

                    println!("Found {} individual tiles", tile_positions.len());

                    // Use tile merger to create optimized colliders
                    let tile_merger = TileMerger::new(TILE_SIZE);
                    let collider_data = tile_merger.create_collider_data(&tile_positions);

                    println!("Merged into {} physics colliders", collider_data.len());

                    let level_entity = commands
                        .spawn((
                            LevelBundle {
                                level_data: StaticLevelData {
                                    level_identifier: "test".to_string(),
                                },
                            },
                            Transform::from_xyz(
                                level_data.world_x as f32,
                                (level_data.world_y * -1) as f32,
                                0.0,
                            ),
                            Sprite {
                                image: asset_server.load(format!(
                                    "ldtk/project/simplified/{}/_composite.png",
                                    level_data.identifier
                                )),
                                anchor: bevy::sprite::Anchor::TopLeft,
                                ..default()
                            },
                        ))
                        .id();

                    // Spawn merged colliders as children of the level
                    for (center_x, center_y, width, height) in collider_data {
                        let collider_entity = commands
                            .spawn((
                                RigidBody::Static,
                                Collider::rectangle(width, height),
                                Transform::from_xyz(
                                    center_x,
                                    center_y * -1.0, // Flip Y coordinate for Bevy
                                    0.0,
                                ),
                                CollisionLayers::new(
                                    GameLayer::LevelGeometry,
                                    [GameLayer::Player, GameLayer::Default],
                                ),
                            ))
                            .id();

                        commands.entity(level_entity).add_child(collider_entity);
                    }
                }
                constants::layers::ENTITIES => {
                    for entity in layer.entity_instances.iter() {
                        match entity.identifier.as_str() {
                            constants::entities::PLAYER_START => {
                                println!("Spawning player, data: {:?}", entity);
                                event_writer.write(PlayerSpawnEvent(Transform::from_xyz(
                                    entity.world_x.unwrap() as f32,
                                    (entity.world_y.unwrap() * -1) as f32,
                                    1.0,
                                )));
                            }
                            _ => {
                                warn!("unhandled entity id: {:?}", entity.identifier)
                            }
                        }
                    }
                }
                constants::layers::LEVEL_GEOMETRY_TILES => {}
                _ => {
                    warn!("unhandled layer id: {:?}", layer.identifier)
                }
            }
        }
    }
}
