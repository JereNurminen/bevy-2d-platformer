use avian2d::prelude::*;
use bevy::prelude::*;

/// Component marker for physics tiles created from IntGrid data
#[derive(Component, Debug, Default)]
pub struct PhysicsTile {}

#[derive(Component, Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TileCoords {
    pub x: i64,
    pub y: i64,
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
