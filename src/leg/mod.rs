use bevy::{prelude::*, render::render_resource::encase::rts_array::Length};
use bevy_mod_raycast::prelude::*;

use crate::{debug_resource::DebugResource, ik_arm};
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
pub struct LegCreatureVisual;

#[derive(Resource)]
struct LegPluginSettings {
    debug: bool
}

#[derive(Default)]
pub struct LegPlugin {
    debug: bool
}

impl LegPlugin {
    pub fn debug() -> LegPlugin {
        LegPlugin { debug: true }
    }
}

impl Plugin for LegPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_up, handle_body, determine_side, handle_leg_creature, handle_legs, move_creature, handle_height, ).chain())
        .observe(setup_legs);
    
        app.insert_resource(LegPluginSettings { debug: self.debug });
    }
}

fn setup_legs(
    trigger: Trigger<OnAdd, IKLeg>,
    mut leg_query: Query<(&GlobalTransform, &mut ik_arm::IKArm, &IKLeg)>
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
        creature.target_offset = (-vec * creature.speed_mult * 1.).clamp_length(0., 1.);
        let copy = transform.clone();
        transform.translation += ((copy.forward() * vec.z) + (copy.right() * vec.x))  * creature.speed_mult * 0.05;
        transform.rotate_local_y(rotation * 0.01);
    }
}

fn handle_body(
    mut leg_creature_query: Query<(&mut Transform, &LegCreature), Without<LegCreatureVisual>>,
) {
    for  (mut transform, leg_creature) in leg_creature_query.iter_mut() {
        let target = transform.aligned_by(Vec3::Y, leg_creature.up, Vec3::X, transform.local_x());
        transform.rotation = transform.rotation.slerp(target.rotation, 0.1);
    }
}

fn handle_height(
    mut leg_creature_query: Query<(&mut Transform, &LegCreature)>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    names_query: Query<&Name>,
    plugin_settings: Res<LegPluginSettings>,
) {
    for ( mut transform, leg_creature) in leg_creature_query.iter_mut() {
        let settings = RaycastSettings {
            visibility: RaycastVisibility::Ignore,
            filter: &|entity| is_valid_raycast_target(entity, &names_query),
            ..default()
        };
        let mut delta = 0.;
        let origin = transform.translation;
        let straight_down = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3()), &settings, plugin_settings.debug.then_some(&mut gizmos), None);
        let bit_behind = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3() + (-transform.forward() * 0.5)), &settings, plugin_settings.debug.then_some(&mut gizmos), None);
        let bit_front = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3() + (transform.forward() * 0.5)), &settings, plugin_settings.debug.then_some(&mut gizmos), None);
        if let Some(down) = straight_down {
            if down.distance() < 0.5 {
                delta = leg_creature.target_height - down.distance()
            }
        } else if let Some(behind) = bit_behind {
            if behind.distance() < 0.5 {
                delta = leg_creature.target_height - behind.distance()
            }
        } else if let Some(front) = bit_front {
            if front.distance() < 0.5 {
                delta = leg_creature.target_height - front.distance()
            }
        }
        transform.translation = transform.translation.lerp(transform.translation + transform.up() * delta, 0.05) ;
    };     
}

fn handle_up(
    mut raycast: Raycast,
    mut leg_creature_query: Query<(&mut Transform, &mut LegCreature)>,
    names_query: Query<&Name>,
    mut gizmos: Gizmos,
    plugin_settings: Res<LegPluginSettings>,
    mut debug_resource: ResMut<DebugResource>
) {
    for (mut transform, mut leg_creature) in leg_creature_query.iter_mut() {
        let settings = RaycastSettings {
            visibility: RaycastVisibility::Ignore,
            filter: &|entity| is_valid_raycast_target(entity, &names_query),
            ..default()
        };
        let mut target_up = leg_creature.up;
        if let Some(hit) = get_wall_hit_data(&mut raycast, &settings, *transform, plugin_settings.debug.then_some(&mut gizmos), Some(&mut debug_resource)) {
            println!("Hit wall at distance {}", hit.distance());
            target_up = leg_creature.up.lerp(hit.normal(), hit.distance());
        }
        else if let Some(hit) = get_cliff_data(&mut raycast, &settings, *transform, plugin_settings.debug.then_some(&mut gizmos), Some(&mut debug_resource)) {
            println!("Hit cliff");
            target_up = leg_creature.up.lerp(hit.normal(), 0.2);
        }
        else if let Some(ground_normal) = get_ground_normal(&mut raycast, &settings, *transform, plugin_settings.debug.then_some(&mut gizmos),  Some(&mut debug_resource)) {
            println!("Hit ground");
            target_up = ground_normal
        }
        let mut copy = transform.clone();
        copy.rotation = Quat::from_rotation_arc(*copy.up(), target_up) * copy.rotation;
        transform.rotation = transform.rotation.lerp(copy.rotation, 0.1);
        leg_creature.up = *transform.up();
    };     
}

fn raycast_first(raycast: &mut Raycast, ray: Ray3d, raycast_settings: &RaycastSettings, maybe_gizmos: Option<&mut Gizmos>, maybe_debug: Option<&mut ResMut<DebugResource>>) -> Option<IntersectionData> {
    let hits;
    if let Some(gizmos) = maybe_gizmos {
        hits = raycast.debug_cast_ray(ray, raycast_settings, gizmos)
    } else {
        hits = raycast.cast_ray(ray, raycast_settings)
    }
    if let Some(debug) = maybe_debug {
        if let Some((entity, _)) = hits.first() {
           // debug.debug(*entity, "HIT RAYCAST");
        }
    }
    hits.first().map(|h| h.1.clone())
}

fn get_ground_normal(raycast: &mut Raycast, raycast_settings: &RaycastSettings, transform: Transform, maybe_gizmos: Option<&mut Gizmos>, maybe_debug: Option<&mut ResMut<DebugResource>>) -> Option<Vec3> {
    let ray = Ray3d::new(transform.translation, transform.down().as_vec3());
    if let Some(hit_data) = raycast_first(raycast, ray, raycast_settings, maybe_gizmos, maybe_debug) {
        if hit_data.distance() < 0.5 {
            return Some(hit_data.normal());
        }
    }
    return None;
}

fn get_wall_hit_data(raycast: &mut Raycast, raycast_settings: &RaycastSettings, transform: Transform, maybe_gizmos: Option<&mut Gizmos>, maybe_debug: Option<&mut ResMut<DebugResource>>) -> Option<IntersectionData> {
    let ray = Ray3d::new(transform.translation, transform.forward().as_vec3());
    if let Some(hit_data) = raycast_first(raycast, ray, raycast_settings, maybe_gizmos, maybe_debug) {
        if hit_data.distance() < 1. {
            return Some(hit_data.clone());
        }
    }
    return None
}

fn get_cliff_data(raycast: &mut Raycast, raycast_settings: &RaycastSettings, transform: Transform, maybe_gizmos: Option<&mut Gizmos>, maybe_debug: Option<&mut ResMut<DebugResource>>) -> Option<IntersectionData> {
    let ray = Ray3d::new(transform.translation + transform.forward().as_vec3() * 1.0, transform.down().as_vec3() - transform.forward().as_vec3());
    if let Some(hit_data) = raycast_first(raycast, ray, raycast_settings, maybe_gizmos, maybe_debug) {
        if hit_data.distance() < 2. {
            return Some(hit_data.clone());
        }
    }
    return None
}

fn determine_side(
    leg_query: Query<&IKLeg>,
    mut leg_creature_query: Query<&mut LegCreature>,
) {
    for mut leg_creature in leg_creature_query.iter_mut() {
        let mut left_side_moving = false;
        let mut right_side_moving = false;
        for (leg_entity, _) in &leg_creature.legs_info {
            let Ok(leg) = leg_query.get(*leg_entity) else {continue;};
            if leg.stepping {
                match leg.leg_side {
                    LegSide::Left => left_side_moving = true,
                    LegSide::Right => right_side_moving = true,
                    LegSide::None => {},
                }
            }
        }
        if !left_side_moving && !right_side_moving {
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
    leg_creature_query: Query<(&LegCreature, &GlobalTransform)>,
) {
    for (leg_creature, leg_creature_transform) in leg_creature_query.iter() {
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
    leg_creature_query: Query<(&LegCreature, &GlobalTransform)>,
    mut leg_query: Query<(&mut ik_arm::IKArm, &mut IKLeg)>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    names_query: Query<&Name>,
    time: Res<Time>,
) {
    for (leg_creature, leg_creature_transform) in leg_creature_query.iter() {
        for (leg_entity, leg_offset) in &leg_creature.legs_info {
            let Ok((mut arm, mut leg)) = leg_query.get_mut(*leg_entity) else {continue;};
            let new_pos = leg_creature_transform.transform_point(leg_creature.target_offset);
            let new_diff = new_pos - leg_creature_transform.translation();
            let desired_pos: Vec3 = leg_creature_transform.transform_point(*leg_offset + leg.step_offset) + new_diff;
            let mut target = arm.target;

            if let Some(pos) = find_step(Transform::from(*leg_creature_transform), desired_pos, &mut raycast, &mut gizmos, &names_query, names_query.get(*leg_entity).unwrap().as_str().to_string()) {
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
    mut _gizmos: &mut Gizmos,
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
        let Some(pos) = try_ray(raycast, &settings, desired_pos + offset, desired_pos, None) else { continue; };
        if pos.distance(desired_pos) < 2. {
            hits.push(pos);
        }
    }
    if hits.length() == 0 {
        println!("NO VALID STEPS FOUND FOR LEG: {name}");
        return None;
    }

    hits.sort_by(|hit_a, hit_b| get_ray_score(*hit_a, desired_pos).partial_cmp(&get_ray_score(*hit_b, desired_pos)).unwrap());
    return Some(*hits.first().unwrap());
}

fn get_ray_score(hit: Vec3, desired_pos: Vec3) -> f32 {
    hit.distance(desired_pos)
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