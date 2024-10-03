use core::panic;
use std::f32::consts::PI;
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};

#[derive(Component)]
pub struct IKArm {
    pub target: Vec3,
    pub up: Vec3,
}

#[derive(Component)]
pub struct IKArmTarget {
    pub target: Entity
}

pub struct IKArmPlugin;

impl Plugin for IKArmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_up, handle_ik, handle_arm_targets));
    }
}

fn handle_arm_targets(
    mut arm_query: Query<(&mut IKArm, &IKArmTarget)>,
    target_query: Query<&GlobalTransform>,
) {
    for (mut arm, arm_target) in arm_query.iter_mut() {
        let Ok(transform) = target_query.get(arm_target.target) else {continue;};
        arm.target = transform.translation();
    }
}


fn handle_ik(
    arm_query: Query<(Entity, &mut IKArm)>,
    children_query: Query<&Children>,
    parent_query: Query<(Entity, &SkinnedMesh)>,
    mut transform_query: Query<&mut Transform>,
    gtransform_query: Query<&mut GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for (arm_entity, arm) in arm_query.iter() {
        for child in children_query.iter_descendants(arm_entity) {
            //Get the info from bevy
            let Ok((entity, skinned_mesh)) = parent_query.get(child) else {continue;};
            let Ok([root_transform]) = gtransform_query.get_many([arm_entity]) else { println!("fuck"); continue; };
            let Ok([mut t0, mut t1, mut arm_transform]) = transform_query.get_many_mut([skinned_mesh.joints[0], skinned_mesh.joints[1], arm_entity]) else { println!("fuck"); continue; };
            
            let t1_pos = t1.translation;
            //Calculate the important positions
            let root = t0.translation;
            let l1: f32 = root.distance(t1.translation);
            let l2: f32 = root.distance(t1_pos);
            let target_position: Vec3 = arm.target;
            let Some(knee_position) = get_knee_position(&mut gizmos, root, target_position, arm.up, l1, l2) else { continue; };

            //Visualize stuff
            gizmos.line(knee_position, target_position, Color::srgb(0., 0.5, 0.));
            gizmos.sphere(knee_position, Quat::IDENTITY, 0.1, Color::srgb(0., 1., 0.));

            //Rotate the bones (t0 & t1) so that the mesh matches the positions
            let knee_direction: Vec3 = (knee_position - root_transform.translation()).normalize();
            let target_direction: Vec3 = (arm.target - knee_position).normalize();
            t0.look_at((target_position - root).reject_from(knee_direction), knee_direction);
            t0.rotate_local_y(-crate::FRAC_PI_2);
            let angle = t0.up().angle_between(target_direction);
            t1.rotation = Quat::from_axis_angle(t1.forward().as_vec3().normalize(), -angle);
        }
    }
}

fn handle_up(
    mut arm_query: Query<&mut IKArm>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mut arm in arm_query.iter_mut() {
        if keys.pressed(KeyCode::Numpad4) {
            arm.up = arm.up.lerp(Vec3::X, 0.1);
        }
        else if keys.pressed(KeyCode::Numpad6) {
            arm.up = arm.up.lerp(-Vec3::X, 0.1);
        }
        else if keys.pressed(KeyCode::Numpad8) {
            arm.up = arm.up.lerp(Vec3::Z, 0.1);
        }
        else if keys.pressed(KeyCode::Numpad5) {
            arm.up = arm.up.lerp(-Vec3::Z, 0.1);
        }
        else if keys.pressed(KeyCode::Numpad7) {
            arm.up = arm.up.lerp(Vec3::Y, 0.1);
        }
        else if keys.pressed(KeyCode::Numpad9) {
            arm.up = arm.up.lerp(-Vec3::Y, 0.1);
        }
        arm.up = arm.up.normalize();
    }
}

fn get_knee_position(gizmos: &mut Gizmos, root: Vec3, target: Vec3, up: Vec3, l1: f32, l2: f32) -> Option<Vec3> {
    let target_direction = (target - root).normalize();
    let knee_circle_center = (target - root) / 2.;
    let knee_circle_distance = root.distance(knee_circle_center);
    let knee_circle_radius = (l1.powi(2) - knee_circle_distance.powi(2)).sqrt();
    let Ok(knee_circle_normal) = Dir3::new(target_direction) else { return None };
    let squished_up = squish_on_plane( up, knee_circle_normal.as_vec3(), knee_circle_radius); //Maybe jitter will be caused by varying solutions when the up is close the the normal. Rejecting corrections when the projected vector is small could be a workaround
    let knee_position = knee_circle_center + squished_up;
    gizmos.circle(knee_circle_center, knee_circle_normal, knee_circle_radius, Color::srgb(0., 0., 0.5));
    if knee_position.is_nan() { return None };
    return Some(knee_position);
}

fn squish_on_plane(v: Vec3, normal: Vec3, radius: f32) -> Vec3 {
    let projected = project_onto_plane(v, normal);
    return projected.normalize() * radius;
}

fn project_onto_plane(v: Vec3, normal: Vec3) -> Vec3 {
    return v - v.dot(normal) * normal;
}