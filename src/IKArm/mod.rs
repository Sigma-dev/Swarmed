use std::f32::consts::PI;
use bevy::{prelude::*, render::mesh::{self, skinning::SkinnedMesh}};

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
        app.add_systems(Update, (handle_ik, handle_arm_targets));
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
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut arm_query: Query<(Entity, &IKArm)>,
    children_query: Query<&Children>,
    parent_query: Query<(Entity, &SkinnedMesh)>,
   // mut transform_query: Query<&mut Transform>,
    mut gtransform_query: Query<&mut GlobalTransform>,
    mut gizmos: Gizmos,
    mut transform_params: ParamSet<(
        TransformHelper,
        Query<&mut Transform>,
    )>,
) {
    for (arm_entity, arm) in arm_query.iter_mut() {
        for child in children_query.iter_descendants(arm_entity) {
            let Ok((entity, skinned_mesh)) = parent_query.get(child) else {continue;};
            let target_position: Vec3 = arm.target;
            let Ok([knee_transform, transform]) = gtransform_query.get_many([skinned_mesh.joints[1], arm_entity]) else { println!("fuck"); continue; };
            let mut query = transform_params.p1();
            let Ok([mut t0, mut t1, mut arm_transform]) = query.get_many_mut([skinned_mesh.joints[0], skinned_mesh.joints[1], arm_entity]) else { println!("fuck"); continue; };
            let dir = (target_position - transform.translation());
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
            let middle = (transform.translation() + target_position) / 2.;
            let time_test = time.elapsed_seconds() * 0.5;
            t0.rotation = Quat::from_euler(EulerRot::XYZ, 0. ,-y , (a - z));
            t1.rotation = Quat::from_rotation_z(b);
            let dot = dir.normalize().dot(arm.up);
            let enabled = {dot > 0.};
            
            
            

            if let Ok(updated_knee_transform) = transform_params.p0().compute_global_transform(skinned_mesh.joints[1]) {
                let knee_vec = (updated_knee_transform.translation() - middle).normalize();
                //gizmos.line(middle, middle+ knee_vec, Color::WHITE);
                //gizmos.line(middle, middle+ arm.up, Color::WHITE);
                //println!("{}", knee_vec.dot(arm.up).acos().to_degrees());
                //println!("{}", knee_vec.angle_between(arm.up).to_degrees());
                let rotation_arc = Quat::from_rotation_arc( arm.up, knee_vec);
            // println!("len: {}", rotation_arc.length());
                let signed_angle = signed_angle_between(knee_vec, arm.up, dir);
                //let signed_angle2 = knee_vec.angle_between(arm.up) * (knee_vec.cross(arm.up)).signum();
                let sg = calculate_correction(middle, knee_transform.translation(), arm.up, dir);
                
                //println!("Dot: {}", dot);
             //   println!("SG: {}", sg.to_degrees());
                
                let n = 90_f32.to_radians();
                //transform_params.p1().get_mut(skinned_mesh.joints[0]).unwrap().rotate_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), 0_f32.lerp(signed_angle, 1.)); // we already verified this exists above
            };
            if keys.pressed(KeyCode::KeyB) {
                //println!("FIRST");
                
                //t0.rotate(rotation_arc);
                //let l = t0.looking_to(dir, arm.up);
                //*t0 = l;
            }


           // ZE.rotate_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), rot);
            /*
            if (enabled) {
                t0.rotate_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), -signed_angle / 4.);
                println!("{}", signed_angle.to_degrees());
            } else {
                println!("disabled");
            } */
            //t0.rotate_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), 0_f32.lerp(signed_angle, 0.01));
            //let signed_angle2 = knee_vec.angle_between(arm.up) * (knee_vec.cross(arm.up)).signum();
            //t0.rotate_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), 90_f32.to_radians());

            //arm_transform.rotate_local_axis( Dir3::from_xyz(dir.x, dir.y, dir.z).unwrap(), time_test);
            /*
            FIRST UNDERSTAND WHY SIMPLY ROTATING THE WHOLE THING BY 
            THE ANGLE DIFF DOESN4T WORK, ONLY THEN CAN YOU FIND A FIX
                The angle works, atleast partly, but it oscillates because basically:
                    First frame, the arm moves, so it needs ex 90° so it rotates that much
                    Second frame, t0 rotates again, but since the calculation doesn't see that change, it sees it as perfect, so it does nothing.
                    SOLUTION: Transform Helper to get the up to date globaltransform for the calculation.

            Solution Plan:
            Find way to evaluate angle
                Get GlobalTransform's of [1] joint's position
                Get the vector from the middle of root to target vector
                Take the dot product of that and the up vector, minimize to 0
                FIND A WAY TO COMPUTER WHERE THE KNEE WILL BE
                TRY 360 SOLUTIONS, TAKE THE BEST ONE, ROTATE BY THAT MUCH
            Make a function that finds the best rotate_axis value
            rotate_axis by that amount

            
            
            */
        }
    }
}

fn calc_angles(transform: &GlobalTransform, dir: Vec3) -> (f32, f32) {
    let y = (-transform.right()).xz().angle_between(dir.xz());
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

fn find_best() -> f32 {
    let mut best_i = 0;
    let mut best = 0.;
    for i in 0..359 {
        let result = 0.;
        if (result < best) {
            best = result;
            best_i = i;
        }
    }
    return best_i as f32;
}

fn calculate_correction(middle: Vec3, knee_position: Vec3, up :Vec3, dir: Vec3) -> f32 {
    let knee_vec = (knee_position - middle).normalize();
    let signed_angle = signed_angle_between(knee_vec, up, -dir);
    return signed_angle;
}

fn signed_angle_between(a: Vec3, b: Vec3, normal: Vec3) -> f32 {
    let v1 = a.normalize();
    let v2 = b.normalize();

    let dot_product = v1.dot(v2).clamp(-1.0, 1.0); // Clamp to avoid numerical errors
    let unsigned_angle = dot_product.acos(); // This is the unsigned angle

    let cross_product = v1.cross(v2);
    let sign = cross_product.dot(normal.normalize());

    if sign < 0.0 {
        -unsigned_angle
    } else {
        unsigned_angle
    }
}