use std::f32::{consts::*, NAN};
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use IKArm::{IKArmPlugin};

mod IKArm;

#[derive(Component)]
struct Movable;

#[derive(Copy, Clone, PartialEq)]
enum LegSide {
    Left,
    Right,
    None,
}

#[derive(Component)]
struct Leg {
    step_offset: Vec3,
    step_distance: f32,
    step_duration: f32,
    step_height: f32,
    step_start: Vec3,
    stepping: bool,
    step_elapsed: f32,
    leg_side: LegSide,
    can_start_step: bool
}

#[derive(Component)]
struct LegCreature {
    current_side: LegSide
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
        .add_systems(Update, (determine_side, handle_leg_creature))
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
    /*
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
 */
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.3, 0.3)),
            material: materials.add(Color::srgb_u8(10, 10, 10)),
            transform: Transform::from_xyz(0., 0.3, 0.0),
            ..default()
        },
        Movable,
        LegCreature { current_side: LegSide::None}
    )).with_children(|parent| {
        for i in 0..2 {
            let side_mult = if (i == 0) { 1. }  else {-1.};
            let side = if (i == 0) { LegSide::Left }  else { LegSide::Right };
            let side2 = if (i == 0) { LegSide::Right }  else { LegSide::Left };
            parent.spawn((SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
                transform: Transform::from_xyz(0.15 * side_mult, -0.2, -0.1),
                ..default()
                }, 
                IKArm::IKArm { 
                    target: Vec3{x: 1., y: 0., z: 1.}
                },
                Leg { 
                    step_offset: Vec3{x: 0.5 * side_mult, y: -0.1, z: -0.35 }, 
                    step_distance: 0.35, 
                    step_duration: 0.15,
                    step_height: 0.3,
                    stepping: false,
                    step_start: Vec3::ZERO,
                    step_elapsed: 0.,
                    leg_side: side,
                    can_start_step: false
                }
            ));

            parent.spawn((SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
                transform: Transform::from_xyz(0.15 * side_mult, -0.2, 0.1),
                ..default()
                }, 
                IKArm::IKArm { 
                    target: Vec3{x: 1., y: 0., z: 1.}
                },
                Leg { 
                    step_offset: Vec3{x: 0.5 * side_mult, y: -0.1, z: 0.35 }, 
                    step_distance: 0.35, 
                    step_duration: 0.15,
                    step_height: 0.3,
                    stepping: false,
                    step_start: Vec3::ZERO,
                    step_elapsed: 0.,
                    leg_side: side2,
                    can_start_step: false
                }
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

fn determine_side(
    leg_query: Query<(&Leg)>,
    mut leg_creature_query: Query<(Entity, &mut LegCreature)>,
    children_query: Query<&Children>,
) {
    for (creature_entity, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut left_side_moving = false;
        let mut right_side_moving = false;
        for child in children_query.iter_descendants(creature_entity) {
            let Ok((mut leg)) = leg_query.get(child) else {continue;};
            if leg.stepping {
                match leg.leg_side {
                    LegSide::Left => left_side_moving = true,
                    LegSide::Right => right_side_moving = true,
                    LegSide::None => {},
                }
            }
        }
        if (!left_side_moving && !right_side_moving) {
            println!("SWAP");
            if leg_creature.current_side == LegSide::Left {
                leg_creature.current_side = LegSide::Right;
            } else {
                leg_creature.current_side = LegSide::Left;
            }
        }
        println!("SIDE: {}", leg_creature.current_side as i8);
    }
}

fn handle_leg_creature(
    mut leg_query: Query<(&mut Leg)>,
    leg_creature_query: Query<(Entity, &LegCreature)>,
    children_query: Query<&Children>,
) {
    println!("SETTING LEG CAN START");
    for (creature_entity, mut leg_creature) in leg_creature_query.iter() {
        for child in children_query.iter_descendants(creature_entity) {
            let Ok((mut leg)) = leg_query.get_mut(child) else {continue;};
            if (leg.leg_side == leg_creature.current_side) {
                leg.can_start_step = true;
            } else {
                leg.can_start_step = false;
            }
            println!("{} {}",  leg.leg_side as i8,leg.can_start_step,);
        }
    }
}

fn handle_legs(
    mut leg_query: Query<(&GlobalTransform, &mut IKArm::IKArm, &mut Leg)>,
    time: Res<Time>,
) {
    //let group = get_highest_distance_group(&leg_query);
    for (transform, mut arm, mut leg) in leg_query.iter_mut() {
        let desired_pos = transform.translation() + leg.step_offset;
        let distance = arm.target.distance(desired_pos);
        println!("LEGS");
        println!("{} {}",  leg.leg_side as i8,leg.can_start_step,);
        if (!leg.stepping) {
            if (distance > leg.step_distance && leg.can_start_step) {
                println!("STEP");
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

fn get_highest_distance_group(
    mut leg_query: &Query<(&GlobalTransform, &mut IKArm::IKArm, &mut Leg)>,
) -> LegSide {
    let mut left_dist = 0.;
    let mut right_dist = 0.;
    
    for (transform, arm, leg) in leg_query.iter() {
        let desired_pos = transform.translation() + leg.step_offset;
        let distance = arm.target.distance(desired_pos);
        match leg.leg_side {
            LegSide::Left => left_dist += distance,
            LegSide::Right => right_dist += distance,
            LegSide::None => {},
        }
    }
    if (left_dist > right_dist) {
        return LegSide::Left
    } else {
        return LegSide::Right
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