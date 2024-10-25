use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, utils::HashMap};

use crate::weapon_system::weapon;

use super::{WeaponEvent, WeaponSystem};

#[derive(Component, Debug, Clone)]
pub struct WeaponVisualsGltf {
    identifier: String,
    graph: AnimationGraph,
    animations: Vec<AnimationNodeIndex>
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
    mut commands: Commands,
    animations: Res<Animations>,
    // The "Added<AnimationPlayer>" filter means this system only gets run the first time an AnimationPlayer component is added
    // to a given entity. That ensures that we'll also only have one "AnimationTransitions" component, which we create here.
    // We spawn it and immediately play animations.animations[0] which is your equipped animation
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions.play(&mut player, animations.animations[0], Duration::ZERO);
        commands.
            entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

pub fn play_anim(
    mut commands: &mut Commands,
    // We now include AnimationTransitions in the query.
    mut players: &mut Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions)>,
    mut weapon_visual_query: &mut Query<(&WeaponVisualsGltf, &mut Visibility)>,
    animations: &Res<Animations>,
    anim_index: usize
) {
    for (entity, mut player, mut transitions) in players {
        for (visual, mut visibility) in weapon_visual_query.iter_mut() {
            *visibility = Visibility::Visible;
        }

        // We use transitions to play. Don't spawn it every time.
        transitions.play(&mut player, animations.animations[anim_index], Duration::ZERO);
        // let mut transitions = AnimationTransitions::new();
        // transitions
        //     .play(&mut player, animations.animations[anim_index], Duration::ZERO);
        // //2 = equip
        // commands
        //     .entity(entity)
        //     .insert(animations.graph.clone())
        //     .insert(transitions);
    }
}

pub(crate) fn handle_weapon_events(
    mut commands: Commands,
    assets: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf)>,
    mut weapons_visuals_gltf_query: Query<(&WeaponVisualsGltf, &mut Visibility)>,
    weapon_systems_query: Query<&WeaponSystem>,
    // This query is modified to include the AnimationTransitions component
    mut players: Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions)>,
    mut weapon_events_reader: EventReader<WeaponEvent>,
    animations: Res<Animations>,
) {
    for weapon_event in weapon_events_reader.read() {
        for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
            let mut children_vec: Vec<&WeaponVisualsGltf> = Vec::new();
            if let Some(children) = maybe_children {
                for child in children {
                    let visual_res = weapons_visuals_gltf_query.get(*child);
                    if let Ok((visual, visibility)) = visual_res {
                        children_vec.push(visual);
                    }
                }
            }
            let Ok(weapon_system) = weapon_systems_query.get(weapon_event.system_entity) else { continue; };
            match &weapon_event.event_type {
                super::WeaponEventType::StartReload(reload_type) => play_anim(&mut commands, &mut players, &mut weapons_visuals_gltf_query, &animations, 
                    match reload_type {
                        super::weapon_inventory::ReloadType::Normal => 1,
                        super::weapon_inventory::ReloadType::Empty => 2,
                    }
                ),
                super::WeaponEventType::Shoot(shoot_type) => play_anim(&mut commands, &mut players, &mut weapons_visuals_gltf_query, &animations,
                    match shoot_type {
                        super::weapon_inventory::ShootType::Normal => 3,
                        super::weapon_inventory::ShootType::Last => 4,
                    }
                ),
                super::WeaponEventType::Equip =>  play_anim(&mut commands, &mut players, &mut weapons_visuals_gltf_query, &animations, 0),
            }
        }
    }
}

pub fn pre_spawn(
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    weapons_visuals_manager_gltf_query: Query<(Entity, Option<&Children>, &WeaponVisualsManagerGltf), Added<WeaponVisualsManagerGltf>>,
) {
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
        println!("insert");
        let graph = graphs.add(graph);
        commands.insert_resource(Animations {
            animations,
            graph: graph.clone(),
        });
    for (visual_entity, maybe_children, visual) in weapons_visuals_manager_gltf_query.iter() {
        println!("Damso");
        commands.entity(visual_entity).with_children(|p| {
            for (identifier, gltf_path) in visual.match_list.iter() {
                let mut graph = AnimationGraph::new();
                p.spawn((
                    SceneBundle {
                        scene: assets_server.load(GltfAssetLabel::Scene(0).from_asset(gltf_path.clone())),
                        transform: Transform::from_xyz(0.015, -0.015, -0.05).with_scale(Vec3::splat(0.2)),
                     //transform: Transform::from_xyz(0., -0.015 * 5., -0.2).with_rotation(Quat::from_rotation_y(PI)),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    WeaponVisualsGltf {
                        identifier: identifier.to_string(),
                        //animations: graph.add_clips((0..2).map(|i| {assets_server.load(GltfAssetLabel::Animation(i).from_asset(gltf_path))}), 1.0,graph.root).collect(),
                        //animations: AnimationGraph::from_clip(assets_server.load(GltfAssetLabel::Animation(0).from_asset(gltf_path.clone())))
                        graph: graph.to_owned(),
                        animations: graph
                        .add_clips(
                            [
                                GltfAssetLabel::Animation(2).from_asset(gltf_path.clone()),
                                GltfAssetLabel::Animation(1).from_asset(gltf_path.clone()),
                                GltfAssetLabel::Animation(0).from_asset(gltf_path.clone()),
                            ]
                            .into_iter()
                            .map(|path| assets_server.load(path)),
                            1.0,
                            graph.root,
                        )
                        .collect()
                    }
                ));
            }
        });
    }
}

/*
pub(crate) fn handle_weapon_spawn(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    mut weapon_visual_query: Query<&WeaponVisualsGltf>,
    animations: Res<Animations>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
    /*
    for (entity, mut player) in &mut players {
        let Ok(visual) = weapon_visual_query.get_single() else {continue;};
        println!("Play");
        println!("{:?}", entity);
        println!("{:?}", player.is_playing_animation(visual.animations[0]));
        //let anim = player.play(visual.animations[0]);
       //println!("{:?}", anim.is_paused());
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, visual.animations[0], Duration::ZERO)
            .repeat();
        commands
        .entity(entity)
        .insert(visual.graph.clone())
        .insert(transitions);
    } */
}
 */