use std::f32::{consts::*, NAN};
use bevy::{diagnostic::LogDiagnosticsPlugin, math::{NormedVectorSpace, VectorSpace}, prelude::*, render::{mesh::{self, skinning::SkinnedMesh}, settings::{Backends, RenderCreation, WgpuSettings}, RenderPlugin}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use bevy_steam_p2p::*;
use fps_camera::{FpsCamera, FpsCameraPlugin};
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::spawn_spider;
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod leg;
mod spider;
mod fps_camera;

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
        .add_plugins((IKArmPlugin, LegPlugin, FpsCameraPlugin))
        .add_plugins(LogDiagnosticsPlugin::default(),)
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, (setup, ).chain())
        .add_systems(Update, (movable, steam_system, receive_network_messages))
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
    mut client: ResMut<crate::client::SteamP2PClient>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        client.create_lobby(8);
    }
    else if keys.just_pressed(KeyCode::KeyV) {
        client.leave_lobby();
    }
    else if keys.just_pressed(KeyCode::KeyP) {
        client.send_message_all(NetworkData::NetworkMessage("2048".into()), SendFlags::RELIABLE);
    }
    else if keys.just_pressed(KeyCode::KeyT) {
       client.instantiate(FilePath(0),Vec3 {x:1., y:2., z: 1.}).unwrap_or_else(|e| eprintln!("Instantiation error: {e}"));
    }
}

fn receive_network_messages(
    mut commands: Commands,
    mut evs_messages: EventReader<NetworkPacket>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in evs_messages.read() {
        if let NetworkData::NetworkMessage(ref msg) = ev.data {
            if let Ok(id) = msg.parse::<u32>() {
                commands.spawn((
                    PbrBundle {
                    mesh: meshes.add(Capsule3d::new(1.0, 1.0)),
                    material: materials.add(Color::srgb_u8(124, 144, 255)),
                    ..default()
                    },
                    NetworkIdentity { owner_id: ev.sender, id },
                    NetworkedTransform{synced: true, target: Vec3::ZERO},
                )).with_children(|parent| {
                    parent.spawn((
                        Camera3dBundle {
                            transform: Transform::from_xyz(0., 1.8, 0.),
                            ..default()
                        },
                        FpsCamera { sensitivity: 1. }
                    ));
                });
            }
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