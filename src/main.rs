use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use bevy_steamworks::{Client, FriendFlags, GameLobbyJoinRequested, LobbyId, LobbyType, Manager, Matchmaking, SteamError, SteamId, SteamworksEvent, SteamworksPlugin};
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::spawn_spider;
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod leg;
mod spider;

#[derive(Component)]
struct Movable;

#[derive(Resource)]
struct NetworkClient {
    id: SteamId,
    current_lobby: Option<LobbyId>
}

fn lobby_joined(client: &mut ResMut<NetworkClient>, lobby: LobbyId) {
    println!("Lobby joined: {}", lobby.raw());
    client.current_lobby = Some(lobby)
}

fn send_message(steam_client: &Res<Client>, lobby_id: LobbyId, data: i32) {
    for player in steam_client.matchmaking().lobby_members(lobby_id) {
        //steam_client.networking_messages().send_message_to_user(player, , 1, 0);
    }
}

fn steam_system(
    steam_client: Res<Client>,
    keys: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<NetworkClient>
) {
    if (keys.just_pressed(KeyCode::KeyC)) {
        println!("HERE");
        steam_client.matchmaking().create_lobby(
            LobbyType::FriendsOnly,
            2,
            |r| { 
                match r {
                    Ok(lobby_id) => println!("YES: {}", lobby_id.raw()),
                    Err(e) => println!("NO: {}", e),
                }
            });
        return;
    }
    else if (keys.just_pressed(KeyCode::KeyV)) {
        println!("?");
        let Some(lobby) = client.current_lobby else {return; };
        println!("Leave");
        steam_client.matchmaking().leave_lobby(lobby)
    }
    else if (keys.just_pressed(KeyCode::KeyT)) {
        if let Some(lobby_id) = client.current_lobby {
            send_message(&steam_client, lobby_id, 4);
        }
    }
}

fn steam_start(
    steam_client: Res<Client>,
    mut commands: Commands,
) {
    println!("Connected: {}", steam_client.user().steam_id().raw());
    commands.insert_resource(NetworkClient {
        id: steam_client.user().steam_id(),
        current_lobby: None 
    });
}

fn steam_events(
    steam_client: Res<Client>,
    mut evs: EventReader<SteamworksEvent>,
    mut client: ResMut<NetworkClient>
) {
    for ev in evs.read() {
        println!("EV");
        match ev {
            SteamworksEvent::GameLobbyJoinRequested(info) => {
                println!("Trying to join: {}", info.lobby_steam_id.raw());
                steam_client.matchmaking().join_lobby(info.lobby_steam_id, |_| {});
            },
            SteamworksEvent::LobbyChatUpdate(info) => {
                println!("Chat Update");
                match info.member_state_change {
                    bevy_steamworks::ChatMemberStateChange::Entered => lobby_joined(&mut client, info.lobby),
                    bevy_steamworks::ChatMemberStateChange::Left => client.current_lobby = None,
                    bevy_steamworks::ChatMemberStateChange::Disconnected => client.current_lobby = None,
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

fn main() {
    App::new()
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .add_plugins(DefaultPlugins)
        .add_plugins((IKArmPlugin, LegPlugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, steam_start)
        .add_systems(Update, (steam_system, steam_events))
        .add_systems(Startup, (setup, ).chain())
        .add_systems(Update, (movable))
       // .observe(modify_meshes)
        .run();
}

fn modify_meshes(
    trigger: Trigger<OnAdd, Handle<Mesh>>,
    mut commands: Commands,
  ) {
    commands
      .entity(trigger.entity())
      .insert(NoBackfaceCulling);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-7.0, 7., -7.0)
            .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
        ..default()
    });

    spawn_spider(&mut commands, &asset_server, &mut meshes, &mut materials);
        
    commands.spawn(SceneBundle {
        scene: asset_server.load("map/map.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

}

fn movable(
    mut transform_query: Query<&mut Transform, With<Movable>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut movable_transform) in transform_query.iter_mut() {
        let mut vec = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            vec.z += 1.0
        }
        if keys.pressed(KeyCode::KeyS) {
            vec.z -= 1.0
        }
        if keys.pressed(KeyCode::KeyD) {
            vec.x -= 1.0
        }
        if keys.pressed(KeyCode::KeyA) {
            vec.x += 1.0
        }
        if keys.pressed(KeyCode::KeyQ) {
            vec.y += 1.0
        }
        if keys.pressed(KeyCode::KeyE) {
            vec.y -= 1.0
        }
        movable_transform.translation += vec * 0.01;
    }
}