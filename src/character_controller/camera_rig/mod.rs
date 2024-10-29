use bevy::{prelude::*, utils::HashMap};
use bevy_steam_p2p::{NetworkIdentity, SteamP2PClient};

use crate::weapon_system::{gltf::{WeaponVisualsGltf, WeaponVisualsManagerGltf}, WeaponSystem};

use super::CharacterControllerSet;

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, (track_entity).in_set(CharacterControllerSet::CameraSync))
        .add_systems(Startup, create_camera);
}

// Specifies that this is the primary camera and should be used for the main view
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct RiggedCamera;

// Specifies the entity that we are attached to, as well as the offset from that entity
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct TrackedEntity(pub Vec3);

pub fn track_entity(
    mut query: Query<(&TrackedEntity, &mut Transform, &NetworkIdentity), Without<RiggedCamera>>,
    mut camera_query: Query<&mut Transform, With<RiggedCamera>>,
    client: Res<SteamP2PClient>
) {
    // There should only ever be one tracked entity and one rigged camera.
    for (tracked_entity, tracked_transform, network_identity) in query.iter_mut() {
        if network_identity.owner_id != client.id { continue; };
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation = tracked_entity.0 + tracked_transform.translation;
        }
    }
}

pub fn create_camera(
    mut commands: Commands,
    mut client: ResMut<SteamP2PClient>,
    weapon_system_query: Query<(Entity, &NetworkIdentity), With<WeaponSystem>>
) {
    return;
    /* let mut match_list = HashMap::new();
    match_list.insert("glock".to_string(), "weapons/glock/glock.glb".to_string());
    let mut maybe_system = None;
    for (system, identity) in weapon_system_query.iter() {
        println!("Yes");
        if identity.owner_id == client.id {
            maybe_system = Some(system);
        }
    }
  //  let Some(system_entity) = maybe_system else { return };
    commands.spawn((
        RiggedCamera,
        Camera3dBundle {
            // Adjust our rotation so we're looking backwards on spawn
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)).with_scale(Vec3::ONE * 15.),
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.384, 0.71, 0.949)),
                ..Default::default()
            },
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.01,
                ..default()
            }),
            ..Default::default()
        },
        WeaponVisualsManagerGltf {
            match_list,
         //   system: system_entity,
        },
    )); */
}
