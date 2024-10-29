use std::time::Duration;

use bevy::{prelude::*, utils::{hashbrown::HashMap}};
use itertools::Itertools;

pub struct AnimatedGltfPlugin;

impl Plugin for AnimatedGltfPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (handle_new, handle_loaded,handle_loaded_anims, handle_new_anim));
    }
}

#[derive(Component)]
pub struct AnimatedGltf {
    gltf: Handle<Gltf>,
    gltf_path: String,
    animations: HashMap<String, AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
    currently_playing: Option<String>,
    updated: bool,
    loaded: bool,
}

impl AnimatedGltf {
    pub fn new(gltf_path: impl Into<String>) -> AnimatedGltf {
        AnimatedGltf {
            gltf_path: gltf_path.into(),
            gltf: Handle::default(),
            animations: HashMap::new(),
            currently_playing: None,
            updated: false,
            loaded: false,
            graph: Handle::default()
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        self.currently_playing = Some(name.into());
        self.updated = true;
    }
}

fn handle_new(
    asset_server: Res<AssetServer>,
    mut animated_query: Query<&mut AnimatedGltf, Added<AnimatedGltf>>
) {
    for mut animated in animated_query.iter_mut() {
        animated.gltf = asset_server.load(&animated.gltf_path);
    }
}

fn handle_loaded(
    mut commands: Commands,
    mut gltf_events: EventReader<AssetEvent<Gltf>>,
    mut animated_query: Query<(Entity, &mut AnimatedGltf)>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>
) {
    for (animated_entity, mut animated) in animated_query.iter_mut() {
        if animated.loaded { continue; };
        if !asset_server.is_loaded_with_dependencies(animated.gltf.id()) { continue; };
        animated.loaded = true;
        let Some(gltf) = assets_gltf.get(animated.gltf.id()) else { continue; };
        commands.entity(animated_entity).with_children(|p| {
            p.spawn((SceneBundle {
                scene: gltf.scenes[0].clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            }, Name::new("ModelGlock")));
        });
        let mut graph = AnimationGraph::new();
       /*  let animations: Vec<AnimationNodeIndex> = graph.add_clips(
            gltf.named_animations.iter().map(|(_, handle)| handle.clone()),
            1.0,
            graph.root,
        )
        .collect();*/
        let cloned_animations = gltf.named_animations.clone();
        for (i, (handle)) in gltf.animations.clone().into_iter().enumerate() {
           // let mut maybe_name = None;
            for (name, clip) in gltf.named_animations.clone() {
                if clip.id() == handle.id() {
                    println!("{}", name);
                   // maybe_name = Some(name);
                    animated.animations.insert(name.to_string(),graph.add_clip(clip, 1.0, graph.root));
                }
            }
            //maybe_name = gltf.named_animations.clone().iter().find(|n| n.1.id() == handle.id()).map(|a| a.0).cloned();
           // let Some(name) = maybe_name else { continue; };
            //animated.animations.insert(name.to_string(), animations[i]);
        }
        animated.graph = graphs.add(graph).clone();
/*         commands.
        entity(animated_entity)
        .insert()
        .insert(AnimationTransitions::new());
 */    }
}

pub fn handle_loaded_anims(
    mut commands: Commands,
    mut animated_query: Query<(Entity, &mut AnimatedGltf)>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents: Query<&Parent>,
    names: Query<&Name>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, mut player) in &mut players {
        let Ok(fp) = parents.get(entity) else { continue; };
        let Ok(sp) = parents.get(fp.get()) else { continue; };
        let Ok(tp) = parents.get(sp.get()) else { continue; };
        let Ok((e, animated)) = animated_query.get(tp.get()) else { continue; };
        commands.entity(entity).insert(animated.graph.clone()).insert(AnimationTransitions::new());
        
    }
}

fn handle_new_anim(
    commands: Commands,
    mut child_query: Query<&Children>,
    mut animated_query: Query<(Entity, &mut AnimatedGltf, &mut Visibility), Changed<AnimatedGltf>>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>
) { 
    for (animated_entity, mut animated, mut visibility) in animated_query.iter_mut() {
        let Some(animation_name) = animated.currently_playing.clone() else { continue; };
        if !animated.updated { continue; }
        *visibility = Visibility::Visible;
        for child in child_query.iter_descendants(animated_entity) {
            let Ok((mut player, mut transition)) = players.get_mut(child) else { continue; };
            transition.play(&mut player, animated.animations[&animation_name], Duration::ZERO);
        }
        animated.updated = false;
    }
}