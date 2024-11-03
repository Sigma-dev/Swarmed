use std::f32::{consts::*, NAN};
use animated_gltf::{AnimatedGltf, AnimatedGltfPlugin};
use avian3d::{prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy, RigidBody}, PhysicsPlugins};
use bevy::{color::palettes::css::{ANTIQUE_WHITE, CRIMSON}, diagnostic::LogDiagnosticsPlugin, math::{NormedVectorSpace, VectorSpace}, prelude::*, render::{mesh::{self, skinning::SkinnedMesh}, settings::{Backends, RenderCreation, WgpuSettings}, RenderPlugin}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use bevy_steam_p2p::*;
use character_controller::{spawn_test_character, spawn_weapon_camera, CurrentPlayer, LocalPlayer};
use debug_component::DebugPlugin;
use fps_camera::{FpsCamera, FpsCameraPlugin};
use fps_movement::{CharacterControllerBundle, CharacterControllerPlugin};
use health::{Health, HealthPlugin};
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::spawn_spider;
use target_spawner::{TargetRespawner, TargetSpawnerPlugin};
use weapon_system::{auxiliary::weapon_target::WeaponTarget, WeaponSystemPlugin};
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod debug_component;
mod leg;
mod spider;
mod fps_camera;
mod fps_movement;
mod character_controller;
mod weapon_system;
mod animated_gltf;
mod health;
mod target_spawner;

#[derive(Component)]
struct Movable {
    pub speed: f32
}

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct MenuEntity;

fn main() {
    App::new()
        .add_plugins(SteamP2PPlugin)
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((IKArmPlugin, LegPlugin, FpsCameraPlugin, WeaponSystemPlugin, AnimatedGltfPlugin, HealthPlugin, TargetSpawnerPlugin, DebugPlugin))
        .add_plugins((LogDiagnosticsPlugin::default(), PhysicsPlugins::default(), CharacterControllerPlugin, character_controller::plugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, (setup, ).chain())
        .add_systems(Update, (movable, steam_system, handle_unhandled_instantiations, update, player_spawned))
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

fn steam_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut evs_lobby: EventReader<LobbyJoined>,
    mut client: ResMut<SteamP2PClient>,
    mut commands: Commands,
    menu_query: Query<Entity, With<MenuEntity>>
) {
    if keys.just_pressed(KeyCode::KeyC) {
        client.create_lobby(8);
    }
    else if keys.just_pressed(KeyCode::KeyV) {
        client.leave_lobby();
    }
    else if keys.just_pressed(KeyCode::KeyT) {
       //client.instantiate(FilePath::new("InstantiationExample"), None, Vec3 {x:1., y:2., z: 1.}).unwrap_or_else(|e| eprintln!("Instantiation error: {e}"));
    }

    for _ in evs_lobby.read() {
        for menu_entity in menu_query.iter() {
            commands.get_entity(menu_entity).unwrap().despawn();
        }
        let player_network_identity = client.instantiate(FilePath::new("Player"), None,Vec3::ZERO).unwrap();
    }
}

fn player_spawned(
    player_query: Query<&NetworkIdentity, Added<LocalPlayer>>,
    mut client: ResMut<SteamP2PClient>,
) {
    let Ok(player) = player_query.get_single() else { return; };
    client.instantiate(FilePath::new("PlayerCamera"), Some(player.id), Vec3::ZERO);
}

fn handle_unhandled_instantiations(
    mut commands: Commands,
    mut evs_unhandled: EventReader<UnhandledInstantiation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: ResMut<AssetServer>,
    mut crosshair_query: Query<(&mut Style, &mut Visibility, Option<&Crosshair>)>,
    mut client: ResMut<SteamP2PClient>,
    player_query: Query<(Entity, &NetworkIdentity), With<CurrentPlayer>>,
) {
    for ev in evs_unhandled.read() {
        println!("Instantiated");
        if ev.network_identity.instantiation_path == "Player" {
            spawn_test_character(&mut client, &mut commands, &mut meshes, &mut materials, ev.network_identity.clone());
        }
        else if ev.network_identity.instantiation_path == "PlayerCamera" {
            spawn_weapon_camera(&mut client, &mut commands, ev.network_identity.clone(), &player_query, &mut crosshair_query);
        }
    }
}

fn update(
    mut animated_query: Query<&mut AnimatedGltf>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mut animated in animated_query.iter_mut() {
        if keys.just_pressed(KeyCode::KeyH) {
            animated.play("Equip");
        }
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(TargetRespawner::new(Vec3 { x: 5., y: 1., z: 0. }, 2.));

    let texture_handle = asset_server.load("crosshairs/default.png");

    commands
    .spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(32.),
                    height: Val::Px(32.),
                    ..default()
                },
                image: UiImage::new(texture_handle),
                visibility: Visibility::Hidden,
                ..default()
            },
            Crosshair,
        ),
        );
    });
    /* 
    commands.spawn(
        SceneBundle {
                scene: asset_server.load("weapons/glock/glock.glb#Scene0"),
                transform: Transform::from_xyz(0., 5., 0.),
                ..default()
                }
            );
            */
    // Create a camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-7.0, 7., -7.0)
                .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
            ..default()
        },
        MenuEntity
    ));
    //spawn_spider(&mut commands, &asset_server, &mut meshes, &mut materials);
        
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("map/map.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
    ));
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
    mut transform_query: Query<(&mut Transform, &Movable)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>
) {
    for (mut movable_transform, movable) in transform_query.iter_mut() {
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
        movable_transform.translation += vec * time.delta_seconds() * movable.speed;
    }
}