use fxhash::FxHashMap;
use std::collections::VecDeque;
use winny::asset::server::AssetServer;
use winny::audio::AudioSource;
use winny::ecs::sets::IntoSystemStorage;
use winny::prelude::*;

use crate::{should_run_game, ThreatLevel};

#[derive(Debug)]
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<Music>()
            .insert_resource(AudioMaster::default())
            .add_systems(Schedule::StartUp, load_all_sounds)
            .add_systems(Schedule::PostUpdate, update_sound_queue)
            .add_systems(Schedule::PostUpdate, update_music.run_if(should_run_game));
    }
}

#[derive(Resource, Default)]
pub struct AudioMaster {
    sample_map: FxHashMap<AudioPath, AudioSample>,
    queue: VecDeque<AudioBundle>,
}

impl AudioMaster {
    pub fn insert_sample(&mut self, path: AudioPath, sample: AudioSample) {
        self.sample_map.insert(path, sample);
    }

    pub fn get_handle_or_dangle(&self, path: &AudioPath) -> Handle<AudioSource> {
        self.sample_map
            .get(path)
            .map(|s| s.handle.clone())
            .unwrap_or_else(|| {
                warn!("returning dangling handle: {:?}", path);
                Handle::dangling()
            })
    }

    pub fn queue_new_bundle(&mut self, path: AudioPath, playback_settings: PlaybackSettings) {
        let bundle = AudioBundle {
            handle: self.get_handle_or_dangle(&path),
            playback_settings,
        };
        self.queue_bundle(bundle);
    }

    pub fn queue_bundle(&mut self, bundle: AudioBundle) {
        self.queue.push_back(bundle);
    }
}

#[derive(Component)]
struct QueuedSound;

fn update_sound_queue(mut commands: Commands, mut master: ResMut<AudioMaster>) {
    if let Some(bundle) = master.queue.pop_front() {
        commands.spawn((bundle, QueuedSound));
    }

    master.queue.drain(..);
}

#[derive(Debug)]
pub struct AudioSample {
    pub handle: Handle<AudioSource>,
}

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AudioPath(pub &'static str);

#[derive(Resource)]
pub struct Music {
    pub track_1: Track,
    pub track_2: Track,
    pub track_3: Track,
    pub track_4: Track,
}

pub struct Track {
    pub handle: Handle<AudioSource>,
    pub entity: Option<Entity>,
}

impl Track {
    pub fn new(handle: Handle<AudioSource>) -> Self {
        Self {
            handle,
            entity: None,
        }
    }
}

fn load_all_sounds(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Music {
        track_1: Track::new(server.load("res/nuclear jam/jam1.wav")),
        track_2: Track::new(server.load("res/nuclear jam/jam2.wav")),
        track_3: Track::new(server.load("res/nuclear jam/jam3.wav")),
        track_4: Track::new(server.load("res/nuclear jam/jam4.wav")),
    });
}

fn update_music(
    mut commands: Commands,
    mut music: ResMut<Music>,
    reader: EventReader<ExitingStream>,
    threat: Res<ThreatLevel>,
) {
    for entity in reader.read() {
        if Some(entity.0) == music.track_1.entity {
            music.track_1.entity = None;
        } else if Some(entity.0) == music.track_2.entity {
            music.track_2.entity = None;
        } else if Some(entity.0) == music.track_3.entity {
            music.track_3.entity = None;
        } else if Some(entity.0) == music.track_4.entity {
            music.track_4.entity = None;
        }
    }

    if music.track_1.entity.is_none() {
        info!("playing stem 1");
        music.track_1.entity = Some(
            commands
                .spawn(AudioBundle {
                    handle: music.track_1.handle.clone(),
                    playback_settings: PlaybackSettings::default().loop_track().with_volume(250.0),
                })
                .entity(),
        );

        if music.track_2.entity.is_none() && threat.0 >= 2 {
            info!("playing stem 2");
            music.track_2.entity = Some(
                commands
                    .spawn(AudioBundle {
                        handle: music.track_2.handle.clone(),
                        playback_settings: PlaybackSettings::default()
                            .loop_track()
                            .with_volume(250.0),
                    })
                    .entity(),
            );
        }

        if music.track_3.entity.is_none() && threat.0 >= 3 {
            info!("playing stem 3");
            music.track_3.entity = Some(
                commands
                    .spawn(AudioBundle {
                        handle: music.track_3.handle.clone(),
                        playback_settings: PlaybackSettings::default()
                            .loop_track()
                            .with_volume(250.0),
                    })
                    .entity(),
            );
        }

        if music.track_4.entity.is_none() && threat.0 >= 4 {
            info!("playing stem 4");
            music.track_4.entity = Some(
                commands
                    .spawn(AudioBundle {
                        handle: music.track_4.handle.clone(),
                        playback_settings: PlaybackSettings::default()
                            .loop_track()
                            .with_volume(250.0),
                    })
                    .entity(),
            );
        }
    }
}
