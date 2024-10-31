use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;

use crate::{health::{Death, Health}, weapon_system::auxiliary::weapon_target::WeaponTarget};

#[derive(Component)]
pub struct TargetRespawner {
    spawn_position: Vec3,
    respawn_delay: f32,
    last_death_time: Option<f32>,
    entity: Option<Entity>,
}

impl TargetRespawner {
    pub fn new(spawn_position: Vec3, respawn_delay: f32) -> TargetRespawner {
        TargetRespawner {
            spawn_position,
            respawn_delay,
            last_death_time: None,
            entity: None
        }
    }
}

#[derive(Component)]
pub struct HealthShrinker;

pub struct TargetSpawnerPlugin;

impl Plugin for TargetSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_respawning, handle_shrinking));
    }
}

fn handle_respawning(
    mut death_events: EventReader<Death>,
    mut respawners_query: Query<&mut TargetRespawner>,
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>
) {
    for mut respawner in respawners_query.iter_mut() {
        for event in death_events.read() {
            let Some(entity) = respawner.entity else { continue; };
            if event.entity == entity {
                respawner.last_death_time = Some(time.elapsed_seconds());
                respawner.entity = None;
            }
        }
        if respawner.entity.is_none() && respawner.last_death_time.is_none_or(|last_death| time.elapsed_seconds() > last_death + respawner.respawn_delay) {
            let id = commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid { half_size: Vec3::splat(0.5) }),
                    material: materials.add(StandardMaterial::from_color(Color::linear_rgb(1., 0., 0.))),
                    transform: Transform::from_translation(respawner.spawn_position),
                    ..Default::default()
                },
                Health::new(100),
                WeaponTarget,
                HealthShrinker,
            )).id();
            respawner.entity = Some(id);
        }
    }
}

fn handle_shrinking(
    mut shrinker_query: Query<(&mut Transform, &Health), Changed<Health>>
) {
    for (mut shrinker_transform, health) in shrinker_query.iter_mut() {
        shrinker_transform.scale = Vec3::splat(health.get_hp() as f32 / health.get_max_hp() as f32);
    }
}