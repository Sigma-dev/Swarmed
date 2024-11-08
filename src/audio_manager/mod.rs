use bevy::{audio::{PlaybackMode, Volume}, prelude::*, utils::HashMap};

#[derive(Event)]
pub struct PlayAudio3D {
    pub path: String,
    pub position: Vec3,
    pub volume_mut: f32,
    pub one_shot: bool,
    pub follow: Option<Entity>,
}

#[derive(Component)]
pub struct AudioFollower {
    pub followed: Entity,
}

#[derive(Resource, Default)]
pub struct AudioManager {
    audio_handles: HashMap<String, Handle<AudioSource>>
}

impl AudioManager {
    pub fn get_audio(&mut self, asset_server: &Res<AssetServer>, path: &String) -> Handle<AudioSource> {
        if let Some(audio) = self.audio_handles.get(path) {
            return audio.clone()
        }
        let handle = asset_server.load(path.clone());
        self.audio_handles.insert(path.clone(), handle.clone());
        handle
    }
}

pub struct AudioManagerPlugin;

impl Plugin for AudioManagerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (handle_sound_events, handle_followers))
        .add_event::<PlayAudio3D>()
        .insert_resource(AudioManager::default());
    }
}

fn handle_sound_events(
    mut audio_manager: ResMut<AudioManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sound_reader: EventReader<PlayAudio3D>
) {
    for sound in sound_reader.read() {
        let settings = PlaybackSettings {
            mode: if sound.one_shot { PlaybackMode::Despawn } else { PlaybackMode::Loop },
            volume: Volume::new(1. * sound.volume_mut),
            spatial: true,
            ..default()
        };
        let source = audio_manager.get_audio(&asset_server, &sound.path);
        let mut e = commands.spawn((
            AudioBundle {
                source,
                settings: settings,
            },
            SpatialBundle {
                transform: Transform::from_translation(sound.position),
                ..default()
            },
        ));
        if let Some(followed) = sound.follow {
            e.insert(AudioFollower { followed });
        }
    }
}

fn handle_followers(
    mut followers_query: Query<(&mut Transform, &AudioFollower)>,
    transforms_query: Query<& Transform, Without<AudioFollower>>
) {
    for (mut follower_transform, follower) in followers_query.iter_mut() {
        follower_transform.translation = transforms_query.get(follower.followed).unwrap().translation;
    }
}