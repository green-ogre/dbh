use atoms::{Atom, AtomBundle, AtomPlugin};
use audio::{AudioMaster, SoundPlugin};
use bullet::NeutronBundle;
use bullet::{spawner::WeaponPlugin, RadialVelocity};
use camera::CameraPlugin;
use collision::CollisionPlugin;
use enemy::spawn_regular;
use player::{Crosshair, CrosshairOffset, PlayerBundle, PlayerPlugin};
use rand::Rng;
use regular::{RegularPolygons, RegularPolygonsPlugin};
use shaders::materials::PlayerMaterial;
use shaders::{ColorPalette, Paper8};
use shaders::{ShaderArtPlugin, SpaceHaze};
use std::f32::consts::TAU;
use std::io::Read;
use text::{TextPlugin, TypeWriter};
use winny::ecs::sets::IntoSystemStorage;
use winny::gfx::cgmath::{Quaternion, Rad, Rotation3};
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
        ))
        .egui_resource::<ThreatLevel>()
        .egui_component::<Parent>()
        .egui_component::<ChildOffset>()
        .insert_resource(ThreatLevel(1))
        .insert_resource(GameState::Menu)
        .add_plugins((
            winny::prelude::TextPlugin::new("res/fonts/SuperPixel-m2L8j.ttf"),
            ShaderArtPlugin,
            AtomPlugin,
            mouse::MousePlugin,
            ChildrenPlugin,
            enemy::EnemyPlugin,
            TextPlugin,
        ))
        // .insert_resource(TypeWriter::new(
        //     "Meltdown ...".into(),
        //     0.1,
        //     Vec2f::new(500., 500.),
        //     40.0,
        //     Modulation(Vec4f::new(1.0, 1.0, 1.0, 1.0)),
        // ))
        // .add_systems(Schedule::StartUp, startup)
        .add_systems(Schedule::StartUp, pre_menu_startup)
        .add_systems(Schedule::Update, menu.run_if(should_run_menu))
        .add_systems(
            Schedule::PreUpdate,
            update_threat_level.run_if(should_run_game),
        )
        .add_systems(
            Schedule::PostUpdate,
            (apply_velocity, apply_radial_velocity).run_if(should_run_game),
        )
        .run();
}

#[derive(Resource, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Game,
}

pub fn should_run_game(game_state: Res<GameState>) -> bool {
    *game_state == GameState::Game
}

pub fn should_run_menu(game_state: Res<GameState>) -> bool {
    *game_state == GameState::Menu
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

fn pre_menu_startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn menu(
    mut commands: Commands,
    mut text_renderer: Option<ResMut<TextRenderer>>,
    context: Res<RenderContext>,
    reader: EventReader<KeyInput>,
    mut game_state: ResMut<GameState>,
) {
    let Some(text_renderer) = &mut text_renderer else {
        return;
    };
    use winny::gfx::wgpu_text::glyph_brush::*;

    text_renderer.draw(&context, || {
        let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let meltdown = Section::default()
            .add_text(Text::new("MELTDOWN").with_scale(50.0).with_color(color))
            .with_bounds((
                context.config.width() as f32,
                context.config.height() as f32,
            ))
            .with_screen_position((context.config.width() as f32 / 2.0, 250.0))
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
            );

        let press_continue = Section::default()
            .add_text(
                Text::new("Press any key ...")
                    .with_scale(35.0)
                    .with_color(color),
            )
            .with_bounds((
                context.config.width() as f32,
                context.config.height() as f32,
            ))
            .with_screen_position((context.config.width() as f32 / 2.0, 800.0))
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
            );

        let controls = Section::default()
            .add_text(
                Text::new("Dash   -- Shift\nShoot  -- Space / Left Click")
                    .with_scale(35.0)
                    .with_color(color),
            )
            .with_bounds((
                context.config.width() as f32,
                context.config.height() as f32,
            ))
            .with_screen_position((180.0, 500.0))
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Left)
                    .v_align(VerticalAlign::Center),
            );

        vec![meltdown, controls, press_continue]
    });

    if reader.peak().is_some() {
        commands.run_system_once_when(startup, |_: Commands| true);
        *game_state = GameState::Game;
    }
}

fn startup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    mut assets: ResMut<Assets<Mesh2d>>,
    mut audio: ResMut<AudioMaster>,
    // mut audio: ResMut<GlobalAudio>,
    // type_writer: Res<TypeWriter>,
) {
    // type_writer.start(&mut commands);
    // audio.volume = 0.0;
    clear_color.0 = Modulation(SpaceHaze::dark_blue());
    // #[cfg(target_arch = "wasm32")]
    // server.set_prefix(
    //     wasm::ITCH_PREFIX
    //         .get()
    //         .unwrap_or_else(|| panic!("itch prefix was not set")),
    // );

    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        DirWatcherBundle {
            watcher: DirWatcher::new("res").unwrap(),
        },
        WatchForAsset,
    ));

    // commands.spawn((
    //     SpriteBundle {
    //         sprite: Sprite::default(),
    //         material: Material2d {
    //             texture: server.load("res/noise/noise.png"),
    //             ..Default::default()
    //         },
    //     },
    //     Transform::default(),
    // ));

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

    let cross_scale = Vec2f::new(0.2, 0.2);

    let make = |rotation: f32, offset: Vec3f| {
        (
            Transform {
                rotation: Quaternion::from_angle_z(Rad(rotation)),
                scale: cross_scale,
                ..Default::default()
            },
            Crosshair,
            server.load::<Mesh2d, _>("res/saved/player_mesh.msh"),
            PlayerMaterial {
                modulation: Modulation(SpaceHaze::white()),
            },
            CrosshairOffset(offset),
        )
    };

    let amt = 10.;
    commands.spawn(make(0., Vec3f::new(0., amt, 0.)));
    commands.spawn(make(TAU * 0.25, Vec3f::new(-amt, 00., 0.)));
    commands.spawn(make(TAU * 0.5, Vec3f::new(0., -amt, 0.)));
    commands.spawn(make(TAU * 0.75, Vec3f::new(amt, 00., 0.)));

    let polygons = RegularPolygons::new(40., &mut assets);
    let mut rng = rand::thread_rng();
    // for _ in 0..10 {
    //     let x = rng.gen_range(-500f32..500f32);
    //     let y = rng.gen_range(-500f32..500f32);

    //     let velocity = Vec3f::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5), 0.);

    //     AtomBundle::spawn(
    //         &mut commands,
    //         Vec3f::new(x, y, 0.),
    //         Some(velocity),
    //         None,
    //         &polygons,
    //         0,
    //         &server,
    //         &mut audio,
    //     );
    // }
    spawn_regular(
        Default::default(),
        &polygons,
        &mut commands,
        &server,
        &mut audio,
        4,
    );

    commands.insert_resource(polygons);
}
