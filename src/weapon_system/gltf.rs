use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, utils::HashMap};

use crate::{animated_gltf::AnimatedGltf, weapon_system::weapon};

use super::{WeaponEvent, WeaponSystem};

#[derive(Component, Debug, Clone)]
pub struct WeaponVisualsGltf {
    identifier: String,
}

#[derive(Component)]
pub struct WeaponVisualsManagerGltf {
    pub match_list: HashMap<String, String>
}

#[derive(Resource)]
pub struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

pub fn setup_anims(
   // animations: Res<Animations>,
    // The "Added<AnimationPlayer>" filter means this system only gets run the first time an AnimationPlayer component is added
    // to a given entity. That ensures that we'll also only have one "AnimationTransitions" component, which we create here.
    // We spawn it and immediately play animations.animations[0] which is your equipped animation
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
      //  transitions.play(&mut player, animations.animations[0], Duration::ZERO);
        //commands.entity(entity).insert(AnimationGraph::new().clone());
       // println!("gltf: {:?}", graphs.get(animations.graph.id()).unwrap());
       // commands.entity(entity).insert(animations.graph.clone());
       
         
    }
}

pub fn play_anim(
    mut players: &mut Query<(Entity, &mut AnimatedGltf, &WeaponVisualsGltf)>,
    //mut weapon_visual_query: &mut Query<(Entity, &WeaponVisualsGltf, &mut Visibility)>,
    anim: impl Into<String>
) {
    let str = anim.into();
    for (e, mut animated, visual) in players.iter_mut() {
        /* for (e2, visual, mut visibility) in weapon_visual_query.iter_mut() {
            *visibility = Visibility::Visible;
            println!("{} {}", e, e2);
        } */
        animated.play(str.clone());
        //transitions.play(&mut player, animations.animations[anim_index], Duration::ZERO);

    }
}

pub(crate) fn handle_weapon_events(
    mut commands: Commands,
    assets: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf)>,
    mut weapons_visuals_gltf_query: Query<(Entity, &WeaponVisualsGltf, &mut Visibility)>,
    weapon_systems_query: Query<&WeaponSystem>,
    mut players: Query<(Entity, &mut AnimatedGltf, &WeaponVisualsGltf)>,
    mut weapon_events_reader: EventReader<WeaponEvent>,
) {
    for weapon_event in weapon_events_reader.read() {
        for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
            let mut children_vec: Vec<&WeaponVisualsGltf> = Vec::new();
            if let Some(children) = maybe_children {
                for child in children {
                    let visual_res = weapons_visuals_gltf_query.get(*child);
                    if let Ok((e, visual, visibility)) = visual_res {
                        children_vec.push(visual);
                    }
                }
            }
            let Ok(weapon_system) = weapon_systems_query.get(weapon_event.system_entity) else { continue; };
            play_anim(&mut players, match &weapon_event.event_type {
                super::WeaponEventType::StartReload(reload_type) => match reload_type {
                        super::weapon_inventory::ReloadType::Normal => "Reload",
                        super::weapon_inventory::ReloadType::Empty => "ReloadEmpty",
                    }
                super::WeaponEventType::Shoot(shoot_type) => match shoot_type {
                        super::weapon_inventory::ShootType::Normal => "Shoot",
                        super::weapon_inventory::ShootType::Last => "ShootLast",
                    }
                super::WeaponEventType::Equip =>  "Equip",
            });
        }
    }
}

pub fn pre_spawn(
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf), Added<WeaponVisualsManagerGltf>>,
) {
    
    for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
        commands.entity(visual_entity).with_children(|p: &mut ChildBuilder<'_>| {
            for (identifier, gltf_path) in visual.match_list.iter() {
                p.spawn((
                    SpatialBundle {
                        transform: Transform::from_xyz(0.015, -0.015, -0.05).with_scale(Vec3::splat(0.2)),
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
    return;
    let mut graph = AnimationGraph::new();
    let animations: Vec<AnimationNodeIndex> = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(0).from_asset("weapons/glock/glock.glb"),
                GltfAssetLabel::Animation(1).from_asset("weapons/glock/glock.glb"),
                GltfAssetLabel::Animation(2).from_asset("weapons/glock/glock.glb"),
                GltfAssetLabel::Animation(3).from_asset("weapons/glock/glock.glb"),
                GltfAssetLabel::Animation(4).from_asset("weapons/glock/glock.glb"),
                GltfAssetLabel::Animation(5).from_asset("weapons/glock/glock.glb"),
            ]
            .into_iter()
            .map(|path| assets_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();
        let graph = graphs.add(graph);
        commands.insert_resource(Animations {
            animations,
            graph: graph.clone(),
        });
    for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
        commands.entity(visual_entity).with_children(|p| {
            for (identifier, gltf_path) in visual.match_list.iter() {
                let mut graph = AnimationGraph::new();
                p.spawn((
                    SceneBundle {
                        scene: assets_server.load(GltfAssetLabel::Scene(0).from_asset(gltf_path.clone())),
                        transform: Transform::from_xyz(0.015, -0.015, -0.05).with_scale(Vec3::splat(0.2)),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    WeaponVisualsGltf {
                        identifier: identifier.to_string(),
                    }
                ));
            }
        });
    }

}