use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkEntity;

use super::player::{Player, PlayerBundle};

#[derive(Component, Debug, Default)]
pub struct PlayerSpawnPoint;

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerSpawnBundle {
    player_spawn_point: PlayerSpawnPoint,
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct CurrentSpawn(pub Option<Transform>);

#[derive(Event, Default)]
pub struct RespawnPlayer;

#[derive(Event)]
pub struct SetSpawn(pub Transform);

#[derive(Resource, Clone)]
pub struct SpawnSettings {}
impl Default for SpawnSettings {
    fn default() -> Self {
        Self {}
    }
}

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSpawn>()
            .init_resource::<SpawnSettings>()
            .add_event::<RespawnPlayer>()
            .add_event::<SetSpawn>()
            // Run after your level/scene is loaded (Startup is fine for code-built levels).
            .add_systems(Update, (apply_set_spawn_events, handle_respawn));
    }
}

/// Update CurrentSpawn based on `SetSpawn` events.
fn apply_set_spawn_events(mut current: ResMut<CurrentSpawn>, mut ev_set: EventReader<SetSpawn>) {
    for SetSpawn(t) in ev_set.read() {
        current.0 = Some(*t);
    }
}

/// Despawn existing players and (re)spawn at CurrentSpawn on `RespawnPlayer`.
fn handle_respawn(
    mut commands: Commands,
    mut ev_respawn: EventReader<RespawnPlayer>,
    current: Res<CurrentSpawn>,
    mut player: Query<Entity, With<Player>>,
) {
    let mut should = false;
    for _ in ev_respawn.read() {
        should = true;
    }
    if !should {
        return;
    }

    if let Ok(player) = player.single_mut() {
        commands.entity(player).despawn();

        let at = match current.0 {
            Some(t) => t,
            None => {
                warn!("Respawn requested but no CurrentSpawn set; using origin.");
                Transform::default()
            }
        };

        commands.spawn(PlayerBundle {
            transform: at,
            ..Default::default()
        });
    }
}
