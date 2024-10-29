use bevy::{prelude::*, utils::hashbrown::HashMap};
use std::time::Duration;

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
    mut animated_query: Query<(Entity, &mut AnimatedGltf)>,
    assets_gltf: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>
) {
    for (animated_entity, mut animated) in animated_query.iter_mut() {
        if animated.loaded { continue; };
        if !asset_server.is_loaded_with_dependencies(animated.gltf.id()) { continue; };
        animated.loaded = true;
        let Some(gltf) = assets_gltf.get(animated.gltf.id()) else { continue; };
        commands.entity(animated_entity).with_children(|p| {
            p.spawn(SceneBundle {
                scene: gltf.scenes[0].clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            });
        });
        let mut graph = AnimationGraph::new();
        for handle in &gltf.animations {
            for (name, clip) in gltf.named_animations.clone() {
                if clip.id() == handle.id() {
                    animated.animations.insert(name.to_string(),graph.add_clip(clip, 1.0, graph.root));
                }
            }
        }
        animated.graph = graphs.add(graph).clone();
    }
}

pub fn handle_loaded_anims(
    mut commands: Commands,
    animated_query: Query<(Entity, &AnimatedGltf)>,
    players: Query<Entity, Added<AnimationPlayer>>,
    children_query: Query<&Children>,
) {
    for (animated_entity, animated) in animated_query.iter() {
        for child in children_query.iter_descendants(animated_entity) {
            if let Ok(player) = players.get(child) {
                commands.entity(player).insert(animated.graph.clone()).insert(AnimationTransitions::new());
            }
        }
    }
}

fn handle_new_anim(
    child_query: Query<&Children>,
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