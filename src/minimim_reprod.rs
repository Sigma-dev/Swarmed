use bevy::prelude::*;
use bevy_steamworks::{self, Client, LobbyType, SteamworksEvent, SteamworksPlugin};

fn steam_system(
    steam_client: Res<Client>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
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
}

fn steam_events(
    mut evs: EventReader<SteamworksEvent>,
) {
    for ev in evs.read() {
        println!("Event received");
        match ev {
            SteamworksEvent::LobbyChatUpdate(info) => {
                println!("Chat Update");
                match info.member_state_change {
                    bevy_steamworks::ChatMemberStateChange::Entered => println!("ENTERED"),
                    _ => println!("Non-entered chat update")
                }
            },
            _ => println!("Non chat update event received"),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .add_plugins(DefaultPlugins)
        .add_systems(Update, (steam_system, steam_events).chain())
        .run();
}
