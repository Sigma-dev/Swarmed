use bevy::prelude::*;
use bevy_steam_p2p::{NetworkData, NetworkIdentity, NetworkedAction, SendFlags, SteamP2PClient};

use crate::weapon_system::{WeaponEvent, WeaponEventType, WeaponSystem};

#[derive(Component)]
pub struct NetworkedWeaponSystem;

pub struct NetworkedWeaponSystemPlugin;

impl Plugin for NetworkedWeaponSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (receive_packets, send_packets));
    }
}

fn receive_packets(
    mut networked_actions_reader: EventReader<NetworkedAction>,
    mut client: Res<SteamP2PClient>,
    mut weapon_events_writer: EventWriter<WeaponEvent>,
    networked_weapon_system_query: Query<(Entity, &NetworkIdentity, &WeaponSystem), With<NetworkedWeaponSystem>>
) {
    for event in networked_actions_reader.read() {
        if event.action_id == 0 {
            let Some(weapon_action_id) = event.action_data.first() else { continue; };
            for (system_entity, network_identity, weapon_system) in networked_weapon_system_query.iter() {
                if event.network_identity == *network_identity {
                    weapon_events_writer.send(WeaponEvent { event_type: WeaponEventType::from_index(*weapon_action_id), system_entity, weapon_index: weapon_system.inventory.get_equipped_index().unwrap()});
                }
            }
        }
    }
}

fn send_packets(
    mut client: Res<SteamP2PClient>,
    mut weapon_events_reader: EventReader<WeaponEvent>,
    networked_weapon_system_query: Query<(Entity, &NetworkIdentity, &NetworkedWeaponSystem), With<WeaponSystem>>
) {
    for event in weapon_events_reader.read() {
        for (entity, network_identity, networked_weapon_system) in networked_weapon_system_query.iter() {
            if event.system_entity != entity { continue; };
            let data = NetworkData::NetworkedAction(
                network_identity.clone(),
                0,
                vec![event.event_type.to_index()] 
            );
            println!("Data: {:?}", data);
            client.send_message_others(data, SendFlags::RELIABLE);
        }
    }
}