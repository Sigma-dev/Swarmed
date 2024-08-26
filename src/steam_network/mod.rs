use std::path::Path;

use bevy::{app::{App, Plugin, Startup, Update}, input::ButtonInput, math::{Vec3, VectorSpace}, prelude::{Commands, Component, EventReader, KeyCode, Res, ResMut, Resource, Transform}, scene::serde};
use bevy_steamworks::{Client, FriendFlags, GameLobbyJoinRequested, LobbyId, LobbyType, Manager, Matchmaking, SteamError, SteamId, SteamworksEvent, SteamworksPlugin};
use flume::{Receiver, Sender};
use ::serde::{Deserialize, Serialize};
use steamworks::networking_types::{NetworkingIdentity, SendFlags};
pub struct SteamNetworkPlugin;

pub impl Plugin for SteamNetworkPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = flume::unbounded();
        app
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .insert_resource(LobbyIdCallbackChannel {
            tx, rx
        })
        .add_systems(Startup, steam_start)
        .add_systems(Update, (steam_system, steam_events, receive_messages));
    }
}

#[derive(Resource)]
pub struct NetworkClient {
    id: SteamId,
    lobby_status: LobbyStatus,
    steam_client: bevy_steamworks::Client
}

impl NetworkClient {
    fn create_lobby(&self, channel: &Res<LobbyIdCallbackChannel>) {
        let (tx, rx): (Sender<LobbyId>, Receiver<LobbyId>) = (channel.tx.clone(), channel.rx.clone());
        if self.lobby_status != LobbyStatus::OutOfLobby { return; };
        self.steam_client.matchmaking().create_lobby(LobbyType::Public, 2, move |res| {
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
}

#[derive(PartialEq)]
enum LobbyStatus {
    InLobby(LobbyId),
    OutOfLobby
}

#[derive(Component, Serialize, Deserialize, Debug)]
struct NetworkId(u32);

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
struct FilePath(u32);

#[derive(Serialize, Deserialize, Debug)]
enum NetworkData {
    SendObjectData(NetworkId, i8, Vec<u8>), //NetworkId of receiver, id of action, data of action
    Instantiate(NetworkId, FilePath, Vec3), //NetworkId of created object, filepath of prefab, starting position
    PositionUpdate(NetworkId, Vec3), //NetworkId of receiver, new position
    Destroy(NetworkId), //NetworkId of object to be destroyed
}

#[derive(Resource)]
struct LobbyIdCallbackChannel {
    tx: Sender<LobbyId>,
    rx: Receiver<LobbyId>
}

fn lobby_joined(client: &mut ResMut<NetworkClient>, lobby: LobbyId) {
    println!("Lobby joined: {}", lobby.raw());
    client.lobby_status = LobbyStatus::InLobby(lobby)
}

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
}

fn receive_messages(steam_client: Res<Client>) {
    let messages: Vec<steamworks::networking_types::NetworkingMessage<steamworks::ClientManager>> = steam_client.networking_messages().receive_messages_on_channel(0, 1);
    if (messages.len() > 0 ) {
        println!("Received {} messages", messages.len())
    }
    for message in messages {
        let serialized_data = message.data();
        let data_try: Result<NetworkData, _> = rmp_serde::from_slice(serialized_data);
        match data_try {
            Ok(data) => match data {
                NetworkData::SendObjectData(id, action_id, action_data) => println!("Action"),
                NetworkData::Instantiate(id, prefab_path, pos) => println!("Instantiation"),
                NetworkData::PositionUpdate(id, pos) => println!("Position updated {}", pos),
                NetworkData::Destroy(id) => println!("Destroyed"),
            },
            Err(err) => println!("{}", err.to_string())
        } 
    }
}

fn steam_system(
    steam_client: Res<Client>,
    keys: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<NetworkClient>,
    channel: Res<LobbyIdCallbackChannel>
) { 
    let (tx, rx): (Sender<LobbyId>, Receiver<LobbyId>) = (channel.tx.clone(), channel.rx.clone());
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
        println!("Received: {}", lobby_id.raw());
    }
}

fn steam_start(
    steam_client: Res<Client>,
    mut commands: Commands,
) {
    println!("Connected: {}", steam_client.user().steam_id().raw());
    commands.insert_resource(NetworkClient {
        id: steam_client.user().steam_id(),
        lobby_status: LobbyStatus::OutOfLobby,
        steam_client: steam_client.clone()
    });
}

fn steam_events(
    steam_client: Res<Client>,
    mut evs: EventReader<SteamworksEvent>,
    mut client: ResMut<NetworkClient>
) {
    for ev in evs.read() {
        //println!("EV");
        match ev {
            SteamworksEvent::GameLobbyJoinRequested(info) => {
                println!("Trying to join: {}", info.lobby_steam_id.raw());
                steam_client.matchmaking().join_lobby(info.lobby_steam_id, |_| {});
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
        }
       /*  if let SteamworksEvent::GameLobbyJoinRequested(info) = ev {
            println!("Trying to join: {}", info.lobby_steam_id.raw());
            steam_client.matchmaking().join_lobby(info.lobby_steam_id, |cb| if let Ok(lobby_id) = cb {client.current_lobby = Some(lobby_id)});
        }*/
    }
}