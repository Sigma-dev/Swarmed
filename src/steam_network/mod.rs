use bevy::{app::{App, Plugin, Startup, Update}, input::ButtonInput, math::Vec3, prelude::{Commands, EventReader, KeyCode, Res, ResMut, Resource}, scene::serde};
use bevy_steamworks::{Client, FriendFlags, GameLobbyJoinRequested, LobbyId, LobbyType, Manager, Matchmaking, SteamError, SteamId, SteamworksEvent, SteamworksPlugin};
use flume::{Receiver, Sender};
use ::serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;
use steamworks::networking_types::{NetworkingIdentity, SendFlags};
pub struct SteamNetworkPlugin;

impl Plugin for SteamNetworkPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = flume::unbounded();
        app
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .insert_resource(LobbyIdCallbackChannel {
            tx, rx
        })
        .add_systems(Startup, steam_start)
        .add_systems(Update, (steam_system, steam_events));
    }
}

#[derive(Resource)]
struct NetworkClient {
    id: SteamId,
    lobby_status: LobbyStatus
}

#[derive(PartialEq)]
enum LobbyStatus {
    InLobby(LobbyId),
    OutOfLobby
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkId(u32);

#[derive(Serialize, Deserialize, Debug)]
enum NetworkData {
    PositionUpdate((NetworkId, Vec3))
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
        //send_type: 0 Unreliable 8 reliable
        let serialize_data = serde_binary::to_vec(&data, Endian::Big);
        let Ok(serialized) = serialize_data else {return;};
       // let buffer: [u8, 64] = [];
        let data_arr = serialized.as_slice();
        for n in data_arr {
            println!("{}", n);
        }
        let network_identity = NetworkingIdentity::new_steam_id(player);
        if network_identity.is_valid() {
            println!("Valid identity");
        } else {
            println!("Invalid identity");
        }
        let res = steam_client.networking_messages().send_message_to_user(network_identity, SendFlags::RELIABLE, &data_arr, 0);
        match res {
            Ok(_) => println!("Message sent succesfully"),
            Err(err) => println!("Message error: {}", err.to_string()),
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
            send_message(&steam_client, lobby_id, NetworkData::PositionUpdate((NetworkId(0), Vec3::ONE)));
        }
    }
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
        lobby_status: LobbyStatus::OutOfLobby 
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