use bevy::prelude::*;
use bevy_steam_p2p::{NetworkData, NetworkIdentity, NetworkedAction, SendFlags, SteamP2PClient};

use crate::weapon_system::{WeaponEvent, WeaponEventType, WeaponSystem};

use super::{Health, HealthChange};

#[derive(Component)]
pub struct NetworkedHealth;

pub struct NetworkedHealthPlugin;

impl Plugin for NetworkedHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (receive_packets, send_packets));
    }
}

fn receive_packets(
    mut networked_actions_reader: EventReader<NetworkedAction>,
    mut weapon_events_writer: EventWriter<HealthChange>,
    networked_weapon_system_query: Query<(Entity, &NetworkIdentity), With<NetworkedHealth>>
) {
    for event in networked_actions_reader.read() {
        if event.action_id == 1 {
            let Some((change, new_health)) = (match event.action_data.as_slice() {
                [change, new_health, ..] => Some((change, new_health)),
                _ => None,
            }) else { continue; };
            for (entity, network_identity) in networked_weapon_system_query.iter() {
                if event.network_identity == *network_identity {
                    weapon_events_writer.send(HealthChange { entity, change: *change as i32, new_health: *new_health as i32 });
                }
            }
        }
    }
}

fn send_packets(
    client: Res<SteamP2PClient>,
    mut weapon_events_reader: EventReader<HealthChange>,
    networked_weapon_system_query: Query<(Entity, &NetworkIdentity), (With<Health>, With<NetworkedHealth>)>
) {
    for event in weapon_events_reader.read() {
        for (entity, network_identity) in networked_weapon_system_query.iter() {
            if event.entity != entity { continue; };
            let data = NetworkData::NetworkedAction(
                network_identity.clone(),
                1,
                vec![event.new_health] 
            );
            client.send_message_others(data, SendFlags::RELIABLE);
        }
    }
}