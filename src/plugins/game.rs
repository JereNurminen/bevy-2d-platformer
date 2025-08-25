use bevy::prelude::*;

use crate::bundles::PlatformBundle;
use crate::components::GameEntity;
use crate::constants::*;
use crate::states::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup_level)
            .add_systems(OnExit(GameState::Game), cleanup_game);
    }
}

fn setup_level(mut commands: Commands) {
    // Ground platform
    commands.spawn(PlatformBundle::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(times_phys_length_unit(20), times_phys_length_unit(2)),
        Color::linear_rgb(0.5, 0.5, 0.5),
    ));
}

fn cleanup_game(mut commands: Commands, game_query: Query<Entity, With<GameEntity>>) {
    for entity in &game_query {
        commands.entity(entity).despawn();
    }
}
