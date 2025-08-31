use std::collections::{HashMap, HashSet};

use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{
    bundles::{
        level::{
            LevelBundle, MergedTileColliderBundle, PhysicsTile, SingleTile, StaticLevelData,
            TileCoords,
        },
        player::PlayerBundle,
    },
    constants::{self, TILE_SIZE},
    level_enums::*,
    states::GameState,
    tile_merger::{self, TileMerger},
    times_phys_length_unit,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        println!("Building level");
        app.add_systems(OnEnter(GameState::Game), (setup_level));
    }
}

pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                    let tiles: Vec<MergedTileColliderBundle> = layer
                        .int_grid_csv
                        .iter()
                        .enumerate()
                        .filter_map(|(index, &tile)| {
                            if tile == 1 {
                                let x = (index % width) as i64;
                                let y = (index / width) as i64;
                                Some(MergedTileColliderBundle {
                                    rigid_body: RigidBody::Static,
                                    collider: Collider::rectangle(TILE_SIZE, TILE_SIZE),
                                    transform: Transform::from_xyz(
                                        times_phys_length_unit(x) as f32,
                                        times_phys_length_unit(y) * -1 as f32,
                                        0.0,
                                    ),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    println!("Tiles for level: {:?}", tiles);

                    commands
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
                        ))
                        .with_children(|level_parent| {
                            for tile in tiles {
                                level_parent.spawn(tile);
                            }
                        });
                    /*
                    let tile_merger = tile_merger::TileMerger::new(TILE_SIZE);
                    let merged_tiles =
                        tile_merger
                            .merge_tiles(&tiles)
                            .into_iter()
                            .map(|merged_tile| MergedTileColliderBundle {
                                rigid_body: RigidBody::Static,
                                collider: Collider::rectangle(
                                    merged_tile.width as f32,
                                    merged_tile.height as f32,
                                ),
                                transform: Transform::from_xyz(
                                    times_phys_length_unit(merged_tile.x) as f32,
                                    times_phys_length_unit(merged_tile.y) as f32,
                                    0.0,
                                ),
                            });

                    println!("Read level data: {:?}", level_data);
                    println!(
                        "Merged {:?} tiles into {:?} rectangles!",
                        tiles.len(),
                        merged_tiles.len()
                    );

                    commands
                        .spawn((
                            LevelBundle {
                                level_data: StaticLevelData {
                                    level_identifier: "test".to_string(),
                                },
                            },
                            Transform::from_xyz(
                                level_data.world_x as f32,
                                level_data.world_y as f32,
                                0.0,
                            ),
                        ))
                        .with_children(|level_parent| {
                            for tile in merged_tiles {
                                level_parent.spawn(tile);
                            }
                        });
                        */
                }
                constants::layers::ENTITIES => {
                    for entity in layer.entity_instances.iter() {
                        match entity.identifier.as_str() {
                            constants::entities::PLAYER_START => {
                                println!("Spawning player, data: {:?}", entity);
                                commands.spawn(PlayerBundle {
                                    transform: Transform::from_xyz(
                                        entity.world_x.unwrap() as f32,
                                        (entity.world_y.unwrap() * -1) as f32,
                                        0.0,
                                    ),
                                    ..Default::default()
                                });
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
