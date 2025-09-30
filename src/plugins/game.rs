use bevy::prelude::*;

use crate::components::GameEntity;
use crate::states::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup)
            .add_systems(OnExit(GameState::Game), cleanup_game);
    }
}

fn setup(mut _commands: Commands) {}

fn cleanup_game(mut commands: Commands, game_query: Query<Entity, With<GameEntity>>) {
    for entity in &game_query {
        commands.entity(entity).despawn();
    }
}
