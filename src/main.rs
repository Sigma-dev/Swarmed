use std::f32::{consts::*, NAN};
use avian3d::{prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy, RigidBody}, PhysicsPlugins};
use bevy::{diagnostic::LogDiagnosticsPlugin, math::{NormedVectorSpace, VectorSpace}, prelude::*, render::{mesh::{self, skinning::SkinnedMesh}, settings::{Backends, RenderCreation, WgpuSettings}, RenderPlugin}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use bevy_steam_p2p::*;
use character_controller::spawn_test_character;
use fps_camera::{FpsCamera, FpsCameraPlugin};
use fps_movement::{CharacterControllerBundle, CharacterControllerPlugin};
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::spawn_spider;
use weapon_system::WeaponSystemPlugin;
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod leg;
mod spider;
mod fps_camera;
mod fps_movement;
mod character_controller;
mod weapon_system;

#[derive(Component)]
struct Movable {
    pub speed: f32
}

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
        .add_plugins((IKArmPlugin, LegPlugin, FpsCameraPlugin, WeaponSystemPlugin))
        .add_plugins((LogDiagnosticsPlugin::default(), PhysicsPlugins::default(), CharacterControllerPlugin, character_controller::plugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, (setup, ).chain())
        .add_systems(Update, (movable, steam_system, handle_unhandled_instantiations))
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
    mut client: ResMut<crate::client::SteamP2PClient>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        client.create_lobby(8);
    }
    else if keys.just_pressed(KeyCode::KeyV) {
        client.leave_lobby();
    }
    else if keys.just_pressed(KeyCode::KeyT) {
       client.instantiate(FilePath::new("InstantiationExample"),Vec3 {x:1., y:2., z: 1.}).unwrap_or_else(|e| eprintln!("Instantiation error: {e}"));
    }

    for _ in evs_lobby.read() {
        client.instantiate(FilePath::new("Player"), Vec3::ZERO);
    }
}

fn handle_unhandled_instantiations(
    mut commands: Commands,
    mut evs_unhandled: EventReader<UnhandledInstantiation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: ResMut<AssetServer>
) {
    for ev in evs_unhandled.read() {
        println!("Instantiated");
        if ev.network_identity.instantiation_path == "Player" {
            spawn_test_character(&mut commands, &mut meshes, &mut materials, ev.network_identity.clone());
        }
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    // Create a camera
     /* 
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-7.0, 7., -7.0)
            .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
        ..default()
    });
*/
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