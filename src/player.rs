// use crate::assets::Background;
// use crate::collision::{CircleCollider, Collider, PlayerCollideEvent};
// use crate::enemy::state_machine::{velocity_to_lop, EnemyStateMachine, LopDirection};
// use crate::enemy::EnemyType;
// use crate::sounds::{AudioMaster, AudioPath};
// use crate::weapons::aoe::AoeAttack;
// use crate::weapons::bullet::FireSkullBundle;
// use crate::{game_is_running, ChildOf, Health, Velocity};
// use winny::app::render::RenderContext;
// use winny::ecs::EntityCommands;
// use winny::gfx::render_pipeline::material::Material2d;
// use winny::gfx::sprite::{AnimatedSprite, AnimatedSpriteBundle, SpriteBundle};
// use winny::{
//     gfx::{
//         cgmath::{Quaternion, Zero},
//         sprite::Sprite,
//         text::TextRenderer,
//         transform::Transform,
//     },
//     prelude::*,
// };

use winny::{
    asset::server::AssetServer,
    gfx::cgmath::{Quaternion, Zero},
    math::vector::{Vec2f, Vec3f},
    prelude::*,
};

use crate::{
    collision::{CircleCollider, Collider},
    Health, Velocity,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(PressedState::default())
            .egui_component::<Dash>()
            // .add_systems(
            //     Schedule::PostUpdate,
            //     // level_up_sfx,
            //     // handle_player_level_up,
            //     // update_player_ui,
            // )
            .add_systems(Schedule::Update, (update_keystate, update_player));
    }
}

/// Player marker struct.
#[derive(Debug, Clone, Copy, Component)]
pub struct Player;

/// Keeps track of velocity in every direction
/// from the input stream. This makes certain
/// operations easier.
#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct DirectionalVelocity {
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
}

impl From<DirectionalVelocity> for Vec3f {
    fn from(value: DirectionalVelocity) -> Self {
        Vec3f::new(-value.left + value.right, -value.up + value.down, 0f32)
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LastKnownVelocity(pub Vec3f);

#[derive(Component, Debug)]
pub struct PlayerExp(pub u32);

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerLevel(pub u32);

impl PlayerLevel {
    pub fn level_up_exp(&self) -> u32 {
        self.0 * 5
    }
}

#[derive(AsEgui, Component, Debug)]
pub struct Dash {
    strength: f32,
    duration: f32,
    remaining: f32,
    cooldown_duration: f32,
    cooldown: f32,
}

impl Default for Dash {
    fn default() -> Self {
        Self {
            strength: 20.0,
            duration: 0.1,
            remaining: 0.0,
            cooldown_duration: 0.5,
            cooldown: 0.0,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    transform: Transform,
    velocity: Velocity,
    collider: Collider,
    player: Player,
    directional_velocity: DirectionalVelocity,
    health: Health,
    sprite: SpriteBundle,
    last_known_vel: LastKnownVelocity,
    dash: Dash,
}

impl PlayerBundle {
    pub fn new(position: Vec3f, server: &AssetServer) -> Self {
        Self {
            transform: Transform {
                rotation: Quaternion::zero(),
                scale: Vec2f::one(),
                translation: position,
            },
            velocity: Default::default(),
            last_known_vel: Default::default(),
            // collider: Collider::Rect(RectCollider {
            //     tl: Vec3f::new(0., 0., 0.),
            //     size: Vec3f::new(20., 25., 0.),
            // }),
            directional_velocity: DirectionalVelocity::default(),
            collider: PlayerBundle::collider(),
            player: Player,
            health: Health::new(100., 0.),
            sprite: SpriteBundle {
                sprite: Sprite {
                    scale: Vec2f::new(0.1, 0.1),
                    ..Default::default()
                },
                material: Material2d::default(),
                handle: server.load("res/player.png"),
            },
            dash: Dash::default(),
        }
    }

    fn collider() -> Collider {
        Collider::Circle(CircleCollider {
            position: Vec3f::new(0., 0., 0.),
            radius: 25.,
        })
    }
}

// #[derive(Component)]
// pub struct PlayerHealthBar;
//
// #[derive(Component)]
// pub struct PlayerXpBar;
//
// const UI_SCALE: f32 = 1.;

// fn spawn_ui(position: Vec3f, commands: &mut Commands, server: &mut AssetServer) {
//     let path = "textures/health.png";
//
//     let health_transform = Transform {
//         translation: position,
//         ..Default::default()
//     };
//
//     commands.spawn((
//         Background,
//         SpriteBundle {
//             handle: server.load(path),
//             sprite: Sprite {
//                 position: Vec3f::new(0., 0., 0.),
//                 scale: Vec2f::new(UI_SCALE, UI_SCALE),
//                 z: 3000,
//                 ..Default::default()
//             },
//             material: Material2d::default(),
//         },
//         health_transform,
//     ));
//
//     commands.spawn((
//         Background,
//         PlayerHealthBar,
//         SpriteBundle {
//             handle: server.load(path),
//             sprite: Sprite {
//                 position: Vec3f::new(0., 0., 0.),
//                 scale: Vec2f::new(UI_SCALE, UI_SCALE),
//                 z: 3001,
//                 ..Default::default()
//             },
//             material: Material2d::default(),
//         },
//         health_transform,
//     ));
//
//     let xp_transform = Transform {
//         translation: Vec3f {
//             x: position.x,
//             y: position.y + 30.,
//             z: 0f32,
//         },
//         ..Default::default()
//     };
//
//     commands.spawn((
//         Background,
//         SpriteBundle {
//             handle: server.load(path),
//             sprite: Sprite {
//                 position: Vec3f::new(0., 0., 0.),
//                 scale: Vec2f::new(UI_SCALE, UI_SCALE),
//                 z: 3000,
//                 ..Default::default()
//             },
//             material: Material2d::default(),
//         },
//         xp_transform,
//     ));
//
//     commands.spawn((
//         Background,
//         PlayerXpBar,
//         SpriteBundle {
//             handle: server.load(path),
//             sprite: Sprite {
//                 position: Vec3f::new(0., 0., 0.),
//                 scale: Vec2f::new(UI_SCALE, UI_SCALE),
//                 z: 3001,
//                 ..Default::default()
//             },
//             material: Material2d::default(),
//         },
//         xp_transform,
//     ));
// }

// #[derive(Debug, Event)]
// pub struct LevelUpEvent(pub PlayerLevel);

const PLAYER_SPEED: f32 = 10.0;

pub fn update_player(
    mut commands: Commands,
    mut q: Query<
        (
            Entity,
            Mut<Velocity>,
            Mut<DirectionalVelocity>,
            Mut<LastKnownVelocity>,
            Mut<Dash>,
        ),
        With<Player>,
    >,
    state: Res<PressedState>,
    delta: Res<DeltaTime>,
    // collision: EventReader<PlayerCollideEvent>,
    // mut level_writer: EventWriter<LevelUpEvent>,
) {
    let Some((player, Velocity(velocity), dir_vel, last_known_vel, dash)) = q.iter_mut().next()
    else {
        return;
    };

    if dash.remaining > 0.0 {
        *velocity = last_known_vel.0.normalize() * dash.strength;
        dash.remaining -= delta.delta;

        // finished dash
        if dash.remaining <= 0.0 {
            commands.get_entity(player).insert(PlayerBundle::collider());
        }
    } else {
        let speed = 0.25f32;

        dir_vel.up = state.up.then_some(speed).unwrap_or_default();
        dir_vel.down = state.down.then_some(speed).unwrap_or_default();
        dir_vel.left = state.left.then_some(speed).unwrap_or_default();
        dir_vel.right = state.right.then_some(speed).unwrap_or_default();

        let mut norm: Vec3f = (*dir_vel).into();

        if !norm.is_zero() {
            norm = norm.normalize() * PLAYER_SPEED;
            *last_known_vel = LastKnownVelocity(norm);
        }
        *velocity = norm;
    }

    dash.cooldown = (dash.cooldown - delta.delta).clamp(0.0, 100.0);

    // while exp.0 >= level.level_up_exp() {
    //     exp.0 -= level.level_up_exp();
    //     level.0 += 1;
    //     level_writer.send(LevelUpEvent(*level));
    // }

    // for event in collision.peak_read() {
    //     info!("Collision event: {event:?}");
    // }
}

pub fn update_keystate(
    mut commands: Commands,
    input: EventReader<KeyInput>,
    mut state: ResMut<PressedState>,
    mut player: Query<(Entity, Mut<Dash>), With<Player>>,
) {
    for event in input.peak_read() {
        let ks = matches!(event.state, KeyState::Pressed);

        match event.code {
            KeyCode::W => state.up = ks,
            KeyCode::S => state.down = ks,
            KeyCode::A => state.left = ks,
            KeyCode::D => state.right = ks,
            _ => {}
        }

        if matches!(
            event,
            KeyInput {
                code: KeyCode::Space,
                state: KeyState::Pressed,
                ..
            }
        ) {
            let Ok((player, dash)) = player.get_single_mut() else {
                return;
            };

            if dash.cooldown <= 0.0 {
                dash.remaining = dash.duration;
                dash.cooldown = dash.cooldown_duration;
                commands.get_entity(player).remove::<Collider>();
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Resource)]
pub struct PressedState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

// pub fn update_player_ui(
//     player: Query<(Entity, Transform, Health, PlayerExp, PlayerLevel), With<Player>>,
//     mut health: Query<Mut<Sprite>, With<PlayerHealthBar>>,
//     mut xp: Query<Mut<Sprite>, With<PlayerXpBar>>,
//     mut text_renderer: Option<ResMut<TextRenderer>>,
//     input: EventReader<KeyInput>,
//     context: Res<RenderContext>,
//     mut commands: Commands,
//     mut audio_master: ResMut<AudioMaster>,
// ) {
//     let Some((player, transform, player_health, PlayerExp(player_xp), level)) =
//         player.iter().next()
//     else {
//         return;
//     };
//
//     for event in input.peak_read() {
//         match event {
//             KeyInput {
//                 code: KeyCode::J,
//                 state: KeyState::Pressed,
//             } => {
//                 crate::weapons::aoe::AoeAttack::NecroCircle.spawn_hit_enemy(
//                     player,
//                     transform.translation,
//                     &mut commands,
//                     &mut audio_master,
//                 );
//             }
//             KeyInput {
//                 code: KeyCode::K,
//                 state: KeyState::Pressed,
//             } => {
//                 crate::weapons::aoe::AoeAttack::NecroSpin.spawn_hit_enemy(
//                     player,
//                     transform.translation,
//                     &mut commands,
//                     &mut audio_master,
//                 );
//             }
//             KeyInput {
//                 code: KeyCode::L,
//                 state: KeyState::Pressed,
//             } => {
//                 crate::weapons::aoe::AoeAttack::BloodBurst.spawn_hit_enemy(
//                     player,
//                     transform.translation,
//                     &mut commands,
//                     &mut audio_master,
//                 );
//             }
//             _ => {}
//         }
//     }
//
//     for bar in health.iter_mut() {
//         let ratio = player_health.ratio();
//         bar.scale.v[0] = ratio;
//         bar.position.x = -(256. * (1. - ratio)) * 0.5 * UI_SCALE;
//     }
//
//     for bar in xp.iter_mut() {
//         let ratio = *player_xp as f32 / level.level_up_exp() as f32;
//         bar.scale.v[0] = ratio;
//         bar.position.x = -(256. * (1. - ratio)) * 0.5 * UI_SCALE;
//     }
//
//     let Some(text_renderer) = &mut text_renderer else {
//         return;
//     };
//
//     use winny::gfx::wgpu_text::glyph_brush::*;
//
//     let level_text = format!("{}", level.0);
//     text_renderer.draw(&context.device, &context.queue, || {
//         let xp = Section::default()
//             .add_text(
//                 Text::new(&level_text)
//                     .with_scale(20.0)
//                     .with_color([1., 1., 1., 1.]),
//             )
//             .with_bounds((
//                 context.config.width() as f32,
//                 context.config.height() as f32,
//             ))
//             .with_screen_position((150., 110.))
//             .with_layout(
//                 Layout::default()
//                     .h_align(HorizontalAlign::Center)
//                     .v_align(VerticalAlign::Center),
//             );
//
//         vec![xp]
//     });
// }

// fn level_up_sfx(mut master: ResMut<AudioMaster>, lvl_up: EventReader<LevelUpEvent>) {
//     if lvl_up.peak().is_some() {
//         let sound_path = "013_Confirm_03.wav";
//         let bundle = AudioBundle {
//             handle: master.get_handle_or_dangle(&AudioPath(sound_path)),
//             playback_settings: PlaybackSettings {
//                 volume: 5.0,
//                 ..Default::default()
//             },
//         };
//
//         master.queue_bundle(bundle);
//     }
// }
//
// fn handle_player_level_up(
//     mut commands: Commands,
//     player: Query<Entity, With<Player>>,
//     lvl_up: EventReader<LevelUpEvent>,
// ) {
//     let Ok(player) = player.get_single() else {
//         return;
//     };
//
//     for lvl_up in lvl_up.read() {
//         let level = lvl_up.0;
//         match level.0 {
//             1 => {
//                 // commands.spawn((
//                 //     FireBallBundle::new_spawner(),
//                 //     Transform::default(),
//                 //     ChildOf(player),
//                 // ));
//             }
//             2 => {
//                 commands.spawn((
//                     FireSkullBundle::new_spawner(),
//                     Transform::default(),
//                     ChildOf(player),
//                 ));
//             }
//             3 => {
//                 commands.spawn((
//                     AoeAttack::NecroCircle.new_spawner(1.5),
//                     Transform::default(),
//                     ChildOf(player),
//                 ));
//             }
//             4 => {
//                 commands.spawn((
//                     AoeAttack::NecroSpin.new_spawner(1.),
//                     Transform::default(),
//                     ChildOf(player),
//                 ));
//             }
//             _ => {}
//         }
//     }
// }
