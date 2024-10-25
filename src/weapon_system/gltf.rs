use bevy::{prelude::*, utils::HashMap};

use crate::weapon_system::weapon;

use super::{WeaponEvent, WeaponSystem};

#[derive(Component, Debug, Clone)]
pub struct WeaponVisualsGltf {
    identifier: String,
    reload_anim: AnimationNodeIndex
}

#[derive(Component)]
pub struct WeaponVisualsManagerGltf {
    pub match_list: HashMap<String, String>
}

pub(crate) fn handle_weapon_events(
    mut commands: Commands,
    assets: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf)>,
    weapons_visuals_gltf_query: Query<&WeaponVisualsGltf>,
    weapon_systems_query: Query<&WeaponSystem>,
    mut weapon_events_reader: EventReader<WeaponEvent>
) {
    for weapon_event in weapon_events_reader.read() {
        for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
            let mut children_vec: Vec<&WeaponVisualsGltf> = Vec::new();
            if let Some(children) = maybe_children {
                for child in children {
                    let visual_res = weapons_visuals_gltf_query.get(*child);
                    if let Ok(visual) = visual_res {
                        children_vec.push(visual);
                    }
                }
            }
            let Ok(weapon_system) = weapon_systems_query.get(weapon_event.system_entity) else { continue; };
            match weapon_event.event_type {
                super::WeaponEventType::StartReload => println!("Starting a reload"),
                super::WeaponEventType::Shoot => println!("Shoot"),
                super::WeaponEventType::Equip => println!("Equip")
            }
        }
    }
}

pub fn pre_spawn(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf), Added<WeaponVisualsManagerGltf>>,
) {
    for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
        commands.entity(visual_entity).with_children(|p| {
            for (identifier, gltf_path) in visual.match_list.iter() {
                p.spawn((
                    SceneBundle {
                        scene: assets_server.load(GltfAssetLabel::Scene(0).from_asset(gltf_path.clone())),
                        transform: Transform::from_xyz(0.015, -0.015, -0.06).with_scale(Vec3::splat(0.2)),
                        ..default()
                    },
                    WeaponVisualsGltf {
                        identifier: identifier.to_string(),
                        reload_anim: AnimationGraph::from_clip(assets_server.load(GltfAssetLabel::Animation(0).from_asset(gltf_path.clone()))).1,
                    }
                ));
            }
        });
    }
}

pub(crate) fn handle_weapon_spawn(
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    mut weapon_visual_query: Query<&WeaponVisualsGltf>
) {
    for (entity, mut player) in &mut players {
        let Ok(visual) = weapon_visual_query.get_single() else {continue;};
        println!("Play");
        println!("{:?}", entity);
        println!("{:?}", player.is_playing_animation(visual.reload_anim));
        let anim = player.play(visual.reload_anim);
        println!("{:?}", anim.elapsed());

    }
}
