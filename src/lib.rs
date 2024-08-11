use atoms::AtomPlugin;
use audio::SoundPlugin;
use bullet::{spawner::WeaponPlugin, NeutronBundle};
use camera::CameraPlugin;
use collision::CollisionPlugin;
use player::{PlayerBundle, PlayerPlugin};
use winny::{
    asset::server::AssetServer,
    gfx::{camera::Camera2dBundle, mesh2d::Mesh2d},
    math::vector::{Vec2f, Vec3f},
    prelude::*,
};

pub mod atoms;
pub mod audio;
pub mod bullet;
pub mod camera;
pub mod collision;
pub mod loader;
pub mod player;
pub mod types;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use types::*;

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
            CollisionPlugin,
            PlayerPlugin,
            WeaponPlugin,
            CameraPlugin,
            SoundPlugin,
            ChildrenPlugin,
            AtomPlugin,
        ))
        .add_systems(Schedule::StartUp, startup)
        .add_systems(Schedule::PostUpdate, apply_velocity)
        .run();
}

pub fn apply_velocity(mut q: Query<(Mut<Transform>, Velocity)>) {
    for (transform, vel) in q.iter_mut() {
        transform.translation += vel.0;
    }
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

    // commands.spawn((
    //     SpriteBundle {
    //         sprite: Sprite {
    //             scale: Vec2f::new(0.1, 0.1),
    //             ..Default::default()
    //         },
    //         material: Material2d::default(),
    //         handle: server.load("res/player.png"),
    //     },
    //     Transform::default(),
    // ));

    // commands.spawn((
    //     ParticleBundle {
    //         emitter: ParticleEmitter {
    //             acceleration: Vec3f::new(0., -100., 0.),
    //             width: 400.,
    //             height: 400.,
    //             particle_scale: Vec2f::new(0.2, 0.2),
    //             ..Default::default()
    //         },
    //         material: Material2d::default(),
    //         handle: server.load("winny/res/cube.png"),
    //     },
    //     Transform::default(),
    //     server.load::<Toml, _>("res/emitter.toml"),
    // ));

    commands.spawn(PlayerBundle::new(Vec3f::zero(), &server));

    commands.spawn((NeutronBundle::new_spawner(), Transform::default()));
    // commands.spawn(FireSkullBundle::new(
    //     &server,
    //     Transform::default(),
    //     Velocity(Vec3f::new(1.0, 0.0, 0.0)),
    // ));
}
