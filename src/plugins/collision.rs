use std::f32::{INFINITY, NEG_INFINITY};

use avian2d::prelude::*;
use avian2d::spatial_query::ShapeCastConfig;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_inspector_egui::InspectorOptions;

use super::player::AfterJumpGravityImmunityTimer;

#[derive(Component, Default)]
pub struct IsGrounded(pub bool);

#[derive(Component, Default)]
pub struct IsTouchingWallLeft(pub bool);

#[derive(Component, Default)]
pub struct IsTouchingWallRight(pub bool);

#[derive(Component, Default)]
pub struct IsTouchingCeiling(pub bool);

#[derive(Component, Default)]
pub struct CollisionConfig {
    pub ground_check_distance: f32,
    pub wall_check_distance: f32,
    pub ceiling_check_distance: f32,
    pub collision_filter: SpatialQueryFilter,
}

#[derive(Component, Default, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct GroundedStopwatch(pub Stopwatch);

#[derive(Bundle, Default)]
pub struct CollisionBundle {
    pub is_grounded: IsGrounded,
    pub is_touching_wall_left: IsTouchingWallLeft,
    pub is_touching_wall_right: IsTouchingWallRight,
    pub is_touching_ceiling: IsTouchingCeiling,
    pub grounded_stopwatch: GroundedStopwatch,
    pub config: CollisionConfig,
    pub velocity: Velocity,
}

fn shape_cast(
    spatial_query: &SpatialQuery,
    origin: Vec2,
    direction: Vec2,
    distance: f32,
    collider: &Collider,
    filter: &SpatialQueryFilter,
) -> Option<ShapeHitData> {
    if let Ok(dir) = Dir2::new(direction) {
        let hits = spatial_query.cast_shape(
            &collider,
            origin,
            0.0,
            dir,
            &ShapeCastConfig {
                max_distance: distance,
                ..Default::default()
            },
            &filter,
        );
        return hits;
    }
    None
}

pub fn check_grounded_state(
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &mut IsGrounded,
            &CollisionConfig,
            &Transform,
            &Children,
            Option<&mut GroundedStopwatch>,
            &mut Velocity,
        ),
        Without<Collider>,
    >,
    collider_query: Query<(&Collider, &Transform)>,
    time: Res<Time>,
) {
    for (mut is_grounded, config, transform, children, grounded_stopwatch, mut velocity) in
        query.iter_mut()
    {
        // Find the collider and its transform from children
        let collider_data = children
            .iter()
            .find_map(|child| collider_query.get(child).ok());

        let Some((collider, collider_transform)) = collider_data else {
            continue;
        };

        let hit = shape_cast(
            &spatial_query,
            Vec2 {
                x: transform.translation.x + collider_transform.translation.x,
                y: transform.translation.y + collider_transform.translation.y,
            },
            Vec2::NEG_Y,
            config.ground_check_distance,
            collider,
            &config.collision_filter,
        );

        if let Some(_hit) = hit {
            *is_grounded = IsGrounded(true);
            velocity.0.y = velocity.0.y.clamp(0.0, INFINITY);
            if let Some(mut stopwatch) = grounded_stopwatch {
                stopwatch.0.reset();
            }
        } else {
            *is_grounded = IsGrounded(false);
            if let Some(mut stopwatch) = grounded_stopwatch {
                stopwatch.0.tick(time.delta());
            }
        }
    }
}

pub fn check_ceiling_state(
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &mut IsTouchingCeiling,
            &CollisionConfig,
            &Transform,
            &Children,
            &mut Velocity,
            Option<&mut AfterJumpGravityImmunityTimer>,
        ),
        Without<Collider>,
    >,
    collider_query: Query<(&Collider, &Transform)>,
) {
    for (
        mut is_touching_ceiling,
        config,
        transform,
        children,
        mut velocity,
        after_jump_gravity_immunity_timer,
    ) in query.iter_mut()
    {
        // Find the collider and its transform from children
        let collider_data = children
            .iter()
            .find_map(|child| collider_query.get(child).ok());

        let Some((collider, collider_transform)) = collider_data else {
            continue;
        };

        let hit = shape_cast(
            &spatial_query,
            Vec2 {
                x: transform.translation.x + collider_transform.translation.x,
                y: transform.translation.y + collider_transform.translation.y,
            },
            Vec2::Y,
            config.ceiling_check_distance,
            collider,
            &config.collision_filter,
        );
        if let Some(_hit) = hit {
            *is_touching_ceiling = IsTouchingCeiling(true);
            velocity.0.y = velocity.0.y.clamp(NEG_INFINITY, -1.0);
            // If the entity (i.e. the player) has immunity to gravity after jumping for a set time,
            // finish the timer manually here
            if let Some(mut timer) = after_jump_gravity_immunity_timer {
                let duration = timer.0.duration();
                timer.0.set_elapsed(duration);
            }
        } else {
            *is_touching_ceiling = IsTouchingCeiling(false);
        }
    }
}

pub fn check_wall_left_state(
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &mut IsTouchingWallLeft,
            &CollisionConfig,
            &Transform,
            &Children,
            &mut Velocity,
        ),
        Without<Collider>,
    >,
    collider_query: Query<(&Collider, &Transform)>,
) {
    for (mut is_touching_wall_left, config, transform, children, mut velocity) in query.iter_mut() {
        // Find the collider and its transform from children
        let collider_data = children
            .iter()
            .find_map(|child| collider_query.get(child).ok());

        let Some((collider, collider_transform)) = collider_data else {
            continue;
        };

        let hit = shape_cast(
            &spatial_query,
            Vec2 {
                x: transform.translation.x + collider_transform.translation.x,
                y: transform.translation.y + collider_transform.translation.y + 1.0,
            },
            Vec2::NEG_X,
            config.wall_check_distance,
            collider,
            &config.collision_filter,
        );
        if let Some(_hit) = hit {
            *is_touching_wall_left = IsTouchingWallLeft(true);
            velocity.0.x = velocity.0.x.clamp(0.0, INFINITY);
            println!("touching wall LEFT");
        } else {
            *is_touching_wall_left = IsTouchingWallLeft(false);
        }
    }
}

pub fn check_wall_right_state(
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &mut IsTouchingWallRight,
            &CollisionConfig,
            &Transform,
            &Children,
            &mut Velocity,
        ),
        Without<Collider>,
    >,
    collider_query: Query<(&Collider, &Transform)>,
) {
    for (mut is_touching_wall_right, config, transform, children, mut velocity) in query.iter_mut()
    {
        // Find the collider and its transform from children
        let collider_data = children
            .iter()
            .find_map(|child| collider_query.get(child).ok());

        let Some((collider, collider_transform)) = collider_data else {
            continue;
        };

        let hit = shape_cast(
            &spatial_query,
            Vec2 {
                x: transform.translation.x + collider_transform.translation.x,
                y: transform.translation.y + collider_transform.translation.y + 1.0,
            },
            Vec2::X,
            config.wall_check_distance,
            collider,
            &config.collision_filter,
        );
        if let Some(_hit) = hit {
            *is_touching_wall_right = IsTouchingWallRight(true);
            velocity.0.x = velocity.0.x.clamp(0.0, INFINITY);
            println!("touching wall RIGHT");
        } else {
            *is_touching_wall_right = IsTouchingWallRight(false);
        }
    }
}

pub fn apply_velocity(
    spatial_query: SpatialQuery,
    time: Res<Time>,
    mut query: Query<
        (
            &CollisionConfig,
            &Children,
            &mut Velocity,
            &mut Transform,
            Option<&IsTouchingWallLeft>,
            Option<&IsTouchingWallRight>,
            Option<&IsTouchingCeiling>,
        ),
        Without<Collider>,
    >,
    collider_query: Query<(&Collider, &Transform)>,
) {
    for (
        config,
        children,
        mut velocity,
        mut transform,
        is_touching_wall_left,
        is_touching_wall_right,
        is_touching_ceiling,
    ) in query.iter_mut()
    {
        // Find the collider and its transform from children
        let collider_data = children
            .iter()
            .find_map(|child| collider_query.get(child).ok());

        let Some((collider, collider_transform)) = collider_data else {
            continue;
        };

        if -1.0 < velocity.0.x && velocity.0.x < 1.0 {
            velocity.0.x = 0.0;
        }

        if let Some(is_touching_wall_left) = is_touching_wall_left {
            if is_touching_wall_left.0 && velocity.0.x < 0.0 {
                velocity.0.x = 0.0;
            }
        }

        if let Some(is_touching_wall_right) = is_touching_wall_right {
            if is_touching_wall_right.0 && velocity.0.x > 0.0 {
                velocity.0.x = 0.0;
            }
        }

        if let Some(is_touching_ceiling) = is_touching_ceiling {
            if is_touching_ceiling.0 && velocity.0.y > 0.0 {
                velocity.0.y = -1.0;
            }
        }

        if velocity.0.length() == 0.0 || velocity.0.length() == INFINITY {
            continue;
        }

        let target_distance = velocity.0.length() * time.delta_secs();
        let hit = shape_cast(
            &spatial_query,
            Vec2 {
                x: transform.translation.x + collider_transform.translation.x,
                y: transform.translation.y + collider_transform.translation.y,
            },
            velocity.0.normalize(),
            target_distance,
            collider,
            &config.collision_filter,
        );
        let actual_distance = hit.map_or(target_distance, |hit| hit.distance - 0.1);
        *transform = transform.with_translation(Vec3 {
            x: transform.translation.x + (velocity.0.normalize() * actual_distance).x,
            y: transform.translation.y + (velocity.0.normalize() * actual_distance).y,
            z: transform.translation.z,
        });
    }
}

////

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_grounded_state,
                check_wall_left_state,
                check_wall_right_state,
                check_ceiling_state,
                apply_velocity,
            ),
        )
        .register_type::<GroundedStopwatch>()
        .register_type::<Velocity>();
    }
}
