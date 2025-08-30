use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, IntGridCell, LdtkIntCell};

use crate::components::GameEntity;
use crate::constants::PHYSICS_LENGTH_UNIT;

/// Component marker for physics tiles created from IntGrid data
#[derive(Component, Debug, LdtkIntCell, Default)]
pub struct PhysicsTile {}

/// Bundle for creating physics collision tiles from LDtk IntGrid data
#[derive(Bundle, LdtkIntCell)]
pub struct PhysicsTileBundle {
    pub physics_tile: PhysicsTile,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub transform: Transform,
}

impl Default for PhysicsTileBundle {
    fn default() -> Self {
        Self {
            physics_tile: PhysicsTile::default(),
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(PHYSICS_LENGTH_UNIT, PHYSICS_LENGTH_UNIT),
            transform: Transform::default(),
        }
    }
}
