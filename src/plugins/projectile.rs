use std::ops::Deref;

use avian2d::prelude::{Collider, RigidBody};
use bevy::{platform::time, prelude::*};

#[derive(Component)]
struct Projectile;

#[derive(Component, Clone)]
pub struct ProjectileVelocity(pub Vec2);

#[derive(Event, Clone)]
pub struct ProjectileSpawnEvent {
    pub transform: Transform,
    pub velocity: ProjectileVelocity,
    pub sprite: Handle<Image>,
}

pub fn spawn_projectile(
    mut commands: Commands,
    mut spawn_events: EventReader<ProjectileSpawnEvent>,
) {
    for event in spawn_events.read().into_iter() {
        println!("Projectile spawned at {:?}", event.transform.translation);
        commands.spawn((
            Projectile,
            event.transform,
            event.velocity.clone(),
            Sprite {
                image: event.sprite.clone_weak(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(3.0, 3.0),
        ));
    }
}

fn move_projectiles(
    mut query: Query<(&mut Transform, &ProjectileVelocity), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        let movement = velocity.0 * time.delta_secs();
        transform.translation += Vec3 {
            x: movement.x,
            y: movement.y,
            z: 0.0,
        };
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileSpawnEvent>()
            .add_systems(Update, (spawn_projectile, move_projectiles));
    }
}
