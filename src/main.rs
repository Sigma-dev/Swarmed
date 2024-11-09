use std::env;

use bevy::prelude::*;
use leg::LegPlugin;
use multi_pos::MultiPosCamera;
use spider::spawn_spider;
use ik_arm::IKArmPlugin;

pub mod ik_arm;
pub mod leg;
pub mod spider;
pub mod debug_resource;
pub mod movable;
pub mod multi_pos;

#[derive(Component)]
struct GroundMarker;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((IKArmPlugin, LegPlugin::default()))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_plugins((debug_resource::plugin, movable::plugin, multi_pos::plugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3dBundle::default(),
        MultiPosCamera::new(
            vec![
                (Vec3::new(-7.0, 7., -7.0), Vec3::new(0.0, 0., 0.0)),
                (Vec3::new(0.0, 10., 0.0), Vec3::new(0.0, 0., 0.0))
            ]
        ).with_lerp(0.05)
    ));        
    commands.spawn((
        SceneBundle {
        scene: asset_server.load("map/map.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
        },
        GroundMarker,
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    spawn_spider(&mut commands, &asset_server);
}