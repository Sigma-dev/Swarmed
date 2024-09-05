use bevy::{color::palettes::css::BLACK, math::{NormedVectorSpace, VectorSpace}, prelude::*, reflect::Array, render::mesh::{self, skinning::SkinnedMesh}};

use crate::{leg::{IKLeg, LegCreature, LegSide}, IKArm::{self, IKArmTarget}, Movable};

pub fn spawn_spider(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>
) {
    let target = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(-0.5, 0., 0.),
            ..default()
        },
        Movable,
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
        LegCreature::new(LegSide::None, 0.2, legs_info, 0.2)
    ));
}

pub fn spawn_test_arm(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>
) {
    let target = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            transform: Transform::from_xyz(0.2, 0.3, 0.0),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            ..default()
        },
        Movable,
    )).id();

    commands.spawn((SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("leg/leg2.glb")),
            ..default()
        }, 
        IKArm::IKArm { 
            target: Vec3{x: 1., y: 1., z: 1.},
            up: Vec3::Y
        },
        Name::new("Arm"),
        IKArmTarget {target}
    )
    );
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
                    target: Vec3{x: 1., y: 0., z: 1.},
                    up: Vec3::Y
                },
                IKLeg::new(
                    Vec3{x: 0.5 * side_mult, y: -0.1, z: 0.35 * front_or_back_mult }, 
                    0.1, 
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