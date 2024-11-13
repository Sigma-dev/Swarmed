use bevy::prelude::*;
use crate::{ik_arm::{self, IKArmTarget}, leg::{leg_creature::{target::TargetControl, LegCreature, LegSide}, IKLeg}, movable::Movable};

pub fn spawn_spider(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let legs_info: Vec<(Entity, Vec3)> = spawn_legs(&mut commands, &asset_server, 2);

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("spider/spiderV2.glb#Scene0"),
            transform: Transform::from_xyz(0., 0.3, 0.0),
            ..default()
        },
        LegCreature::new(LegSide::None, 0.25, legs_info, 0.2),
        TargetControl,
        Name::new("SpiderBody")
    ));
}

pub fn _spawn_test_arm(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>
) {
    let target = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            transform: Transform::from_xyz(0.2, 0.3, 0.0),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            ..default()
        },
    )).id();

    commands.spawn((SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("leg/legV2.glb")),
            ..default()
        }, 
        ik_arm::IKArm { 
            target: Vec3{x: 1., y: 1., z: 1.},
            up: Vec3::Y
        },
        Name::new("Arm"),
        IKArmTarget {target},
        Movable::new(1.),
    )
    );
}

fn spawn_legs(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    number_per_side: u8,
) -> Vec<(Entity, Vec3)> {
    let step_width = 0.5;
    let step_spacing = 0.45;
    let step_target_height = -0.1;
    let step_distance = 0.15;
    let step_duration = 0.15;
    let step_height = 0.3;

    let leg_width = 0.15;
    let leg_spacing = 0.15;
    let leg_height = -0.1;
    let mut legs = Vec::new();

    for side in [-1, 1] {
        legs.reverse();
        for n in 0..number_per_side {
            let mut group = n;
            if side == 1 { group += 1 };
            let leg_side = if group % 2 == 0 { LegSide::Left } else { LegSide::Right };
            let forward_progress = (n as f32 / (number_per_side - 1) as f32) * 2. - 1.;
            legs.push((commands.spawn((
                SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("leg/legV2.glb")),
                    ..default()
                }, 
                ik_arm::IKArm::default(),
                IKLeg::new(
                    Vec3{x: step_width * side as f32, y: step_target_height, z: step_spacing * forward_progress }, 
                    step_distance, 
                    step_duration,
                    step_height,
                    leg_side,
                    false,
                ),
                Name::new(format!("Leg"))
            )).id(), Vec3::new(leg_width * side as f32, leg_height, leg_spacing * forward_progress)));
        }
    }
    return legs;
}