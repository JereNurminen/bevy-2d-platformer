use std::collections::{HashMap, HashSet};
use std::default;
use std::fmt::format;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::components::GameEntity;
use crate::constants::{self, TILE_SIZE};
use crate::layers::LEVEL_GEOMETRY_TILES;
use crate::{level, tile_merger};

/// Component marker for physics tiles created from IntGrid data
#[derive(Component, Debug, Default)]
pub struct PhysicsTile {}

#[derive(Component, Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TileCoords {
    pub x: i64,
    pub y: i64,
}

pub struct SingleTile {
    pub physics_tile: PhysicsTile,
    pub coords: TileCoords,
}

#[derive(Bundle, Debug)]
pub struct MergedTileColliderBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub transform: Transform,
}

#[derive(Component)]
pub struct StaticLevelData {
    pub level_identifier: String,
}

#[derive(Bundle)]
pub struct LevelBundle {
    pub level_data: StaticLevelData,
}
