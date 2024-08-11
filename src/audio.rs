use fxhash::FxHashMap;
use std::collections::VecDeque;
use winny::asset::server::AssetServer;
use winny::audio::AudioSource;
use winny::prelude::*;

#[derive(Debug)]
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(AudioMaster::default())
            .add_systems(Schedule::StartUp, load_all_sounds)
            .add_systems(Schedule::PreUpdate, update_sound_queue);
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
}

#[derive(Debug)]
pub struct AudioSample {
    pub handle: Handle<AudioSource>,
}

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AudioPath(pub &'static str);

fn load_all_sounds(mut _server: ResMut<AssetServer>, mut _audio_map: ResMut<AudioMaster>) {
    // let paths = [
    //     // "./res/lop/boss/demonlord",
    //     "./res/audio/RPG_Essentials_Free",
    // ];
    //
    // for path in paths.into_iter() {
    //     load_audio_directory(&mut server, path, &mut audio_map);
    // }
}
