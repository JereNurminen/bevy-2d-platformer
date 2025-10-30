use bevy::prelude::*;

use crate::bundles::camera::{self, CameraBundle, MainCamera};
use crate::bundles::player::Player;
use crate::states::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera.run_if(in_state(GameState::Game)));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        CameraBundle::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
                viewport_height: 400.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
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

    let offset_y = 64.0;

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y + offset_y;
}
