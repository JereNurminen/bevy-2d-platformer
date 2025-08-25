use bevy::prelude::*;

use crate::bundles::CameraBundle;
use crate::components::{MainCamera, Player};
use crate::states::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera.run_if(in_state(GameState::Game)));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(CameraBundle::default());
}

fn update_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let Some(mut camera_transform) = camera_query.iter_mut().next() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
