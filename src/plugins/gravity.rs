use bevy::prelude::*;

use super::collision::{IsGrounded, Velocity};

#[derive(Default, Component)]
pub struct EntityGravity {
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub enabled: bool,
}

pub fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&EntityGravity, &mut Velocity, Option<&IsGrounded>)>,
) {
    for (gravity, mut velocity, is_grounded) in query.iter_mut() {
        if gravity.enabled && velocity.0.y > -gravity.max_fall_speed {
            if let Some(is_grounded) = is_grounded {
                if !is_grounded.0 {
                    velocity.0.y -= gravity.gravity * time.delta_secs()
                }
            } else {
                velocity.0.y -= gravity.gravity * time.delta_secs()
            }
        }
    }
}

////

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_gravity);
    }
}
