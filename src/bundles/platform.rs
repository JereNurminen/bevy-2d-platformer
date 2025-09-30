use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Platform;

#[derive(Bundle)]
pub struct PlatformBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub platform: Platform,
}
