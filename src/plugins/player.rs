use std::{collections::HashMap, time::Duration};

use bevy::{prelude::*, time::Stopwatch};

use avian2d::prelude::*;

use bevy_inspector_egui::InspectorOptions;
use leafwing_input_manager::{
    Actionlike,
    prelude::{ActionState, InputMap},
};

use crate::{
    bundles::player::Player,
    constants::{GameLayer, PLAYER_HEIGHT, PLAYER_WIDTH, multiply_by_tile_size},
};

/// Represents a rectangular bounds with position and dimensions
struct BoundsRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl BoundsRect {
    fn from_aseprite_rect(rect: &crate::aseprite_deserialize::Rect) -> Self {
        Self {
            x: rect.x as f32,
            y: rect.y as f32,
            width: rect.w as f32,
            height: rect.h as f32,
        }
    }
}

/// Calculate offset from sprite center for a given bounds rectangle
///
/// Aseprite uses top-left origin, but Bevy sprites are centered.
/// This converts bounds to an offset from the sprite's center point.
///
/// # Arguments
/// * `bounds` - The rectangular bounds to convert
/// * `sprite_width` - Total width of the sprite
/// * `sprite_height` - Total height of the sprite
/// * `flip_x` - Whether to flip the x-axis (for sprite mirroring)
///
/// # Returns
/// Vec2 offset from sprite center
fn calculate_sprite_offset(
    bounds: &BoundsRect,
    sprite_width: f32,
    sprite_height: f32,
    flip_x: bool,
) -> Vec2 {
    // Calculate center point of the bounds
    let bounds_center_x = bounds.x + bounds.width / 2.0;
    let bounds_center_y = bounds.y + bounds.height / 2.0;

    // Calculate sprite center
    let sprite_center_x = sprite_width / 2.0;
    let sprite_center_y = sprite_height / 2.0;

    // Calculate offset from sprite center
    // X: positive means right of center
    let offset_x = bounds_center_x - sprite_center_x;
    // Y: Bevy uses bottom-up coordinates, Aseprite uses top-down
    let offset_y = sprite_center_y - bounds_center_y;

    // Apply horizontal flip if needed
    let final_x = if flip_x { -offset_x } else { offset_x };

    Vec2::new(final_x, offset_y)
}

/// Get sprite dimensions for the player
/// This should match the actual sprite dimensions in the asset
const PLAYER_SPRITE_WIDTH: f32 = 64.0;
const PLAYER_SPRITE_HEIGHT: f32 = 64.0;

use super::{
    animation::{AnimationKey, AnimationPlugin, CurrentAnimation, NextAnimation},
    animation_library::{AnimationConfig, AnimationLibrary},
    collision::{CollisionBundle, CollisionConfig, GroundedStopwatch, IsGrounded, Velocity},
    gravity::EntityGravity,
    projectile::{ProjectileSpawnEvent, ProjectileVelocity},
};

#[derive(Event)]
pub struct PlayerSpawnEvent(pub Transform);

#[derive(Event)]
pub struct PlayerShootEvent;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Left,
    Right,
    Jump,
    Shoot,
}

#[derive(Component, Default, Reflect, Resource, InspectorOptions)]
pub struct BarrelPosition(pub Vec2);

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

    let Some(player_anim_data) = &animation_library.player else {
        return;
    };

    if let Some(event) = event_reader.read().last() {
        let input_map = InputMap::new([
            (PlayerAction::Jump, KeyCode::Space),
            (PlayerAction::Left, KeyCode::ArrowLeft),
            (PlayerAction::Left, KeyCode::KeyA),
            (PlayerAction::Right, KeyCode::ArrowRight),
            (PlayerAction::Right, KeyCode::KeyD),
            (PlayerAction::Shoot, KeyCode::KeyJ),
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
        let (hitbox_width, hitbox_height, hitbox_offset) = player_anim_data
            .slices
            .iter()
            .find(|s| s.name == "hitbox")
            .and_then(|s| s.keys.first())
            .map(|key| {
                let bounds = BoundsRect::from_aseprite_rect(&key.bounds);
                let offset = calculate_sprite_offset(
                    &bounds,
                    PLAYER_SPRITE_WIDTH,
                    PLAYER_SPRITE_HEIGHT,
                    false, // No flip for initial setup
                );
                (bounds.width, bounds.height, offset)
            })
            .unwrap_or((PLAYER_WIDTH, PLAYER_HEIGHT, Vec2::ZERO));

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
                    Transform::from_xyz(hitbox_offset.x, hitbox_offset.y, 0.0),
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
                BarrelPosition::default(),
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

fn apply_controls(
    action_state: Single<&ActionState<PlayerAction>, With<Player>>,
    mut event_writer: EventWriter<PlayerShootEvent>,
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

        if action_state.just_pressed(&PlayerAction::Shoot) {
            println!("Player shot!");
            event_writer.write(PlayerShootEvent {});
        }

        velocity.0 += direction;

        match (is_grounded.0, just_jumped, is_running) {
            (false, _, _) | (true, true, _) => {
                next_animation.key = Some(PlayerAnimations::Jump);
            }
            (true, false, true) => {
                next_animation.key = Some(PlayerAnimations::Run);
            }
            (true, false, false) => {
                next_animation.key = Some(PlayerAnimations::Idle);
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

fn update_animated_components(
    mut query: Query<(&Sprite, &mut BarrelPosition)>,
    animation_library: Res<AnimationLibrary>,
) {
    let Some(player_anim_data) = &animation_library.player else {
        return;
    };

    for (sprite, mut barrel_position) in query.iter_mut() {
        if let Some(barrel_positions_for_frames) = player_anim_data.slice_map.get("gun_barrel")
            && let Some(ref atlas) = sprite.texture_atlas
        {
            if let Some(frame) = barrel_positions_for_frames
                .keys
                .iter()
                .find(|&frame| frame.frame == atlas.index)
            {
                let bounds = BoundsRect::from_aseprite_rect(&frame.bounds);
                barrel_position.0 = calculate_sprite_offset(
                    &bounds,
                    PLAYER_SPRITE_WIDTH,
                    PLAYER_SPRITE_HEIGHT,
                    sprite.flip_x,
                );
            }
        }
    }
}

fn shoot(
    mut query: Query<(&BarrelPosition, &Transform, &Sprite, &WalkSpeed), With<Player>>,
    mut event_reader: EventReader<PlayerShootEvent>,
    mut event_writer: EventWriter<ProjectileSpawnEvent>,
    asset_server: Res<AssetServer>,
) {
    if let Some(_) = event_reader.read().last() {
        if let Some((barrel_position, player_transform, sprite, walk_speed)) =
            query.iter_mut().last()
        {
            println!("Player shoot event triggered!");
            let bullet_dir = if sprite.flip_x { -1.0 } else { 1.0 };
            let bullet_speed = (walk_speed.0 + 70.0) * bullet_dir;

            let world_position = player_transform.translation.xy() + barrel_position.0;
            event_writer.write(ProjectileSpawnEvent {
                transform: Transform::from_translation(world_position.extend(0.0)),
                velocity: ProjectileVelocity(Vec2::new(bullet_speed, 0.0)),
                sprite: asset_server.load("sprites/bullet.png"),
            });
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerShootEvent>()
            .add_systems(
                Update,
                (
                    spawn_player,
                    apply_controls,
                    toggle_gravity,
                    //debug_player_colors,
                    update_animated_components,
                    shoot,
                ),
            )
            .add_plugins(AnimationPlugin::<PlayerAnimations>::default());
    }
}
