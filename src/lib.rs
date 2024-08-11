use atoms::{Atom, AtomBundle, AtomPlugin};
use audio::SoundPlugin;
use bullet::NeutronBundle;
use bullet::{spawner::WeaponPlugin, RadialVelocity};
use camera::CameraPlugin;
use collision::CollisionPlugin;
use player::{PlayerBundle, PlayerPlugin};
use rand::Rng;
use regular::RegularPolygonsPlugin;
use shaders::{ColorPalette, Paper8};
use shaders::{ShaderArtPlugin, SpaceHaze};
use std::io::Read;
use text::{TextPlugin, TypeWriter};
use winny::gfx::mesh2d::{Mesh2d, Points};
use winny::math::vector::Vec4f;
use winny::{
    asset::server::AssetServer,
    gfx::camera::Camera2dBundle,
    math::vector::{Vec2f, Vec3f},
    prelude::*,
};

pub mod atoms;
pub mod audio;
pub mod bullet;
pub mod camera;
pub mod collision;
pub mod enemy;
pub mod loader;
pub mod mouse;
pub mod player;
pub mod regular;
pub mod shaders;
pub mod text;
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
                    window_size: Vec2f::new(512.0 * 2.0, 512.0 * 2.0),
                    viewport_size: Vec2f::new(512.0 * 2.0, 512.0 * 2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            RegularPolygonsPlugin,
            TomlPlugin,
            WatcherPlugin,
            CollisionPlugin,
            PlayerPlugin,
            WeaponPlugin,
            CameraPlugin,
            SoundPlugin,
            ShaderArtPlugin,
        ))
        .egui_resource::<ThreatLevel>()
        .insert_resource(ThreatLevel(1))
        .add_plugins((
            AtomPlugin,
            mouse::MousePlugin,
            ChildrenPlugin,
            enemy::EnemyPlugin,
            TextPlugin,
        ))
        .insert_resource(TypeWriter::new(
            "Meltdown ...".into(),
            0.1,
            Vec2f::new(500., 500.),
            40.0,
            Modulation(Vec4f::new(1.0, 1.0, 1.0, 1.0)),
        ))
        .add_systems(Schedule::StartUp, startup)
        .add_systems(Schedule::PreUpdate, update_threat_level)
        .add_systems(
            Schedule::PostUpdate,
            (apply_velocity, apply_radial_velocity),
        )
        .run();
}

pub fn apply_velocity(mut q: Query<(Mut<Transform>, Velocity)>, dt: Res<DeltaTime>) {
    for (transform, vel) in q.iter_mut() {
        transform.translation += vel.0 * 120. * dt.delta;
    }
}

pub fn apply_radial_velocity(
    mut q: Query<(Mut<Transform>, Mut<RadialVelocity>)>,
    dt: Res<DeltaTime>,
) {
    for (transform, vel) in q.iter_mut() {
        vel.update(transform, &dt)
    }
}

#[derive(Resource, AsEgui)]
pub struct ThreatLevel(pub u32);

impl Default for ThreatLevel {
    fn default() -> Self {
        Self(1)
    }
}

fn update_threat_level(atoms: Query<Atom>, mut threat: ResMut<ThreatLevel>) {
    const ENEMEIES_PER_THREAT_LEVEL: u32 = 20;

    let count = atoms.iter().count();
    threat.0 = count as u32 / ENEMEIES_PER_THREAT_LEVEL;
}

fn startup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    mut audio: ResMut<GlobalAudio>,
    type_writer: Res<TypeWriter>,
) {
    type_writer.start(&mut commands);
    audio.volume = 0.0;
    clear_color.0 = Modulation(SpaceHaze::dark_blue());
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
        SpriteBundle {
            sprite: Sprite::default(),
            material: Material2d {
                texture: server.load("res/noise/noise.png"),
                ..Default::default()
            },
        },
        Transform::default(),
    ));

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

    // commands.spawn((NeutronBundle::new_spawner(), Transform::default()));
    // commands.spawn(FireSkullBundle::new(
    //     &server,
    //     Transform::default(),
    //     Velocity(Vec3f::new(1.0, 0.0, 0.0)),
    // ));
}
