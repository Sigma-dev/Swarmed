use std::f32::consts::PI;
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, reflect::Array, render::mesh::{self, skinning::SkinnedMesh}};
use bevy_mod_raycast::prelude::*;
use itertools::Itertools;

use crate::IKArm;
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
    pub legs_info: Vec<(Entity, Vec3)>
}
impl LegCreature {
    pub fn new(
        current_side: LegSide,
        target_height: f32,
        legs_info: Vec<(Entity, Vec3)>
    ) -> Self {
        Self { current_side, target_height, up: Vec3::Y, legs_info}
    }
}

#[derive(Component)]
pub struct LegCreatureVisual {
}

pub struct LegPlugin;

impl Plugin for LegPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_height, handle_visual, determine_side, handle_leg_creature, handle_legs).chain());
    }
}

/*
fn handle_height(
    leg_query: Query<&IKArm::IKArm>,
    mut leg_creature_query: Query<(Entity, &mut Transform, &mut LegCreature)>,
    children_query: Query<&Children>,
) {
    for (creature_entity, mut transform, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut sum = Vec3::ZERO;
        let mut n = 0;
        for child in children_query.iter_descendants(creature_entity) {
            let Ok((mut arm)) = leg_query.get(child) else {continue;};
            sum += arm.target;
            n += 1;
        }
        let median = sum / n as f32;
        transform.translation.y = transform.translation.y.lerp(median.y + leg_creature.target_height, 0.1);
    }
}
 */

fn handle_visual(
    mut leg_creature_query: Query<(Entity, &mut Transform, &LegCreature), Without<LegCreatureVisual>>,
    children_query: Query<(&Children)>,
) {
    for  (creature_entity, mut transform, leg_creature) in leg_creature_query.iter_mut() {
        let target = transform.aligned_by(-Vec3::Y, leg_creature.up, Vec3::X, transform.local_x());
        transform.rotation = transform.rotation.slerp(target.rotation, 0.1);
    }
}

fn handle_height(
    mut leg_creature_query: Query<(Entity, &mut Transform, &mut LegCreature)>,
    mut leg_query: Query<(&IKArm::IKArm, &Name)>,

) {
    'outer: for (creature_entity, mut transform, mut leg_creature) in leg_creature_query.iter_mut() {
        let mut normal_total = Vec3::ZERO;
        let mut pos_total = Vec3::ZERO;
        let mut i = 0;
        for v in leg_creature.legs_info.iter().combinations(3) {
            let legs= [v[0].0, v[1].0, v[2].0];
            let Ok([(a, a_name), (b, b_name), (c, c_name)]) = leg_query.get_many_mut(legs) else {continue;};
            println!("{} {} {}", a_name, b_name, c_name);
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
        let normal_average = normal_total / i as f32;
        let pos_average = pos_total / i as f32;
        transform.translation.y = transform.translation.y.lerp(pos_average.y + leg_creature.target_height, 0.1);
        if (!normal_average.normalize().is_nan()) {
            leg_creature.up = normal_average.normalize();
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
            leg_transform.translation = leg_creature_transform.translation() + *leg_offset;
            if (leg.leg_side == leg_creature.current_side) {
                leg.can_start_step = true;
            } else {
                leg.can_start_step = false;
            }
        }
    }
}

fn handle_legs(
    mut leg_query: Query<(&GlobalTransform, &mut IKArm::IKArm, &mut IKLeg)>,
    mut raycast: Raycast,
    time: Res<Time>,
) {
    //let group = get_highest_distance_group(&leg_query);
    for (transform, mut arm, mut leg) in leg_query.iter_mut() {
        let mut desired_pos: Vec3 = transform.translation() + leg.step_offset;
        let dir = (desired_pos - transform.translation()).normalize();
        let ray = Ray3d::new(desired_pos + Vec3::Y, Vec3::NEG_Y);
        //let ray = Ray3d::new(desired_pos, -dir);
        let hits = raycast.cast_ray(ray, &RaycastSettings::default());
        if let Some((hit, hit_data)) = hits.first() {
         //   println!("{}", hit_data.position().y);
            desired_pos = hit_data.position();
        }

        let distance = arm.target.distance(desired_pos);
        if (!leg.stepping) {
            if (distance > leg.step_distance && leg.can_start_step) {
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
    mut leg_query: &Query<(&GlobalTransform, &mut IKArm::IKArm, &mut IKLeg)>,
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