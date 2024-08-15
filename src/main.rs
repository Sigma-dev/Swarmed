use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use IKArm::{IKArmPlugin, TargetType};

mod IKArm;

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct Leg {
    step_offset: Vec3,
    step_distance: f32, 
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IKArmPlugin)
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, movable)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 5., -2.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        ..default()
    });

    let target = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.85, 0.0, 0.0),
            ..default()
        },
      //  Movable,
    )).id();

    // Spawn the first scene in `models/SimpleSkin/SimpleSkin.gltf`
    commands.spawn((SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
        transform: Transform::from_xyz(0.0, 0.03, 0.0),
        ..default()
        }, 
        Movable, 
        IKArm::IKArm { 
            target: TargetType::Entity(target) 
            //target: TargetType::Position(Vec3{x: 1., y: 0., z: 1.}) 
        }
    ));

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