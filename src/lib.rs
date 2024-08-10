use winny::{
    asset::server::AssetServer,
    gfx::camera::Camera2dBundle,
    math::vector::{Vec2f, Vec3f},
    prelude::*,
};

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub fn run() {
    App::default()
        .add_plugins((
            DefaultPlugins {
                window: WindowPlugin {
                    title: "dbh",
                    close_on_escape: true,
                    #[cfg(target_arch = "wasm32")]
                    window_size: Vec2f::new(1280., 720.),
                    #[cfg(not(target_arch = "wasm32"))]
                    window_size: Vec2f::new(1920., 1080.),
                    ..Default::default()
                },
                ..Default::default()
            },
            TomlPlugin,
            WatcherPlugin,
        ))
        .add_systems(Schedule::StartUp, startup)
        .run();
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    #[cfg(target_arch = "wasm32")]
    server.set_prefix(
        wasm::ITCH_PREFIX
            .get()
            .unwrap_or_else(|| panic!("itch prefix was not set")),
    );

    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        DirWatcherBundle {
            watcher: DirWatcher::new("res").unwrap(),
        },
        WatchForAsset,
    ));

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        ParticleBundle {
            emitter: ParticleEmitter {
                acceleration: Vec3f::new(0., -100., 0.),
                width: 400.,
                height: 400.,
                particle_scale: Vec2f::new(0.2, 0.2),
                ..Default::default()
            },
            material: Material2d::default(),
            handle: server.load("winny/res/cube.png"),
        },
        Transform::default(),
        server.load::<Toml, _>("res/emitter.toml"),
    ));
}
