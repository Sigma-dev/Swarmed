use bevy::app::Startup;
use bevy::asset::AssetServer;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{default, in_state, App, Camera3dBundle, Commands, Component, Display, Entity, Events, ImageBundle, IntoSystemConfigs, KeyCode, NextState, OnEnter, OnExit, Plugin, PositionType, Query, Res, ResMut, Resource, Style, Transform, UiImage, Update, Val, With, World, ZIndex};
use bevy::ui::FocusPolicy;
use bevy::DefaultPlugins;
use bevy_egui::{egui, EguiPlugin, EguiContexts};
use bevy_egui::egui::{Button, Color32, Frame, RichText, TextStyle};
use bevy_steamworks::{Client, LobbyDataUpdate, LobbyId, LobbyType, SteamworksPlugin, UserStatsReceived};
use flume::{Receiver, Sender};

fn main() {
    App::new()
        .add_plugins(SteamworksPlugin::init_app(480).unwrap())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(UIHostLobbyPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-7.0, 7., -7.0)
            .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
        ..default()
    });

}

pub struct UIHostLobbyPlugin;
impl Plugin for UIHostLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            host_lobby_menu_setup
        );
        app.insert_resource(UIState {
            lobby_name: "".parse().unwrap(),
            lobby_type: LobbyType::Public
        });
        let (tx, rx) = flume::unbounded();
        app.insert_resource(LobbyIdCallbackChannel {
            tx, rx
        });
        app.add_systems(Update, host_lobby_menu_system);
        // app.add_event::<LobbyDataUpdate>();
    }
}

#[derive(Resource)]
struct LobbyIdCallbackChannel {
    tx: Sender<LobbyId>,
    rx: Receiver<LobbyId>
}

#[derive(Component)]
struct HostLobbyMenuTag;

#[derive(Resource)]
struct UIState {
    lobby_name: String,
    lobby_type: LobbyType
}

fn host_lobby_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let title_back = asset_server.load("images/title_back.png");
    commands.spawn((
        ImageBundle {
            image: UiImage::new(title_back),
            focus_policy: FocusPolicy::Pass,
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                ..default()
            },
            z_index: ZIndex::Local(-1),
            ..default()
        },
        HostLobbyMenuTag
    ));
}

fn host_lobby_menu_cleanup(cleanup: Query<Entity, With<HostLobbyMenuTag>>, mut commands: Commands) {
    for entity in &cleanup {
        commands.entity(entity).despawn_recursive();
    }
}

fn host_lobby_menu_system(
    mut ui_state: ResMut<UIState>,
    steam_client: Res<Client>,
    channel: Res<LobbyIdCallbackChannel>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (tx, rx): (Sender<LobbyId>, Receiver<LobbyId>) = (channel.tx.clone(), channel.rx.clone());
    if keys.pressed(KeyCode::KeyC) {
        println!("Yes");
        steam_client.matchmaking().create_lobby(ui_state.lobby_type, 2, move |res| {
            if let Ok(lobby_id) = res {
                // access system parameter from here...
                // game_state.set(ClientState::InLobby);
                println!("a {}", res.unwrap().raw());
                match tx.send(lobby_id) {
                    Ok(_) => {
                        println!("send ok")
                    }
                    Err(_) => {
                        println!("send err")
                    }
                }
            }
        });
    }
    if let Ok(lobby_id) = rx.try_recv() {
        //game_state.set(ClientState::InLobby);
        println!("Received: {}", lobby_id.raw() as u32);
        steam_client.matchmaking().set_lobby_data(lobby_id, "name", &*ui_state.lobby_name);
    }
}