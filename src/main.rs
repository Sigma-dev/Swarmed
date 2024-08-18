use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use IKArm::{IKArmPlugin};

mod IKArm;
mod leg;

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
        .observe(modify_meshes)
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
        transform: Transform::from_xyz(-4.0, 5., -4.0)
            .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
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

    let legs_info: Vec<(Entity, Vec3)> = spawn_legs(&mut commands, &asset_server);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.3, 0.3)),
            transform: Transform::from_xyz(0., 0.3, 0.0),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            ..default()
        },
        //Movable,
        LegCreature::new(LegSide::None, 0.2, legs_info)
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

fn spawn_legs(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>
) -> Vec<(Entity, Vec3)> {
    let mut left_legs = Vec::new();
    let mut right_legs = Vec::new();
    for i in 0..2 {
        let side_mult = if (i == 0) { 1. }  else {-1.};
        let side = if (i == 0) { LegSide::Left }  else { LegSide::Right };
        let side2 = if (i == 0) { LegSide::Right }  else { LegSide::Left };
        for j in 0..2 {
            let front_or_back_mult = if (j == 0) { 1. }  else {-1.};
            let offset = Vec3::new(0.15 * side_mult, -0.1, 0.1 * front_or_back_mult);
            let side3 = if j == 0 { side } else {side2};
            let collector = if (i == 0) { &mut left_legs } else {&mut right_legs };
            let name = format!("{i}{j}", i=i, j=j);
            println!("Name: {}", name);
            collector.push((commands.spawn((SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
                ..default()
                }, 
                IKArm::IKArm { 
                    target: Vec3{x: 1., y: 0., z: 1.}
                },
                IKLeg::new(
                    Vec3{x: 0.5 * side_mult, y: -0.1, z: 0.35 * front_or_back_mult }, 
                    0.35, 
                    0.15,
                    0.3,
                    side3,
                    false,
                ),
                Name::new(name)
            )
            ).id(), offset));
        }
    }
    right_legs.reverse();
    left_legs.append(&mut right_legs);
    return left_legs;
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