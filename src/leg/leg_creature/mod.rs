use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;
use crate::debug_resource::DebugResource;
use super::{is_valid_raycast_target, IKLeg, LegPluginSettings};

pub mod input;
pub mod target;

#[derive(Component)]
pub struct LegCreature {
    pub(crate) current_side: LegSide,
    pub target_height: f32,
    pub(crate) up: Vec3,
    pub legs_info: Vec<(Entity, Vec3)>,
    pub speed_mult: f32,
    pub(crate) target_offset: Vec3,
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

#[derive(Copy, Clone, PartialEq, Default)]
pub enum LegSide {
    Left,
    Right,
    #[default] None,
}

impl LegSide {
    pub fn to_string(&self) -> String {
        match self {
            LegSide::Left => "Left".to_string(),
            LegSide::Right => "Right".to_string(),
            LegSide::None => "None".to_string(),
        }
    }
}

pub(crate) fn handle_leg_creature(
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

pub(crate) fn determine_side(
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

pub(crate) fn handle_body(
    mut leg_creature_query: Query<(&mut Transform, &LegCreature), Without<LegCreatureVisual>>,
) {
    for  (mut transform, leg_creature) in leg_creature_query.iter_mut() {
        let target = transform.aligned_by(Vec3::Y, leg_creature.up, Vec3::X, transform.local_x());
        transform.rotation = transform.rotation.slerp(target.rotation, 0.1);
    }
}

pub(crate) fn handle_height(
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
        let straight_down = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3()), &settings, plugin_settings.debug_body.then_some(&mut gizmos), None);
        let bit_behind = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3() + (-transform.forward() * 0.5)), &settings, plugin_settings.debug_body.then_some(&mut gizmos), None);
        let bit_front = raycast_first(&mut raycast, Ray3d::new(origin, transform.down().as_vec3() + (transform.forward() * 0.5)), &settings, plugin_settings.debug_body.then_some(&mut gizmos), None);
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

pub(crate) fn handle_up(
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
        if let Some(hit) = get_wall_hit_data(&mut raycast, &settings, *transform, plugin_settings.debug_body.then_some(&mut gizmos), plugin_settings.debug_body.then_some(&mut debug_resource)) {
            if plugin_settings.debug_body { println!("Hit wall at distance {}", hit.distance()); }
            target_up = leg_creature.up.lerp(hit.normal(), hit.distance());
        }
        else if let Some(hit) = get_cliff_data(&mut raycast, &settings, *transform, plugin_settings.debug_body.then_some(&mut gizmos), plugin_settings.debug_body.then_some(&mut debug_resource)) {
            if plugin_settings.debug_body { println!("Hit cliff"); }
            target_up = leg_creature.up.lerp(hit.normal(), 0.2);
        }
        else if let Some(ground_normal) = get_ground_normal(&mut raycast, &settings, *transform, plugin_settings.debug_body.then_some(&mut gizmos),  plugin_settings.debug_body.then_some(&mut debug_resource)) {
            if plugin_settings.debug_body { println!("Hit ground"); }
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
           debug.debug(*entity, "HIT RAYCAST");
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

