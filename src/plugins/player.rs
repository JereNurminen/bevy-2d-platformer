use std::{collections::HashMap, time::Duration};

use bevy::{prelude::*, time::Stopwatch};

use avian2d::prelude::*;

use leafwing_input_manager::{
    Actionlike,
    prelude::{ActionState, InputMap},
};

use crate::{
    bundles::player::Player,
    constants::{GameLayer, PLAYER_HEIGHT, PLAYER_WIDTH, multiply_by_tile_size},
};

use super::{
    animation::{AnimationKey, AnimationPlugin, CurrentAnimation, NextAnimation},
    animation_library::{AnimationConfig, AnimationLibrary},
    collision::{CollisionBundle, CollisionConfig, GroundedStopwatch, IsGrounded, Velocity},
    gravity::EntityGravity,
};

#[derive(Event)]
pub struct PlayerSpawnEvent(pub Transform);

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Left,
    Right,
    Jump,
}

#[derive(Component, Default)]
pub struct AfterJumpGravityImmunityTimer(pub Timer);

#[derive(Component, Default)]
pub struct JumpForce(pub f32);

#[derive(Component, Default)]
pub struct WalkSpeed(pub f32);

#[derive(Component, Default)]
pub struct WalkAcceleration(pub f32);

#[derive(Component, Default)]
pub struct GroundDeceleration(pub f32);

#[derive(Component, Default)]
pub struct CoyoteTime(pub Duration);

#[derive(Component, Default)]
pub struct JumpCooldownTimer(pub Timer);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PlayerAnimations {
    Idle,
    Run,
    Jump,
}
impl AnimationKey for PlayerAnimations {}

pub fn spawn_player(
    mut event_reader: EventReader<PlayerSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    animation_library: Res<AnimationLibrary>,
) {
    let walk_speed = multiply_by_tile_size(10);
    let walk_acceleration = walk_speed * 2.5;
    let walk_deceleration = walk_acceleration * 2.0;

    let jump_force = multiply_by_tile_size(15);
    let gravity = multiply_by_tile_size(30);
    let max_fall_speed = multiply_by_tile_size(15);
    let gravity_immunity_duration = Duration::from_millis(300);

    // Check if animation library is ready
    let Some(player_anim_data) = &animation_library.player else {
        // Animation data not loaded yet, skip spawning
        return;
    };

    if let Some(event) = event_reader.read().last() {
        let input_map = InputMap::new([
            (PlayerAction::Jump, KeyCode::Space),
            (PlayerAction::Left, KeyCode::ArrowLeft),
            (PlayerAction::Left, KeyCode::KeyA),
            (PlayerAction::Right, KeyCode::ArrowRight),
            (PlayerAction::Right, KeyCode::KeyD),
        ]);

        // Configure player animations
        let animation_configs = HashMap::from([
            (PlayerAnimations::Idle, AnimationConfig::looping("idle")),
            (PlayerAnimations::Run, AnimationConfig::looping("run")),
            (PlayerAnimations::Jump, AnimationConfig::once("jump")),
        ]);

        let animations = AnimationLibrary::create_animation_bundle(
            player_anim_data,
            "sprites/player.png",
            animation_configs,
            PlayerAnimations::Idle,
            &asset_server,
            &mut texture_atlas_layouts,
        );

        // Get hitbox dimensions and offset from the slice data
        let (hitbox_width, hitbox_height, hitbox_offset_x, hitbox_offset_y) = player_anim_data
            .slices
            .iter()
            .find(|s| s.name == "hitbox")
            .and_then(|s| s.keys.first())
            .map(|key| {
                let bounds = &key.bounds;
                // Convert from pixel coordinates to game world units
                // The bounds are relative to the sprite, with origin at top-left
                // We need to center the hitbox, so calculate offset from sprite center
                let sprite_width = 64.0; // from the sprite source size
                let sprite_height = 64.0;

                let width = bounds.w as f32;
                let height = bounds.h as f32;

                // Calculate center offset from sprite center
                // Aseprite coords: (0,0) is top-left, Y increases downward
                // Bevy coords: (0,0) is center, Y increases upward
                let offset_x = (bounds.x as f32 + width / 2.0) - (sprite_width / 2.0);
                let offset_y = (sprite_height / 2.0) - (bounds.y as f32 + height / 2.0);

                (width, height, offset_x, offset_y)
            })
            .unwrap_or((PLAYER_WIDTH, PLAYER_HEIGHT, 0.0, 0.0));

        commands
            .spawn((
                Player,
                animations,
                event.0,
                RigidBody::Kinematic,
                LockedAxes::ROTATION_LOCKED,
            ))
            .with_children(|children| {
                children.spawn((
                    Collider::rectangle(hitbox_width, hitbox_height),
                    Transform::from_xyz(hitbox_offset_x, hitbox_offset_y, 0.0),
                ));
            })
            .insert(CollisionBundle {
                grounded_stopwatch: GroundedStopwatch(Stopwatch::new()),
                config: CollisionConfig {
                    ground_check_distance: 1.0,
                    wall_check_distance: 1.0,
                    ceiling_check_distance: 1.0,
                    collision_filter: SpatialQueryFilter::from_mask(
                        GameLayer::LevelGeometry.to_bits(),
                    ),
                },
                ..Default::default()
            })
            .insert(CollisionLayers::new(
                GameLayer::Player,
                [GameLayer::LevelGeometry, GameLayer::Default],
            ))
            .insert((
                EntityGravity {
                    gravity,
                    max_fall_speed,
                    enabled: true,
                },
                CoyoteTime(Duration::from_millis(500)),
                AfterJumpGravityImmunityTimer(Timer::new(
                    gravity_immunity_duration,
                    TimerMode::Once,
                )),
                JumpCooldownTimer(Timer::new(Duration::from_millis(500), TimerMode::Once)),
                JumpForce(jump_force),
                WalkSpeed(walk_speed),
                WalkAcceleration(walk_acceleration),
                GroundDeceleration(walk_deceleration),
                input_map,
            ));
    }
}

pub fn toggle_gravity(
    action_state: Single<&ActionState<PlayerAction>, With<Player>>,
    mut query: Query<(&mut EntityGravity, &mut AfterJumpGravityImmunityTimer)>,
    time: Res<Time>,
) {
    for (mut entity_gravity, mut gravity_immunity_timer) in query.iter_mut() {
        gravity_immunity_timer.0.tick(time.delta());
        if gravity_immunity_timer.0.finished() || !action_state.pressed(&PlayerAction::Jump) {
            entity_gravity.enabled = true;
        } else {
            entity_gravity.enabled = false;
        }
    }
}

pub fn apply_controls(
    action_state: Single<&ActionState<PlayerAction>, With<Player>>,
    mut query: Query<
        (
            &mut Velocity,
            &IsGrounded,
            &mut AfterJumpGravityImmunityTimer,
            &GroundedStopwatch,
            &CoyoteTime,
            &JumpForce,
            &WalkSpeed,
            &WalkAcceleration,
            &GroundDeceleration,
            &mut JumpCooldownTimer,
            &mut Sprite,
            &mut NextAnimation<PlayerAnimations>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (
        mut velocity,
        is_grounded,
        mut after_jump_gravity_immunity_timer,
        grounded_stopwatch,
        coyote_time,
        jump_force,
        walk_speed,
        walk_acceleration,
        ground_deceleration,
        mut jump_cooldown_timer,
        mut sprite,
        mut next_animation,
    ) in query.iter_mut()
    {
        let mut direction = Vec2::ZERO;

        jump_cooldown_timer.0.tick(time.delta());

        let mut is_running = false;
        let mut just_jumped = false;

        if action_state.pressed(&PlayerAction::Left) {
            if velocity.0.x > -walk_speed.0 {
                direction.x = -walk_acceleration.0 * time.delta_secs();
            }
            sprite.flip_x = true;
            is_running = true;
        } else if action_state.pressed(&PlayerAction::Right) {
            if velocity.0.x < walk_speed.0 {
                direction.x = walk_acceleration.0 * time.delta_secs();
            }
            sprite.flip_x = false;
            is_running = true;
        } else {
            // Moving left but not holding left
            if velocity.0.x < 0.0 {
                direction.x = (ground_deceleration.0 * time.delta_secs())
                    .clamp(velocity.0.x, ground_deceleration.0);
            // Moving right but not holding right
            } else if velocity.0.x > 0.0 {
                direction.x = (-ground_deceleration.0 * time.delta_secs())
                    .clamp(-ground_deceleration.0, velocity.0.x);
            } else {
                direction.x = 0.0;
            }
        }

        if action_state.pressed(&PlayerAction::Jump) {
            if is_grounded.0
                || grounded_stopwatch.0.elapsed() < coyote_time.0
                    && jump_cooldown_timer.0.finished()
            {
                direction.y += jump_force.0;
                after_jump_gravity_immunity_timer.0.reset();
                jump_cooldown_timer.0.reset();
                just_jumped = true;
            } else {
            }
        }

        velocity.0 += direction;

        match (is_grounded.0, just_jumped, is_running) {
            (false, _, _) | (true, true, _) => {
                next_animation.key = Some(PlayerAnimations::Jump);
                println!("Jump animation triggered");
            }
            (true, false, true) => {
                next_animation.key = Some(PlayerAnimations::Run);
                println!("Run animation triggered");
            }
            (true, false, false) => {
                next_animation.key = Some(PlayerAnimations::Idle);
                println!("Idle animation triggered");
            }
        }
    }
}

fn debug_player_colors(mut query: Query<(&mut Sprite, &IsGrounded)>) {
    for (mut sprite, is_grounded) in query.iter_mut() {
        if is_grounded.0 {
            sprite.color = Color::srgb(0.3, 0.7, 0.3);
        } else {
            sprite.color = Color::srgb(0.7, 0.3, 0.3);
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_systems(
                Update,
                (
                    spawn_player,
                    apply_controls,
                    toggle_gravity,
                    //debug_player_colors,
                ),
            )
            .add_plugins(AnimationPlugin::<PlayerAnimations>::default());
    }
}
