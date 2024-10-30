use bevy::{prelude::*, utils::HashMap};
use bevy_steam_p2p::{NetworkIdentity, SteamP2PClient};

use super::CharacterControllerSet;

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, (track_entity).in_set(CharacterControllerSet::CameraSync));
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