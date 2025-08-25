use bevy::prelude::*;

use crate::bundles::PlayerBundle;
use crate::components::{IsGrounded, JumpState, KinematicVelocity, Player, PlayerController};
use crate::states::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_player)
            .add_systems(
                Update,
                (
                    update_jump_timers,
                    kinematic_player_movement,
                    apply_kinematic_movement,
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::default());
}

fn update_jump_timers(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut JumpState, &IsGrounded, &PlayerController), With<Player>>,
) {
    for (mut jump_state, is_grounded, controller) in &mut query {
        let dt = time.delta_secs();

        // Update jump buffer timer
        if keyboard.just_pressed(KeyCode::Space)
            || keyboard.just_pressed(KeyCode::KeyW)
            || keyboard.just_pressed(KeyCode::ArrowUp)
        {
            jump_state.jump_buffer_timer = controller.jump_buffer_time;
        } else {
            jump_state.jump_buffer_timer = (jump_state.jump_buffer_timer - dt).max(0.0);
        }

        // Update coyote timer
        if is_grounded.0 {
            jump_state.coyote_timer = controller.coyote_time;
        } else if jump_state.was_grounded_last_frame {
            // Just left the ground, start coyote time
        } else {
            jump_state.coyote_timer = (jump_state.coyote_timer - dt).max(0.0);
        }

        jump_state.was_grounded_last_frame = is_grounded.0;
    }
}

pub fn kinematic_player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut KinematicVelocity,
            &mut JumpState,
            &PlayerController,
            &IsGrounded,
        ),
        With<Player>,
    >,
) {
    for (mut velocity, mut jump_state, controller, is_grounded) in &mut query {
        let dt = time.delta_secs();

        // Horizontal movement
        let mut move_x = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            move_x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            move_x += 1.0;
        }
        velocity.velocity.x = move_x * controller.move_speed;

        // Jump logic with buffer and coyote time
        let can_jump = is_grounded.0 || jump_state.coyote_timer > 0.0;
        let wants_to_jump = jump_state.jump_buffer_timer > 0.0;

        let just_jumped = can_jump && wants_to_jump;
        if just_jumped {
            velocity.velocity.y = controller.jump_force;
            jump_state.jump_buffer_timer = 0.0;
            jump_state.coyote_timer = 0.0;
            println!("JUMP! velocity.y: {}", velocity.velocity.y);
        }

        // Variable jump height - if player releases jump key early, reduce upward velocity
        let jump_key_pressed = keyboard.pressed(KeyCode::Space)
            || keyboard.pressed(KeyCode::KeyW)
            || keyboard.pressed(KeyCode::ArrowUp);

        if !jump_key_pressed && velocity.velocity.y > 0.0 {
            velocity.velocity.y *= 0.5;
        }

        // Apply gravity (but not on the frame we just jumped)
        if !is_grounded.0 && !just_jumped {
            velocity.velocity.y -= controller.gravity * dt;
            // Cap fall speed
            velocity.velocity.y = velocity.velocity.y.max(-controller.max_fall_speed);
        }
    }
}

pub fn apply_kinematic_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &KinematicVelocity), With<Player>>,
) {
    for (mut transform, velocity) in &mut query {
        let dt = time.delta_secs();
        let movement = velocity.velocity * dt;

        // Apply movement with small epsilon to prevent floating point precision issues
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}
