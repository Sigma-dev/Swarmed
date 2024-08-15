use std::f32::consts::PI;
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};

#[derive(Component)]
pub struct IKArm {
    pub target: Vec3
}

pub struct IKArmPlugin;

impl Plugin for IKArmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_ik);
    }
}

fn handle_ik(
    time: Res<Time>,
    arm_query: Query<(Entity, &IKArm)>,
    children_query: Query<&Children>,
    parent_query: Query<(Entity, &SkinnedMesh)>,
    mut transform_query: Query<&mut Transform>,
    mut gizmos: Gizmos,

) {
    for (arm_entity, arm) in arm_query.iter() {
        for child in children_query.iter_descendants(arm_entity) {
            let Ok((entity, skinned_mesh)) = parent_query.get(child) else {continue;};
            let target_position: Vec3 = arm.target;
            let Ok([mut t0, mut t1, transform]) = transform_query.get_many_mut([skinned_mesh.joints[0], skinned_mesh.joints[1], arm_entity]) else { println!("fuck"); continue; };
            let dir = (target_position - transform.translation);
            let (y, z) = calc_angles(&transform, dir);
            let d_a: f32 = t0.translation.distance(t1.translation);
            let d_b: f32 = t0.translation.distance(t1.translation);
            let mut d_c = dir.length();
            if (d_c > d_a + d_c) {
                d_c = d_a + d_c;
            }
            let mut a = calc_necessary_angle(d_b, d_c, d_a);
            let mut b: f32 = calc_necessary_angle(d_a, d_b, d_c);
            if a.is_nan(){
                a = 0.;
            }
            if b.is_nan() {
                b = PI;
            }
            a = PI/2. - a;
            b = PI - b;
            t0.rotation = Quat::from_euler(EulerRot::XYZ, 0.,-y, a - z);
            t1.rotation = Quat::from_rotation_z(b);
        }
    }
}

fn calc_angles(transform: &Transform, dir: Vec3) -> (f32, f32) {
    let y = (-transform.local_x()).xz().angle_between(dir.xz());
    let mut inbetween = dir;
    inbetween.y = 0.;
    let mut z = dir.angle_between(inbetween);
    if (dir.y < 0.) {
        z = -z;
    }
    return (y, z);
}

fn calc_necessary_angle(a: f32, b: f32, c: f32) -> f32 {
    let top_part = a.powf(2.) + b.powf(2.) - c.powf(2.);
    let bottom_part = 2. * a * b;
    let result = (top_part / bottom_part).acos();
    return result;
}
