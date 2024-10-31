use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;

use crate::health::Health;

use super::weapon_raycaster::WeaponHit;

#[derive(Component)]
pub struct WeaponTarget;

pub struct WeaponTargetPlugin;

impl Plugin for WeaponTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_hits);
    }
}

fn handle_hits(
    mut weapon_events: EventReader<WeaponHit>,
    mut target_query: Query<(Entity, Option<&mut Health>)>
) {
    for event in weapon_events.read() {
        for (entity, maybe_health) in target_query.iter_mut() {
            if event.entity != entity { continue; };
            if let Some(mut health) = maybe_health {
                health.take_damage(event.damage);
            }
        }
    }
}