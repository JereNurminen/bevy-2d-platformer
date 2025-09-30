use avian2d::prelude::*;
use avian2d::spatial_query::ShapeCastConfig;
use bevy::prelude::*;

// Comprehensive collision state updated during movement resolution
#[derive(Component, Default)]
pub struct CollisionState {
    pub grounded: bool,
    pub against_left_wall: bool,
    pub against_right_wall: bool,
    pub touching_ceiling: bool,

    // Surface normals for advanced movement (wall jumps, slopes, etc.)
    pub ground_normal: Vec2,
    pub left_wall_normal: Vec2,
    pub right_wall_normal: Vec2,
    pub ceiling_normal: Vec2,

    // Advanced collision info for Metroidvania mechanics
    pub on_slope: bool,
    pub slope_angle: f32,  // In radians
    pub ledge_left: bool,  // Can grab ledge on left
    pub ledge_right: bool, // Can grab ledge on right

    // For debugging/gameplay
    pub last_ground_position: Vec2,
    pub time_since_grounded: f32,
}

// Movement component
#[derive(Component)]
pub struct Velocity(pub Vec2);

// Configuration for different entity types
#[derive(Component)]
pub struct CollisionConfig {
    pub ground_check_distance: f32,
    pub wall_check_distance: f32,
    pub ceiling_check_distance: f32,
    pub ledge_check_distance: f32,
    pub slope_threshold: f32, // Angle in radians where we consider it a slope vs wall
    pub coyote_time: f32,     // Time after leaving ground where jumps are still allowed
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            ground_check_distance: 2.0,
            wall_check_distance: 2.0,
            ceiling_check_distance: 2.0,
            ledge_check_distance: 8.0, // Larger for ledge detection
            slope_threshold: 0.7854,   // 45 degrees in radians
            coyote_time: 0.1,
        }
    }
}

// Helper function to cast in a direction and get detailed hit info
fn cast_collision_ray(
    spatial_query: &SpatialQuery,
    entity: Entity,
    origin: Vec2,
    direction: Vec2,
    distance: f32,
) -> Option<RayHitData> {
    if let Ok(dir) = Dir2::new(direction) {
        spatial_query.cast_ray(
            origin,
            dir,
            distance,
            true,
            &SpatialQueryFilter::default().with_excluded_entities([entity]),
        )
    } else {
        None
    }
}

// Main integrated movement and collision system
pub fn kinematic_movement_with_collision_detection(
    spatial_query: SpatialQuery,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &Collider,
            &mut Velocity,
            &mut CollisionState,
            &CollisionConfig,
        ),
        With<RigidBody>,
    >,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut transform, collider, mut velocity, mut collision_state, config) in
        query.iter_mut()
    {
        let desired_translation = velocity.0 * dt;

        // Update time since grounded
        if collision_state.grounded {
            collision_state.time_since_grounded = 0.0;
        } else {
            collision_state.time_since_grounded += dt;
        }

        // Reset collision state
        let mut new_collision_state = CollisionState {
            last_ground_position: collision_state.last_ground_position,
            time_since_grounded: collision_state.time_since_grounded,
            ..Default::default()
        };

        if desired_translation.length() < 0.001 {
            // Even if not moving, check for ground/wall contact
            update_static_collision_state(
                &spatial_query,
                entity,
                &transform,
                &mut new_collision_state,
                config,
            );
            *collision_state = new_collision_state;
            continue;
        }

        // Separate X and Y movement for better collision handling
        let movement_x = Vec2::new(desired_translation.x, 0.0);
        let movement_y = Vec2::new(0.0, desired_translation.y);

        // Apply X movement first
        if movement_x.length() > 0.001 {
            apply_movement_axis(
                &spatial_query,
                entity,
                &mut transform,
                collider,
                &mut velocity,
                &mut new_collision_state,
                config,
                movement_x,
                true, // is_horizontal
            );
        }

        // Then Y movement
        if movement_y.length() > 0.001 {
            apply_movement_axis(
                &spatial_query,
                entity,
                &mut transform,
                collider,
                &mut velocity,
                &mut new_collision_state,
                config,
                movement_y,
                false, // is_horizontal
            );
        }

        // Final collision state update
        update_static_collision_state(
            &spatial_query,
            entity,
            &transform,
            &mut new_collision_state,
            config,
        );

        // Update ground position tracking
        if new_collision_state.grounded {
            new_collision_state.last_ground_position = transform.translation.truncate();
            new_collision_state.time_since_grounded = 0.0;
        }

        *collision_state = new_collision_state;
    }
}

fn apply_movement_axis(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &mut Transform,
    collider: &Collider,
    velocity: &mut Velocity,
    collision_state: &mut CollisionState,
    config: &CollisionConfig,
    movement: Vec2,
    is_horizontal: bool,
) {
    let direction = movement.normalize();
    let distance = movement.length();

    if let Ok(dir) = Dir2::new(direction) {
        let shape_config = ShapeCastConfig {
            max_distance: distance,
            target_distance: 0.0,
            compute_contact_on_penetration: true,
            ignore_origin_penetration: false,
        };
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            transform.translation.truncate(),
            0.0,
            dir,
            &shape_config,
            &SpatialQueryFilter::default().with_excluded_entities([entity]),
        ) {
            // Move to just before collision point
            let safe_distance = (hit.distance - 0.001).max(0.0);
            let safe_movement = direction * safe_distance;
            transform.translation += safe_movement.extend(0.0);

            // Update collision state based on collision normal
            let normal = hit.normal1;

            if is_horizontal {
                // Horizontal collision - wall
                if normal.x > 0.5 {
                    collision_state.against_left_wall = true;
                    collision_state.left_wall_normal = normal;
                } else if normal.x < -0.5 {
                    collision_state.against_right_wall = true;
                    collision_state.right_wall_normal = normal;
                }

                // Stop horizontal velocity on wall collision
                velocity.0.x = 0.0;
            } else {
                // Vertical collision
                if normal.y > 0.5 {
                    // Hit ground
                    collision_state.grounded = true;
                    collision_state.ground_normal = normal;

                    // Check if it's a slope
                    let slope_angle = normal.angle_to(Vec2::Y);
                    if slope_angle.abs() > config.slope_threshold {
                        collision_state.on_slope = true;
                        collision_state.slope_angle = slope_angle;
                    }

                    velocity.0.y = velocity.0.y.min(0.0); // Don't bounce off ground
                } else if normal.y < -0.5 {
                    // Hit ceiling
                    collision_state.touching_ceiling = true;
                    collision_state.ceiling_normal = normal;
                    velocity.0.y = velocity.0.y.max(0.0); // Don't stick to ceiling
                }
            }
        } else {
            // No collision, apply full movement
            transform.translation += movement.extend(0.0);
        }
    } else {
        // Invalid direction, apply full movement
        transform.translation += movement.extend(0.0);
    }
}

fn update_static_collision_state(
    spatial_query: &SpatialQuery,
    entity: Entity,
    transform: &Transform,
    collision_state: &mut CollisionState,
    config: &CollisionConfig,
) {
    let position = transform.translation.truncate();

    // Check ground
    if let Some(hit) = cast_collision_ray(
        spatial_query,
        entity,
        position,
        Vec2::NEG_Y,
        config.ground_check_distance,
    ) {
        collision_state.grounded = true;
        collision_state.ground_normal = hit.normal;

        // Slope detection
        let slope_angle = hit.normal.angle_to(Vec2::Y);
        if slope_angle.abs() > config.slope_threshold {
            collision_state.on_slope = true;
            collision_state.slope_angle = slope_angle;
        }
    }

    // Check walls
    if let Some(hit) = cast_collision_ray(
        spatial_query,
        entity,
        position,
        Vec2::NEG_X,
        config.wall_check_distance,
    ) {
        collision_state.against_left_wall = true;
        collision_state.left_wall_normal = hit.normal;
    }

    if let Some(hit) = cast_collision_ray(
        spatial_query,
        entity,
        position,
        Vec2::X,
        config.wall_check_distance,
    ) {
        collision_state.against_right_wall = true;
        collision_state.right_wall_normal = hit.normal;
    }

    // Check ceiling
    if let Some(_hit) = cast_collision_ray(
        spatial_query,
        entity,
        position,
        Vec2::Y,
        config.ceiling_check_distance,
    ) {
        collision_state.touching_ceiling = true;
    }

    // Ledge detection - check if there's a wall but no ground beyond it
    if collision_state.against_left_wall {
        let ledge_check_pos = position + Vec2::new(-config.ledge_check_distance, 0.0);
        if cast_collision_ray(
            spatial_query,
            entity,
            ledge_check_pos,
            Vec2::NEG_Y,
            config.ground_check_distance,
        )
        .is_none()
        {
            collision_state.ledge_left = true;
        }
    }

    if collision_state.against_right_wall {
        let ledge_check_pos = position + Vec2::new(config.ledge_check_distance, 0.0);
        if cast_collision_ray(
            spatial_query,
            entity,
            ledge_check_pos,
            Vec2::NEG_Y,
            config.ground_check_distance,
        )
        .is_none()
        {
            collision_state.ledge_right = true;
        }
    }
}

// Convenience system for entities that need basic collision info without full movement
pub fn update_collision_state_only(
    spatial_query: SpatialQuery,
    mut query: Query<
        (Entity, &Transform, &mut CollisionState, &CollisionConfig),
        Without<Velocity>,
    >,
) {
    for (entity, transform, mut collision_state, config) in query.iter_mut() {
        let mut new_collision_state = CollisionState::default();
        update_static_collision_state(
            &spatial_query,
            entity,
            transform,
            &mut new_collision_state,
            config,
        );
        *collision_state = new_collision_state;
    }
}

// Helper functions for gameplay systems
impl CollisionState {
    pub fn can_jump(&self, config: &CollisionConfig) -> bool {
        self.grounded || self.time_since_grounded < config.coyote_time
    }

    pub fn can_wall_jump(&self) -> bool {
        (self.against_left_wall || self.against_right_wall) && !self.grounded
    }

    pub fn can_grab_ledge(&self) -> bool {
        (self.ledge_left || self.ledge_right) && !self.grounded
    }

    pub fn is_on_walkable_slope(&self, max_walkable_angle: f32) -> bool {
        self.on_slope && self.slope_angle.abs() < max_walkable_angle
    }
}

// Example plugin
pub struct KinematicCollisionPlugin;

impl Plugin for KinematicCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                kinematic_movement_with_collision_detection,
                update_collision_state_only,
            ),
        );
    }
}

// Helper function to spawn entities with collision detection
pub fn spawn_kinematic_entity(
    commands: &mut Commands,
    position: Vec2,
    size: Vec2,
    initial_velocity: Vec2,
    collision_config: Option<CollisionConfig>,
) -> Entity {
    commands
        .spawn((
            Transform::from_translation(position.extend(0.0)),
            RigidBody::Kinematic,
            Collider::rectangle(size.x, size.y),
            Velocity(initial_velocity),
            CollisionState::default(),
            collision_config.unwrap_or_default(),
        ))
        .id()
}

// Different presets for different entity types
pub fn player_collision_config() -> CollisionConfig {
    CollisionConfig {
        ground_check_distance: 3.0,
        wall_check_distance: 3.0,
        ceiling_check_distance: 3.0,
        ledge_check_distance: 12.0, // Larger for responsive ledge grabs
        slope_threshold: 0.5236,    // 30 degrees - more forgiving for player
        coyote_time: 0.15,          // Generous coyote time for players
    }
}

pub fn npc_collision_config() -> CollisionConfig {
    CollisionConfig {
        ground_check_distance: 2.0,
        wall_check_distance: 2.0,
        ceiling_check_distance: 1.0,
        ledge_check_distance: 6.0,
        slope_threshold: 0.7854, // 45 degrees
        coyote_time: 0.0,        // NPCs don't need coyote time
    }
}
