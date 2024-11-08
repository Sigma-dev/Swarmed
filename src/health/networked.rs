use bevy::prelude::*;
use bevy_steam_p2p::{NetworkData, NetworkIdentity, NetworkedAction, SendFlags, SteamP2PClient};
use ::serde::{Deserialize, Serialize};
use super::{Health, HealthChange};

#[derive(Component)]
pub struct NetworkedHealth;

pub struct NetworkedHealthPlugin;

impl Plugin for NetworkedHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (receive_packets, send_packets));
    }
}

#[derive(Serialize, Deserialize)]
struct NetworkedChange {
    pub change: i32,
    pub new_health: i32,
}

fn receive_packets(
    mut networked_actions_reader: EventReader<NetworkedAction>,
    mut networked_weapon_system_query: Query<(&NetworkIdentity, &mut Health), With<NetworkedHealth>>
) {
    for event in networked_actions_reader.read() {
        println!("B");
        if event.action_id == 1 {
            println!("C");
            let change: NetworkedChange = rmp_serde::from_slice(&event.action_data).unwrap();
            for (network_identity, mut health) in networked_weapon_system_query.iter_mut() {
                println!("D");
                if event.network_identity == *network_identity {
                    println!("E");
                    health.change(change.change, false);
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
        if !event.authentic { continue; };
        for (entity, network_identity) in networked_weapon_system_query.iter() {
            if event.entity != entity { continue; };
            let data = NetworkData::NetworkedAction(
                network_identity.clone(),
                1,
                rmp_serde::to_vec(&NetworkedChange { change: event.change, new_health: event.new_health }).unwrap()
            );
            client.send_message_others(data, SendFlags::RELIABLE).expect("Failed to send health message");
        }
    }
}