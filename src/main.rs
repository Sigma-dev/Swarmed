use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use leg::{IKLeg, LegCreature, LegPlugin, LegSide};
use IKArm::{IKArmPlugin};

mod IKArm;
mod leg;

#[derive(Component)]
struct Movable;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((IKArmPlugin, LegPlugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (movable))
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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.3, 0.3)),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            transform: Transform::from_xyz(0., 0.3, 0.0),
            ..default()
        },
        Movable,
        LegCreature { current_side: LegSide::None, target_height: 0.2}
    )).with_children(|parent| {
        for i in 0..2 {
            let side_mult = if (i == 0) { 1. }  else {-1.};
            let side = if (i == 0) { LegSide::Left }  else { LegSide::Right };
            let side2 = if (i == 0) { LegSide::Right }  else { LegSide::Left };
            parent.spawn((SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
                transform: Transform::from_xyz(0.15 * side_mult, -0.1, -0.1),
                ..default()
                }, 
                IKArm::IKArm { 
                    target: Vec3{x: 1., y: 0., z: 1.}
                },
                IKLeg::new(
                     Vec3{x: 0.5 * side_mult, y: -0.1, z: -0.35 }, 
                     0.35, 
                     0.15,
                     0.3,
                     side,
                    false,
                )
            )
            );

            parent.spawn((SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
                transform: Transform::from_xyz(0.15 * side_mult, -0.1, 0.1),
                ..default()
                }, 
                IKArm::IKArm { 
                    target: Vec3{x: 1., y: 0., z: 1.}
                },
                IKLeg::new(
                    Vec3{x: 0.5 * side_mult, y: -0.1, z: 0.35 }, 
                    0.35, 
                    0.15,
                    0.3,
                    side2,
                   false,
                )
            ));
        }
    });
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