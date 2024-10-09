use bevy::{prelude::*, render::render_resource::encase::rts_array::Length};
use bevy_mod_raycast::prelude::*;
use itertools::Itertools;

use crate::{leg, GroundMarker, IKArm};
#[derive(Copy, Clone, PartialEq, Default)]
pub enum LegSide {
    Left,
    Right,
    #[default] None,
}

#[derive(Component)]
pub struct IKLeg {
    pub step_offset: Vec3,
    pub step_distance: f32,
    pub step_duration: f32,
    pub step_height: f32,
    pub leg_side: LegSide,
    pub can_start_step: bool,
    step_start: Vec3,
    stepping: bool,
    step_elapsed: f32,
}

impl IKLeg {
    pub fn new(
        step_offset: Vec3,
        step_distance: f32,
        step_duration: f32,
        step_height: f32,
        leg_side: LegSide,
        can_start_step: bool,
    ) -> Self {
        Self { step_offset, step_distance, step_duration, step_height, leg_side, can_start_step, step_start: Vec3::ZERO, stepping: false, step_elapsed: 0. }
    }
}

#[derive(Component)]
pub struct LegCreature {
    pub(crate) current_side: LegSide,
    pub target_height: f32,
    up: Vec3,
    pub legs_info: Vec<(Entity, Vec3)>,
    pub speed_mult: f32,
    target_offset: Vec3,
}
impl LegCreature {
    pub fn new(
        current_side: LegSide,
        target_height: f32,
        legs_info: Vec<(Entity, Vec3)>,
        speed_mult: f32
    ) -> Self {
        Self { current_side, target_height, up: Vec3::Y, legs_info, target_offset: Vec3::ZERO, speed_mult }
    }
}

#[derive(Component)]
pub struct LegCreatureVisual {
}

pub struct LegPlugin;

impl Plugin for LegPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_visual, determine_side, handle_leg_creature, handle_legs, move_creature, handle_height).chain())
        .observe(setup_legs);
    }
}

fn setup_legs(
    trigger: Trigger<OnAdd, IKLeg>,
    mut leg_query: Query<(&GlobalTransform, &mut IKArm::IKArm, &IKLeg)>
  ) {
    let Ok((transform, mut arm, leg)) = leg_query.get_mut(trigger.entity()) else {return;};
    arm.target = transform.translation() + leg.step_offset;
}

fn move_creature(
    mut creature_query: Query<(&mut Transform, &mut LegCreature)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, mut creature) in creature_query.iter_mut() {
        let mut vec = Vec3::ZERO;
        let mut rotation = 0.;
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
            rotation = 1.
        }
        if keys.pressed(KeyCode::KeyE) {
            rotation = -1.
        }
        if vec != Vec3::ZERO { vec = vec.normalize(); };
        creature.target_offset = -vec * creature.speed_mult;
        let copy = transform.clone();
        transform.translation += ((copy.forward() * vec.z) + (copy.right() * vec.x))  * creature.speed_mult * 0.05;
        transform.rotate_local_y(rotation * 0.01);
    }
}

fn handle_visual(
    mut leg_creature_query: Query<(Entity, &mut Transform, &LegCreature), Without<LegCreatureVisual>>,
    children_query: Query<(&Children)>,
) {
    for  (creature_entity, mut transform, leg_creature) in leg_creature_query.iter_mut() {
        let target = transform.aligned_by(Vec3::Y, leg_creature.up, Vec3::X, transform.local_x());
        transform.rotation = transform.rotation.slerp(target.rotation, 0.05);
        let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
        let (a, b, c) = target.rotation.to_euler(EulerRot::XYZ);
    }
}

/* 
fn handle_height(
    mut leg_creature_query: Query<(Entity, &mut Transform, &mut LegCreature)>,
    mut leg_query: Query<(&IKArm::IKArm, &Name)>,
    mut gizmos: Gizmos,
) {
    'outer: for (creature_entity, mut transform, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut normal_total = Vec3::ZERO;
        let mut pos_total = Vec3::ZERO;
        let mut i = 0;
        for v in leg_creature.legs_info.iter().combinations(3) {
            let legs= [v[0].0, v[1].0, v[2].0];
            let Ok([(a, a_name), (b, b_name), (c, c_name)]) = leg_query.get_many_mut(legs) else {continue;};
            let v1 = a.target;
            let v2 = b.target;
            let v3 = c.target; 
            if (v1 == v2 || v2 == v3 || v1.is_nan() || v2.is_nan() || v3.is_nan()) {
                continue 'outer;
            }
            let (plane, pos) = InfinitePlane3d::from_points(v1, v2, v3);
            normal_total += *plane.normal;
            pos_total += pos;
            i += 1;
        }
        let normal_average = (normal_total / i as f32).normalize();
        let pos_average = pos_total / i as f32;
        let mut target_transform = *transform;
        target_transform.translation = pos_average;

        let target = target_transform.transform_point(Vec3::Y * leg_creature.target_height);
        transform.translation = transform.translation.lerp(target, 0.1);
        if !normal_average.is_nan() {
            leg_creature.up = normal_average;
        }
    };     
}
*/

fn handle_height(
    mut leg_creature_query: Query<(Entity, &mut Transform, &mut LegCreature)>,
    mut leg_query: Query<(&IKArm::IKArm, &Name)>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    names_query: Query<&Name>,
) {
    'outer: for (creature_entity, mut transform, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut normal_total = Vec3::ZERO;
        let mut pos_total = Vec3::ZERO;
        let mut i = 0;
        for v in leg_creature.legs_info.iter().combinations(3) {
            let legs= [v[0].0, v[1].0, v[2].0];
            let Ok([(a, a_name), (b, b_name), (c, c_name)]) = leg_query.get_many_mut(legs) else {continue;};
            let v1 = a.target;
            let v2 = b.target;
            let v3 = c.target; 
            if (v1 == v2 || v2 == v3 || v1.is_nan() || v2.is_nan() || v3.is_nan()) {
                continue 'outer;
            }
            let (plane, pos) = InfinitePlane3d::from_points(v1, v2, v3);
            normal_total += *plane.normal;
            pos_total += pos;
            i += 1;
        }
        let normal_average = (normal_total / i as f32).normalize();
        let settings = RaycastSettings {
            visibility: RaycastVisibility::Ignore,
            filter: &|entity| is_valid_raycast_target(entity, &names_query),
            ..default()
        };
        let origin = transform.translation;
        let ray = Ray3d::new(origin, transform.down().as_vec3());
        let hits = raycast.cast_ray(ray, &settings);
        let mut delta = 0.;
        if let Some((hit, hit_data)) = hits.first() {
            if hit_data.distance() < 0.5 {
                delta = 0.2 - hit_data.distance()
            }
        }
        transform.translation = transform.translation.lerp(transform.translation + transform.up() * delta, 0.05) ;
        if !normal_average.is_nan() {
            leg_creature.up = normal_average;
        }
    };     
}

fn determine_side(
    leg_query: Query<(&IKLeg)>,
    mut leg_creature_query: Query<(Entity, &mut LegCreature)>,
) {
    for (creature_entity, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut left_side_moving = false;
        let mut right_side_moving = false;
        for (leg_entity, leg_offset) in &leg_creature.legs_info {
            let Ok((mut leg)) = leg_query.get(*leg_entity) else {continue;};
            if leg.stepping {
                match leg.leg_side {
                    LegSide::Left => left_side_moving = true,
                    LegSide::Right => right_side_moving = true,
                    LegSide::None => {},
                }
            }
        }
        if (!left_side_moving && !right_side_moving) {
            if leg_creature.current_side == LegSide::Left {
                leg_creature.current_side = LegSide::Right;
            } else {
                leg_creature.current_side = LegSide::Left;
            }
        }
    }
}

fn handle_leg_creature(
    mut leg_query: Query<(&mut IKLeg, &mut Transform)>,
    leg_creature_query: Query<(Entity, &LegCreature, &GlobalTransform)>,
) {
    for (creature_entity, mut leg_creature, leg_creature_transform) in leg_creature_query.iter() {
        for (leg_entity, leg_offset) in &leg_creature.legs_info {
            let Ok((mut leg, mut leg_transform)) = leg_query.get_mut(*leg_entity) else {continue;};
            leg_transform.translation = leg_creature_transform.transform_point(*leg_offset);
            if leg.leg_side == leg_creature.current_side {
                leg.can_start_step = true;
            } else {
                leg.can_start_step = false;
            }
        }
    }
}

fn handle_legs(
    leg_creature_query: Query<(Entity, &LegCreature, &GlobalTransform)>,
    mut leg_query: Query<(&GlobalTransform, &mut IKArm::IKArm, &mut IKLeg)>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    names_query: Query<&Name>,
    time: Res<Time>,
    ground_query: Query<Entity, With<GroundMarker>>
) {
    for (creature_entity, mut leg_creature, leg_creature_transform) in leg_creature_query.iter() {
        for (leg_entity, leg_offset) in &leg_creature.legs_info {
            let Ok((transform,mut arm, mut leg)) = leg_query.get_mut(*leg_entity) else {continue;};
            let mut custom = Transform::from(*leg_creature_transform);
            let new_pos = leg_creature_transform.transform_point(leg_creature.target_offset);
            let new_diff = new_pos - leg_creature_transform.translation();
           let mut desired_pos: Vec3 = leg_creature_transform.transform_point(*leg_offset + leg.step_offset) + new_diff;
           let mut target = arm.target;

            if let Some(pos) = find_step(Transform::from(*leg_creature_transform), desired_pos, &mut raycast, &mut gizmos, &names_query) {
                target = pos;
            }

            let distance = arm.target.distance(target);
            if !leg.stepping {
                if distance > leg.step_distance && leg.can_start_step {
                    leg.stepping = true;
                    leg.step_elapsed = 0.;
                    leg.step_start = arm.target;
                }
            } else {
                let step_progress = leg.step_elapsed / leg.step_duration;
                arm.target = leg.step_start.lerp(target, leg.step_elapsed / leg.step_duration);
                let y_offset = (1. - ((step_progress * 2.) - 1.).abs()) * leg.step_height;
                arm.target += leg_creature_transform.up() * y_offset;
                leg.step_elapsed += time.delta_seconds();
                if leg.step_elapsed >= leg.step_duration {
                    arm.target = target;

                    leg.stepping = false;
                }
            }
            arm.up = leg_creature.up;
        }
    }
}

fn is_valid_raycast_target(entity: Entity, names_query: &Query<&Name>) -> bool {
    match names_query.get(entity) {
        Ok(name) => { name.as_str().contains("Cube") },
        Err(_) => false,
    }
}

fn find_step(
    transform: Transform,
    desired_pos: Vec3,
    raycast: &mut Raycast,
    mut gizmos: &mut Gizmos,
    names_query: &Query<&Name>
) -> Option<Vec3> {
    let mut custom = Transform::from(transform);
    custom.translation = desired_pos;
    custom.translation = custom.transform_point(Vec3::Y * 1.);
    let offsets = [
        transform.up().as_vec3() - transform.forward().as_vec3() * 1.,
        transform.up().as_vec3() + transform.forward().as_vec3() * 1.,
        transform.up().as_vec3() + transform.right().as_vec3() * 0.5,
        transform.up().as_vec3() - transform.right().as_vec3() * 0.5,
        transform.up().as_vec3(),
    ];

    let mut hits = Vec::new();
    let settings = RaycastSettings {
        visibility: RaycastVisibility::Ignore,
        filter: &|entity| is_valid_raycast_target(entity, &names_query),
        ..default()
    };

    for offset in offsets {
        let Some(pos) = try_ray(raycast, &settings, desired_pos + offset, desired_pos, Some(&mut gizmos)) else { continue; };
        if pos.distance_squared(desired_pos) < 5. {
            hits.push(pos);
        }
    }
    if hits.length() == 0 {
        return None;
    }

    hits.sort_by(|hit_a, hit_b| hit_a.distance_squared(desired_pos).partial_cmp(&hit_b.distance_squared(desired_pos)).unwrap());
    return Some(*hits.first().unwrap());
}

fn try_ray(raycast: &mut Raycast, raycast_settings: &RaycastSettings, origin: Vec3, desired_pos: Vec3, maybe_gizmos: Option<&mut Gizmos>) -> Option<Vec3> {
    let ray = Ray3d::new(origin, (desired_pos - origin).normalize());
    let hits;
    if let Some(gizmos) = maybe_gizmos  {
        hits = raycast.debug_cast_ray(ray, raycast_settings, gizmos);
    } else {
       hits = raycast.cast_ray(ray, raycast_settings);
    }
    if let Some((_, hit_data)) = hits.first() {
        return Some(hit_data.position());
    }
    return None;
}