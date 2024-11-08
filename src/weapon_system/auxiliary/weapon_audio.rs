use bevy::prelude::*;

use crate::{audio_manager::PlayAudio3D, weapon_system::{WeaponEvent, WeaponSystem}};

pub struct WeaponAudioPlugin;

impl Plugin for WeaponAudioPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, handle_weapon_audio);
    }
}

fn handle_weapon_audio(
    weapon_system_query: Query<(Entity, &GlobalTransform, &WeaponSystem)>,
    mut audio_events: EventWriter<PlayAudio3D>,
    mut weapon_events: EventReader<WeaponEvent>,
) {
    for weapon_event in weapon_events.read() {
        let (system_entity, system_gt, system) = weapon_system_query.get(weapon_event.system_entity).unwrap();
        let tag = system.inventory.get_visual_identifier(weapon_event.weapon_index).unwrap();
        let maybe_sound = match &weapon_event.event_type {
            crate::weapon_system::WeaponEventType::Shoot(_, _) => Some("shoot"),
            crate::weapon_system::WeaponEventType::StartReload(_) => Some("reload"),
            crate::weapon_system::WeaponEventType::Equip => Some("equip"),
        };
        if let Some(sound) = maybe_sound {
            let path = format!("weapons/{tag}/sounds/{sound}.mp3");
            audio_events.send(PlayAudio3D { path, position: system_gt.translation(), volume_mut: 1., one_shot: true, follow: Some(system_entity)});
        }
    }
}