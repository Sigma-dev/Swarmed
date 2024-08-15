use std::f32::consts::PI;
use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};

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
    pub(crate) current_side: LegSide
}


pub struct LegPlugin;

impl Plugin for LegPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (determine_side, handle_leg_creature, handle_legs).chain());
    }
}

fn determine_side(
    leg_query: Query<(&IKLeg)>,
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
    mut leg_query: Query<(&mut IKLeg)>,
    leg_creature_query: Query<(Entity, &LegCreature)>,
    children_query: Query<&Children>,
) {
    println!("SETTING LEG CAN START");
    for (creature_entity, mut leg_creature) in leg_creature_query.iter() {
        println!("CURRENT GROUP: {}", leg_creature.current_side as i8);
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
    mut leg_query: Query<(&GlobalTransform, &mut IKArm::IKArm, &mut IKLeg)>,
    time: Res<Time>,
) {
    println!("LEGS");

    //let group = get_highest_distance_group(&leg_query);
    for (transform, mut arm, mut leg) in leg_query.iter_mut() {
        let desired_pos = transform.translation() + leg.step_offset;
        let distance = arm.target.distance(desired_pos);
        println!("{} {}",  leg.leg_side as i8,leg.can_start_step,);
        if (!leg.stepping) {
            println!("DADADADA");
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