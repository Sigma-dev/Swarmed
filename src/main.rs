use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::spawn_spider;
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod leg;
mod spider;

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct MEsh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((IKArmPlugin, LegPlugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
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