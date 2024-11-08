use bevy::{prelude::*, utils::HashMap, animation::animate_targets};
use crate::{animated_gltf::AnimatedGltf, weapon_system::{weapon_inventory::{ReloadType, ShootType}, WeaponEvent, WeaponEventType, WeaponSystem}};

#[derive(Component, Debug, Clone)]
pub struct WeaponVisualsGltf {
    identifier: String,
}

#[derive(Component)]
pub struct WeaponVisualsManagerGltf {
    pub match_list: HashMap<String, String>,
    pub system: Entity,
}

pub struct WeaponsGltfPlugin;

impl Plugin for WeaponsGltfPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            pre_spawn,
            handle_weapon_events.before(animate_targets))
        );
    }
}

pub(crate) fn handle_weapon_events(
    mut weapon_events_reader: EventReader<WeaponEvent>,
    weapons_visuals_manager_gltf_query: Query<(Entity, &WeaponVisualsManagerGltf)>,
    weapon_systems_query: Query<Entity, With<WeaponSystem>>,
    mut players: Query<&mut AnimatedGltf, With<WeaponVisualsGltf>>,
    children_query: Query<&Children>,
) {
    for weapon_event in weapon_events_reader.read() {
        for (manager_entity, visual) in weapons_visuals_manager_gltf_query.iter() {
            let Ok(system_entity) = weapon_systems_query.get(weapon_event.system_entity) else { continue; };
            if system_entity != visual.system { continue; };
            let animation_name = weapon_event_to_animation_name(weapon_event);
            for child in children_query.iter_descendants(manager_entity) {
                if let Ok(mut animated) = players.get_mut(child) {
                    animated.play(animation_name.clone());
                }
            }
        }
    }
}

pub fn pre_spawn(
    mut commands: Commands,
    weapons_visuals_manager_gltf_query: Query<(Entity, &WeaponVisualsManagerGltf), Added<WeaponVisualsManagerGltf>>,
) {
    for (visual_entity, visual) in weapons_visuals_manager_gltf_query.iter() {
        commands.entity(visual_entity).with_children(|p: &mut ChildBuilder<'_>| {
            for (identifier, gltf_path) in visual.match_list.iter() {
                p.spawn((
                    SpatialBundle {
                        transform: Transform::from_xyz(0.1, -0.075, -0.25),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    AnimatedGltf::new(gltf_path),
                    WeaponVisualsGltf {
                        identifier: identifier.to_string(),
                    }
                ));
            }
        });
    }
}

fn weapon_event_to_animation_name(weapon_event: &WeaponEvent) -> String {
    match &weapon_event.event_type {
        WeaponEventType::StartReload(reload_type) => match reload_type {
                ReloadType::Normal => "Reload",
                ReloadType::Empty => "ReloadEmpty",
            }
        WeaponEventType::Shoot(_, shoot_type) => match shoot_type {
                ShootType::Normal => "Shoot",
                ShootType::Last => "ShootLast",
            }
        WeaponEventType::Equip =>  "Equip",
    }.to_string()
}