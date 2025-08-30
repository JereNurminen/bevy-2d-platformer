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

impl PlatformBundle {
    pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
        Self {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(size.x, size.y),
            platform: Platform,
        }
    }
}
