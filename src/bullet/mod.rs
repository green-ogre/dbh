use std::f32::consts::PI;

use self::spawner::{BulletSpawner, Lifespan, Uptime};
use crate::{
    audio::AudioMaster,
    collision::{CircleCollider, CollideWithEnemy, Collider},
    CollisionDamage, Velocity,
};
use winny::{
    asset::server::AssetServer,
    gfx::mesh2d::Mesh2d,
    math::vector::{Vec2f, Vec3f},
    prelude::*,
};

pub mod spawner;

// #[derive(Bundle)]
// pub struct FireBallBundle {
//     transform: Transform,
//     velocity: Velocity,
//     collider: Collider,
//     collides: CollideWithEnemy,
//     damage: CollisionDamage,
//     remove: RemoveOnCollision,
//     lifespan: Lifespan,
//     uptime: Uptime,
//     asset_path: LopVfxAssetPath,
// }
//
// impl FireBallBundle {
//     pub fn new(position: Vec3f, velocity: Vec3f) -> (Self, AnimatedSpriteBundle) {
//         (
//             Self {
//                 transform: Transform {
//                     translation: position,
//                     ..Default::default()
//                 },
//                 velocity: Velocity(velocity),
//                 collider: Collider::Circle(CircleCollider {
//                     position: Vec3f::zero(),
//                     radius: 10f32,
//                 }),
//                 collides: CollideWithEnemy,
//                 damage: CollisionDamage(1f32),
//                 remove: RemoveOnCollision,
//                 lifespan: Lifespan(1.5f32),
//                 uptime: Uptime(0f32),
//                 asset_path: LopVfxAssetPath("flame"),
//             },
//             AnimatedSpriteBundle {
//                 material: Material2d::default(),
//                 animated_sprite: AnimatedSprite::default()
//                     .with_dimensions(Dimensions::new(1, 16))
//                     .with_frame_delta(0.1)
//                     .from_range(0..8),
//                 sprite: Sprite {
//                     position: Vec3f::new(0., 0., 0.),
//                     scale: Vec2f::new(1.5, 1.5),
//                     ..Default::default()
//                 },
//                 handle: Handle::dangling(),
//             },
//         )
//     }
//
//     pub fn spawn_audio_bundle(audio_master: &mut AudioMaster) {
//         let sound_path = AudioPath("04_Fire_explosion_04_medium.wav");
//         audio_master.queue_new_bundle(sound_path, PlaybackSettings::default().with_volume(4.0));
//     }
//
//     pub fn new_spawner() -> BulletSpawner {
//         BulletSpawner::new(0.5, |pos, enemies, commands, server, audio_master| {
//             // match enemies.len() {
//             //     0 => {}
//             //     1..=2 => {
//             //         let direction = enemies[0] - pos.translation;
//             //
//             //         commands.spawn(FireBallBundle::new(pos.translation, direction.normalize()));
//             //         FireBallBundle::spawn_audio_bundle(audio_master);
//             //     }
//             //     3..=9 => {
//             //         let mut rng = rand::thread_rng();
//             //         let index = rng.gen_range(0..3);
//             //         let direction = enemies[index] - pos.translation;
//             //
//             //         commands.spawn(FireBallBundle::new(pos.translation, direction.normalize()));
//             //         FireBallBundle::spawn_audio_bundle(audio_master);
//             //     }
//             //     len @ 10.. => {
//             //         let top_30 = len as f32 * 0.3;
//             //         let top_30 = top_30 as usize;
//             //
//             //         let mut rng = rand::thread_rng();
//             //         let index = rng.gen_range(0..top_30);
//             //         let direction = enemies[index] - pos.translation;
//             //
//             //         commands.spawn(FireBallBundle::new(pos.translation, direction.normalize()));
//             //         FireBallBundle::spawn_audio_bundle(audio_master);
//             //     }
//             // }
//
//             // if let Some(nearest) = enemies.first() {
//             //     let direction = *nearest - pos.translation;
//             //
//             //     commands.spawn(FireBallBundle::new(pos.translation, direction.normalize()));
//             //     FireBallBundle::spawn_audio_bundle(audio_master);
//             // } else {
//             //     // don't spawn any if there's no enemies
//             // }
//
//             if !enemies.is_empty() {
//                 let enemy = if let Some(nearest) = to_nearest(&pos.translation, enemies.iter()) {
//                     nearest
//                 } else {
//                     enemies[0] - pos.translation
//                 };
//
//                 let fire_ball = commands
//                     .spawn(FireBallBundle::new(pos.translation, enemy.normalize()))
//                     .entity();
//
//                 const INITIAL_SPEED: f32 = 200.;
//                 let mut initial_velocity = -enemy.normalize() * INITIAL_SPEED;
//                 initial_velocity.y *= -1.0;
//                 commands.spawn((
//                     ParticleBundle {
//                         emitter: ParticleEmitter {
//                             num_particles: 5,
//                             lifetime: 0.2..0.5,
//                             width: 10.,
//                             height: 10.,
//                             // particle_scale: Vec2f,
//                             // particle_rotation: Radf,
//                             initial_velocity,
//                             // acceleration: Vec3f::new(100., 100., 0.),
//                             ..Default::default()
//                         },
//                         material: Material2d::default(),
//                         handle: server.load("textures/particles/ember.png"),
//                     },
//                     Transform {
//                         translation: pos.translation,
//                         ..Default::default()
//                     },
//                     ChildOf(fire_ball),
//                 ));
//
//                 FireBallBundle::spawn_audio_bundle(audio_master);
//             }
//         })
//     }
// }

#[derive(Bundle)]
pub struct FireSkullBundle {
    transform: Transform,
    velocity: Velocity,
    collider: Collider,
    collides: CollideWithEnemy,
    damage: CollisionDamage,
    lifespan: Lifespan,
    uptime: Uptime,
    mesh: Handle<Mesh2d>,
}

impl FireSkullBundle {
    pub fn new(server: &AssetServer, mut transform: Transform, velocity: Velocity) -> Self {
        transform.scale = Vec2f::new(0.1, 0.1);
        Self {
            transform,
            velocity,
            collider: Collider::Circle(CircleCollider {
                position: Vec3f::new(0., -50., 0.),
                radius: 30f32,
            }),
            collides: CollideWithEnemy,
            damage: CollisionDamage(2f32),
            lifespan: Lifespan(6f32),
            uptime: Uptime(2f32),
            mesh: server.load("res/saved/bullet_1_mesh.msh"),
        }
    }

    pub fn spawn_audio_bundle(_audio_master: &mut AudioMaster) {
        // let sound_path = AudioPath("04_Fire_explosion_04_medium.wav");
        // audio_master.queue_new_bundle(sound_path, PlaybackSettings::default().with_volume(4.0));
    }

    pub fn new_spawner() -> BulletSpawner {
        BulletSpawner::new(1.5, move |transform, commands, server, audio_master| {
            const SPEED: f32 = 1.5;
            for i in 0..4 {
                let direction = i as f32 * 0.5 * PI + 0.25 * PI;
                let x = direction.cos() * SPEED;
                let y = direction.sin() * SPEED;
                let velocity = Velocity(Vec3f::new(x, y, 0.));
                commands.spawn(Self::new(server, transform.clone(), velocity));
            }
            Self::spawn_audio_bundle(audio_master);
        })
    }
}
