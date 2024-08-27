use std::path::Path;

use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, color::Color, input::ButtonInput, math::{Vec3, VectorSpace}, pbr::{PbrBundle, StandardMaterial}, prelude::{default, Commands, Component, Cuboid, EventReader, KeyCode, Mesh, Query, Res, ResMut, Resource, Transform, With}, scene::serde};
use bevy_steamworks::{Client, FriendFlags, GameLobbyJoinRequested, LobbyId, LobbyType, Manager, Matchmaking, SteamError, SteamId, SteamworksEvent, SteamworksPlugin};
use flume::{Receiver, Sender};
use ::serde::{Deserialize, Serialize};
use steamworks::networking_types::{NetworkingIdentity, SendFlags};
pub struct SteamNetworkPlugin;

impl Plugin for SteamNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .add_systems(Startup, steam_start)
        .add_systems(Update, (steam_system, steam_events, receive_messages));
    }
}

#[derive(Resource)]
pub struct NetworkClient {
    id: SteamId,
    lobby_status: LobbyStatus,
    steam_client: bevy_steamworks::Client,
    channel: LobbyIdCallbackChannel,
    network_id_counter: u32,
}

impl NetworkClient {
    pub fn create_lobby(&self) {
        let tx = self.channel.tx.clone();
        if self.lobby_status != LobbyStatus::OutOfLobby { return; };
        self.steam_client.matchmaking().create_lobby(LobbyType::Public, 2, 
            move |res| {
                if let Ok(lobby_id) = res {
                    match tx.send(lobby_id) {
                        Ok(_) => {}
                        Err(_) => {
                        }
                    }
                }
            });
    }
    pub fn join_lobby(&self, lobby_id: LobbyId) {
        let tx = self.channel.tx.clone();
        self.steam_client.matchmaking().join_lobby(lobby_id, 
            move |res| {
                if let Ok(lobby_id) = res {
                    match tx.send(lobby_id) {
                        Ok(_) => {}
                        Err(_) => {
                        }
                    }
                }
            });
    }
    pub fn leave_lobby(&mut self) {
        let LobbyStatus::InLobby(lobby) = self.lobby_status else {return; };
        println!("Leave");
        self.steam_client.matchmaking().leave_lobby(lobby);
        self.lobby_status = LobbyStatus::OutOfLobby;
    }
    pub fn send_message(&self, data: NetworkData, only_others: bool) {
        let LobbyStatus::InLobby(lobby_id) = self.lobby_status else { return; };
        for player in self.steam_client.matchmaking().lobby_members(lobby_id) {
            if only_others && player == self.id {
                continue;
            }
            println!("Sending to: {}", player.raw());
            let serialize_data = rmp_serde::to_vec(&data);
            let Ok(serialized) = serialize_data else {return;};
            let data_arr = serialized.as_slice();
            let network_identity = NetworkingIdentity::new_steam_id(player);
            let res = self.steam_client.networking_messages().send_message_to_user(network_identity, SendFlags::RELIABLE, &data_arr, 0);
            match res {
                Ok(_) => println!("Message sent succesfully"),
                Err(err) => println!("Message error: {}", err.to_string()),
            }
        }
    }
}

#[derive(PartialEq)]
enum LobbyStatus {
    InLobby(LobbyId),
    OutOfLobby
}

#[derive(Component, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NetworkId(pub u32);

#[derive(PartialEq)]
enum NetworkSync {
    Disabled,
    Enabled(f32),
}

#[derive(Component)]
struct NetworkedTransform {
    synced: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePath(pub u32);

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkData {
    Handshake,
    SendObjectData(NetworkId, i8, Vec<u8>), //NetworkId of receiver, id of action, data of action
    Instantiate(NetworkId, FilePath, Vec3), //NetworkId of created object, filepath of prefab, starting position
    PositionUpdate(NetworkId, Vec3), //NetworkId of receiver, new position
    Destroy(NetworkId), //NetworkId of object to be destroyed
}

pub struct LobbyIdCallbackChannel {
    pub tx: Sender<LobbyId>,
    pub rx: Receiver<LobbyId>
}

fn lobby_joined(client: &mut ResMut<NetworkClient>, lobby: LobbyId) {
    println!("Lobby joined: {}", lobby.raw());
   // client.lobby_status = LobbyStatus::InLobby(lobby)
    //client.send_message(NetworkData::Handshake, true);
}

/* 
fn send_message(steam_client: &Res<Client>, lobby_id: LobbyId, data: NetworkData) {
    println!("Send");
    for player in steam_client.matchmaking().lobby_members(lobby_id) {
        println!("Id: {}", player.raw());
        let serialize_data = rmp_serde::to_vec(&data);
        let Ok(serialized) = serialize_data else {return;};
        let data_arr = serialized.as_slice();
        let network_identity = NetworkingIdentity::new_steam_id(player);
        let res = steam_client.networking_messages().send_message_to_user(network_identity, SendFlags::RELIABLE, &data_arr, 0);
        match res {
            Ok(_) => println!("Message sent succesfully"),
            Err(err) => println!("Message error: {}", err.to_string()),
        }
    }
}*/

fn instantiate(
    network_id: NetworkId,
    path: FilePath,
    pos: Vec3,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    println!("Instantiation");
    if (path.0 == 0) {
        commands.spawn((
            PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_translation(pos),
            ..default()
            },
            NetworkId::from(network_id),
            NetworkedTransform{synced: true}
        ));
    }
}

fn handle_networked_transform(
    client: Res<NetworkClient>,
    networked_transform_query: Query<(&Transform, &NetworkId, &NetworkedTransform),>
) {
    for (transform, network_id, networked_transform) in networked_transform_query.iter() {
        if !networked_transform.synced { continue; };
        client.send_message(NetworkData::PositionUpdate(*network_id, transform.translation), true)
    }
}

fn receive_messages(
    client: Res<NetworkClient>, 
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let messages: Vec<steamworks::networking_types::NetworkingMessage<steamworks::ClientManager>> = client.steam_client.networking_messages().receive_messages_on_channel(0, 1);
    if (messages.len() > 0 ) {
        println!("Received {} messages", messages.len())
    }
    for message in messages {
        let serialized_data = message.data();
        let data_try: Result<NetworkData, _> = rmp_serde::from_slice(serialized_data);
        match data_try {
            Ok(data) => match data {
                NetworkData::SendObjectData(id, action_id, action_data) => println!("Action"),
                NetworkData::Instantiate(id, prefab_path, pos) => instantiate(id, prefab_path, pos, &mut commands, &mut meshes, &mut materials),
                NetworkData::PositionUpdate(id, pos) => println!("Position updated {}", pos),
                NetworkData::Destroy(id) => println!("Destroyed"),
                NetworkData::Handshake => {
                    println!("Received handshake, sending response");
                    //client.send_message(NetworkData::Handshake, true);
                },
            },
            Err(err) => println!("{}", err.to_string())
        }
        drop(message); //not sure about usefullness, mentionned in steam docs as release
    }
}

fn steam_system(
    steam_client: Res<Client>,
    keys: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<NetworkClient>,
    //channel: Res<LobbyIdCallbackChannel>
) { 
    //let (tx, rx): (Sender<LobbyId>, Receiver<LobbyId>) = (channel.tx.clone(), channel.rx.clone());
    let rx = client.channel.rx.clone();
    /*
    if keys.just_pressed(KeyCode::KeyC) {
        if client.lobby_status != LobbyStatus::OutOfLobby { return; };
        steam_client.matchmaking().create_lobby(LobbyType::Public, 2, move |res| {
            if let Ok(lobby_id) = res {
                println!("a {}", res.unwrap().raw());
                match tx.send(lobby_id) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("send err")
                    }
                }
            }
        });
    }
    else if (keys.just_pressed(KeyCode::KeyV)) {
        let LobbyStatus::InLobby(lobby) = client.lobby_status else {return; };
        println!("Leave");
        client.lobby_status = LobbyStatus::OutOfLobby;
        steam_client.matchmaking().leave_lobby(lobby)
    }
    else if (keys.just_pressed(KeyCode::KeyT)) {
        if let LobbyStatus::InLobby(lobby_id) = client.lobby_status {
            send_message(&steam_client, lobby_id, NetworkData::PositionUpdate(NetworkId(0), Vec3 {x:1., y:2., z: 3.}));
        }
    }
    */
    if let Ok(lobby_id) = rx.try_recv() {
        //game_state.set(ClientState::InLobby);
        client.lobby_status = LobbyStatus::InLobby(lobby_id);
        println!("Joined Lobby: {}", lobby_id.raw());
        //client.send_message(NetworkData::Handshake, true);
    }
}

fn steam_start(
    steam_client: Res<Client>,
    mut commands: Commands,
) {
    println!("Connected: {}", steam_client.user().steam_id().raw());
    let (tx, rx) = flume::unbounded();

    commands.insert_resource(NetworkClient {
        id: steam_client.user().steam_id(),
        lobby_status: LobbyStatus::OutOfLobby,
        steam_client: steam_client.clone(),
        channel: LobbyIdCallbackChannel { tx, rx },
        network_id_counter: 0,
    });
}

fn steam_events(
    steam_client: Res<Client>,
    mut evs: EventReader<SteamworksEvent>,
    mut client: ResMut<NetworkClient>,
    //channel: Res<LobbyIdCallbackChannel>
) {
    for ev in evs.read() {
        //println!("EV");
        match ev {
            SteamworksEvent::GameLobbyJoinRequested(info) => {
                println!("Trying to join: {}", info.lobby_steam_id.raw());
                client.join_lobby(info.lobby_steam_id)
            },
            SteamworksEvent::LobbyChatUpdate(info) => {
                println!("Chat Update");
                match info.member_state_change {
                    bevy_steamworks::ChatMemberStateChange::Entered => lobby_joined(&mut client, info.lobby),
                    bevy_steamworks::ChatMemberStateChange::Left => client.lobby_status = LobbyStatus::OutOfLobby,
                    bevy_steamworks::ChatMemberStateChange::Disconnected => client.lobby_status = LobbyStatus::OutOfLobby,
                    _ => println!("other")
                }
            },
            SteamworksEvent::SteamServersConnected(_) => println!("Connected to steam servers!"),
            SteamworksEvent::AuthSessionTicketResponse(_) => println!("Ticket response"),
            SteamworksEvent::DownloadItemResult(_) => println!("Download item result"),
            SteamworksEvent::P2PSessionConnectFail(_) => println!("P2P Fail"),
            SteamworksEvent::P2PSessionRequest(_) => println!("P2P Session request"),
            SteamworksEvent::PersonaStateChange(persona) => println!("Persona {}: {}", persona.steam_id.raw(), persona.flags.bits()),
            SteamworksEvent::SteamServerConnectFailure(_) => println!("Connection failed"),
            SteamworksEvent::SteamServersDisconnected(_) => println!("Disconnected"),
            SteamworksEvent::TicketForWebApiResponse(_) => println!("Ticket"),
            SteamworksEvent::UserAchievementStored(_) => println!("Achievement stored"),
            SteamworksEvent::UserStatsReceived(_) => println!("UserStatsReceived"),
            SteamworksEvent::UserStatsStored(_) => println!("User stats stored"),
            SteamworksEvent::ValidateAuthTicketResponse(_) => println!("Validate auth ticket"),
            SteamworksEvent::NetworkingMessagesSessionRequest(_) => println!("Auth requested"),
        }
       /*  if let SteamworksEvent::GameLobbyJoinRequested(info) = ev {
            println!("Trying to join: {}", info.lobby_steam_id.raw());
            steam_client.matchmaking().join_lobby(info.lobby_steam_id, |cb| if let Ok(lobby_id) = cb {client.current_lobby = Some(lobby_id)});
        }*/
    }
}