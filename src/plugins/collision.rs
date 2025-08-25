use avian2d::prelude::*;
use bevy::prelude::*;

use crate::components::{CollisionInfo, IsGrounded, KinematicVelocity, Player};
use crate::constants::*;
use crate::states::GameState;

// Small buffer to prevent entities from sinking into surfaces
const COLLISION_BUFFER: f32 = 0.001;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_grounded_status
                .after(crate::plugins::player::kinematic_player_movement)
                .before(handle_kinematic_collisions)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            Update,
            handle_kinematic_collisions
                .after(update_grounded_status)
                .before(crate::plugins::player::apply_kinematic_movement)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            Update,
            correct_positions
                .after(crate::plugins::player::apply_kinematic_movement)
                .run_if(in_state(GameState::Game)),
        );
    }
}

fn update_grounded_status(
    mut player_query: Query<(Entity, &Transform, &mut IsGrounded, &KinematicVelocity)>,
    spatial_query: SpatialQuery,
) {
    for (entity, transform, mut is_grounded, velocity) in &mut player_query {
        // Start ground check from bottom of player
        let player_bottom = transform.translation.y - PLAYER_HEIGHT / 2.0;
        let check_position = Vec2::new(
            transform.translation.x,
            player_bottom + GROUND_CHECK_OFFSET_Y,
        );

        let ground_cast = spatial_query.cast_shape(
            &Collider::rectangle(PLAYER_WIDTH, GROUND_CHECK_HEIGHT),
            check_position,
            0.0,
            Dir2::NEG_Y,
            &ShapeCastConfig {
                max_distance: GROUND_CHECK_DISTANCE,
                ..default()
            },
            &SpatialQueryFilter::from_excluded_entities([entity]),
        );

        let was_grounded = is_grounded.0;
        let ground_hit = ground_cast.is_some();
        let ground_distance = ground_cast.map(|hit| hit.distance).unwrap_or(f32::MAX);
        let distance_threshold = GROUND_CHECK_DISTANCE * 0.8;

        is_grounded.0 = ground_hit && ground_distance < distance_threshold;

        // Debug output - only show state changes
        if was_grounded != is_grounded.0 {
            println!(
                "Ground state CHANGED: {} -> {}, velocity.y: {}",
                was_grounded, is_grounded.0, velocity.velocity.y
            );
        }
    }
}

pub fn handle_kinematic_collisions(
    mut velocity_query: Query<(
        Entity,
        &Transform,
        &mut KinematicVelocity,
        &mut CollisionInfo,
    )>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for (entity, transform, mut velocity, mut collision_info) in &mut velocity_query {
        let dt = time.delta_secs();
        let entity_size = Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT); // Make this configurable later
        let movement = velocity.velocity * dt;

        // Clear previous collision info
        collision_info.horizontal_distance = None;
        collision_info.vertical_distance = None;

        if movement.length() < MIN_MOVEMENT_THRESHOLD {
            continue;
        }

        // Horizontal collision
        if movement.x != 0.0 {
            let horizontal_cast = spatial_query.cast_shape(
                &Collider::rectangle(entity_size.x, entity_size.y),
                transform.translation.truncate(),
                0.0,
                if movement.x > 0.0 {
                    Dir2::X
                } else {
                    Dir2::NEG_X
                },
                &ShapeCastConfig {
                    max_distance: movement.x.abs(),
                    ..default()
                },
                &SpatialQueryFilter::from_excluded_entities([entity]),
            );

            if let Some(hit) = horizontal_cast {
                if hit.distance <= movement.x.abs() + COLLISION_BUFFER {
                    velocity.velocity.x = 0.0;
                    collision_info.horizontal_distance = Some(hit.distance);
                }
            }
        }

        // Vertical collision - only check when moving downward or very slow upward
        if movement.y < 0.0 || (movement.y > 0.0 && movement.y < SLOW_UPWARD_THRESHOLD * dt) {
            let vertical_cast = spatial_query.cast_shape(
                &Collider::rectangle(entity_size.x, entity_size.y),
                transform.translation.truncate(),
                0.0,
                if movement.y > 0.0 {
                    Dir2::Y
                } else {
                    Dir2::NEG_Y
                },
                &ShapeCastConfig {
                    max_distance: movement.y.abs(),
                    ..default()
                },
                &SpatialQueryFilter::from_excluded_entities([entity]),
            );

            if let Some(hit) = vertical_cast {
                if hit.distance <= movement.y.abs() + COLLISION_BUFFER {
                    velocity.velocity.y = 0.0;
                    collision_info.vertical_distance = Some(hit.distance);
                }
            }
        }
    }
}

fn correct_positions(
    mut query: Query<(&mut Transform, &CollisionInfo, &KinematicVelocity), With<Player>>,
) {
    for (mut transform, collision_info, velocity) in &mut query {
        // If we had a vertical collision while moving downward (landing),
        // ensure we're positioned correctly on the ground
        if let Some(distance) = collision_info.vertical_distance {
            if velocity.velocity.y == 0.0 && distance < COLLISION_BUFFER * 2.0 {
                // We're very close to or slightly embedded in the ground
                // Move up slightly to ensure we're exactly on the surface
                transform.translation.y += COLLISION_BUFFER;
            }
        }
    }
}
