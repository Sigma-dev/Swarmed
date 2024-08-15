use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use IKArm::{IKArmPlugin};

mod IKArm;

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct Leg {
    step_offset: Vec3,
    step_distance: f32,
    step_duration: f32,
    step_height: f32,
    step_start: Vec3,
    stepping: bool,
    step_elapsed: f32,
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
        .add_systems(Update, (movable, handle_legs))
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
            target: Vec3{x: 1., y: 0., z: 1.}
        },
        Leg { 
            step_offset: Vec3{x: 0.7, y: 0., z: 0.}, 
            step_distance: 0.5, 
            step_duration: 0.15,
            step_height: 0.3,
            stepping: false,
            step_start: Vec3::ZERO,
            step_elapsed: 0.
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

fn handle_legs(
    mut leg_query: Query<(&Transform, &mut IKArm::IKArm, &mut Leg)>,
    time: Res<Time>,
) {
    for (transform, mut arm, mut leg) in leg_query.iter_mut() {
        let desired_pos = transform.translation + leg.step_offset;
        let distance = arm.target.distance(desired_pos);
        if (!leg.stepping) {
            if (distance > leg.step_distance) {
                leg.stepping = true;
                leg.step_elapsed = 0.;
                leg.step_start = arm.target;
            }
        } else {
            let step_progress = leg.step_elapsed / leg.step_duration;
            arm.target = leg.step_start.lerp(desired_pos, leg.step_elapsed / leg.step_duration);
            let y_offset = (1. - ((step_progress * 2.) - 1.).abs()) * leg.step_height;
            arm.target.y = leg.step_start.y + y_offset;
            leg.step_elapsed += time.delta_seconds();
            if (leg.step_elapsed >= leg.step_duration) {
                arm.target = desired_pos;
                leg.stepping = false;
            }
        }
    }
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