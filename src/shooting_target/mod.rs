use bevy::prelude::*;
use bevy_steam_p2p::{InstantiationData, SteamP2PClient};

use crate::{health::{networked::NetworkedHealth, Death, Health}, weapon_system::auxiliary::weapon_target::WeaponTarget};

#[derive(Component)]
pub struct ShootingTarget {
    respawn_delay: f32,
    last_death_time: Option<f32>,
}

impl ShootingTarget {
    pub fn new(respawn_delay: f32) -> ShootingTarget {
        ShootingTarget {
            respawn_delay,
            last_death_time: None,
        }
    }
}

pub struct ShootingTargetPlugin;

impl Plugin for ShootingTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_deaths, handle_respawning, handle_shrinking));
    }
}

fn handle_deaths(
    mut death_events: EventReader<Death>,
    mut shrinker_query: Query<&mut ShootingTarget>,
    time: Res<Time>
) {
    for death in death_events.read() {
        let Ok(mut shooting_target) = shrinker_query.get_mut(death.entity) else { continue; };
        shooting_target.last_death_time = Some(time.elapsed_seconds());
    }
}

fn handle_respawning(
    mut target_query: Query<(&mut Transform, &mut Health, &mut ShootingTarget)>,
    time: Res<Time>
) {
    for (_, mut health, mut target) in target_query.iter_mut() {
        if target.last_death_time.is_some_and(|last_death| time.elapsed_seconds() > last_death + target.respawn_delay) {
            health.reset(true);
            target.last_death_time = None;
        }
    }
}

fn handle_shrinking(
    mut shrinker_query: Query<(&mut Transform, &Health), (With<ShootingTarget>, Changed<Health>)>
) {
    for (mut shrinker_transform, health) in shrinker_query.iter_mut() {
        shrinker_transform.scale = Vec3::splat((health.get_hp() as f32 / health.get_max_hp() as f32).max(0.));
    }
}

pub fn spawn_shooting_target(
    client: &mut ResMut<SteamP2PClient>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    instantiation_data: InstantiationData,
) {
    let owner_id = instantiation_data.network_identity.owner_id;
    commands.spawn((
        instantiation_data.network_identity,
        PbrBundle {
            mesh: meshes.add(Cuboid { half_size: Vec3::splat(0.5) }),
            material: materials.add(StandardMaterial::from_color(Color::linear_rgb(1., 0., 0.))),
            transform: Transform::from_translation(instantiation_data.starting_pos),
            ..Default::default()
        },
        Health::new(100, false),
        NetworkedHealth,
        WeaponTarget,
        ShootingTarget::new(2.)
    ));
    if client.id == owner_id {
    }
}