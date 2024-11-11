use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;
use leg_creature::{determine_side, handle_body, handle_height, handle_leg_creature, handle_up, input, LegCreature, LegSide};

pub mod leg_creature;
use crate::ik_arm;

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

#[derive(Resource)]
pub(crate) struct LegPluginSettings {
    debug_body: bool,
    debug_legs: bool
}

#[derive(Default)]
pub struct LegPlugin {
    debug_body: bool,
    debug_legs: bool
}

impl LegPlugin {
    pub fn debug() -> LegPlugin {
        LegPlugin { debug_body: true, debug_legs: true }
    }
    pub fn debug_legs() -> LegPlugin {
        LegPlugin { debug_body: false, debug_legs: true }
    }
    pub fn debug_body() -> LegPlugin {
        LegPlugin { debug_body: true, debug_legs: false }
    }
}

impl Plugin for LegPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_up, handle_body, determine_side, handle_leg_creature, handle_legs, handle_height).chain())
        .observe(setup_legs);
    
        app.insert_resource(LegPluginSettings { debug_body: self.debug_body, debug_legs: self.debug_legs });

        app.add_plugins(input::plugin);
    }
}

fn setup_legs(
    trigger: Trigger<OnAdd, IKLeg>,
    mut leg_query: Query<(&GlobalTransform, &mut ik_arm::IKArm, &IKLeg)>
  ) {
    let Ok((transform, mut arm, leg)) = leg_query.get_mut(trigger.entity()) else {return;};
    arm.target = transform.translation() + leg.step_offset;
}

fn handle_legs(
    leg_creature_query: Query<(&LegCreature, &GlobalTransform)>,
    mut leg_query: Query<(&mut ik_arm::IKArm, &mut IKLeg)>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    names_query: Query<&Name>,
    time: Res<Time>,
    plugin_settings: Res<LegPluginSettings>,
) {
    for (leg_creature, leg_creature_transform) in leg_creature_query.iter() {
        for (leg_entity, leg_offset) in &leg_creature.legs_info {
            let Ok((mut arm, mut leg)) = leg_query.get_mut(*leg_entity) else {continue;};
            let new_pos = leg_creature_transform.transform_point(leg_creature.target_offset);
            let new_diff = new_pos - leg_creature_transform.translation();
            let desired_pos: Vec3 = leg_creature_transform.transform_point(*leg_offset + leg.step_offset) + new_diff;
            let mut target = arm.target;

            if let Some(pos) = find_step(Transform::from(*leg_creature_transform), desired_pos, &mut raycast, &mut plugin_settings.debug_legs.then_some(&mut gizmos), &names_query, names_query.get(*leg_entity).unwrap().as_str().to_string()) {
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
        Ok(name) => { name.as_str().contains("Ground") },
        Err(_) => false,
    }
}

fn find_step(
    transform: Transform,
    desired_pos: Vec3,
    raycast: &mut Raycast,
    mut maybe_gizmos: &mut Option<&mut Gizmos>,
    names_query: &Query<&Name>,
    name: String,
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
        let Some(pos) = try_ray(raycast, &settings, desired_pos + offset, desired_pos, &mut maybe_gizmos) else { continue; };
        if pos.distance(desired_pos) < 2. {
            hits.push(pos);
        }
    }
    if hits.len() == 0 {
        println!("NO VALID STEPS FOUND FOR LEG: {name}");
        return None;
    }

    hits.sort_by(|hit_a, hit_b| get_ray_score(*hit_a, desired_pos).partial_cmp(&get_ray_score(*hit_b, desired_pos)).unwrap());
    return Some(*hits.first().unwrap());
}

fn get_ray_score(hit: Vec3, desired_pos: Vec3) -> f32 {
    hit.distance(desired_pos)
}

fn try_ray(raycast: &mut Raycast, raycast_settings: &RaycastSettings, origin: Vec3, desired_pos: Vec3, maybe_gizmos: &mut Option<&mut Gizmos>) -> Option<Vec3> {
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