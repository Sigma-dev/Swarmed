use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;

use crate::{character_controller::LocalPlayer, weapon_system::{WeaponEvent, WeaponEventType}};

#[derive(Component)]
pub struct WeaponRaycaster {
    pub system: Entity,
}

#[derive(Event)]
pub struct WeaponHit {
    pub entity: Entity,
    pub damage: u32,
    pub position: Option<Vec3>,
}

pub struct WeaponRaycasterPlugin;

impl Plugin for WeaponRaycasterPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, handle_events)
        .add_event::<WeaponHit>()
        ;
    }
}

fn handle_events(
    mut weapon_events: EventReader<WeaponEvent>,
    raycaster_query: Query<(&GlobalTransform, &WeaponRaycaster)>,
    player_query: Query<Entity, With<LocalPlayer>>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    mut hit_events: EventWriter<WeaponHit>,
) {
    for event in weapon_events.read() {
        if !event.authentic { return; };
        if let WeaponEventType::Shoot(damage, _) = event.event_type {
            for (raycaster_transform, raycaster) in raycaster_query.iter() {
                if raycaster.system != event.system_entity { return; };
                let ray = Ray3d::new(raycaster_transform.translation(), *raycaster_transform.forward());
                let player = player_query.single();
                let f = |hit| player != hit;
                let settings = RaycastSettings::default().always_early_exit().with_filter(&f);
                let maybe_hit = raycast.debug_cast_ray(ray, &settings, &mut gizmos).first();
                if let Some((hit_entity, hit)) = maybe_hit {
                    hit_events.send(WeaponHit { entity: *hit_entity, damage, position: Some(hit.position()) });
                }
            }
        }
    }
}