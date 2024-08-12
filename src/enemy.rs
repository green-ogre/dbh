use std::f32::consts::{PI, TAU};

use angle::Radf;
use camera::Camera;
use rand::{Rng, SeedableRng};
use server::AssetServer;
use sets::IntoSystemStorage;
use vector::{Vec2f, Vec3f};
use winny::prelude::*;

use crate::{
    atoms::AtomBundle,
    audio::AudioMaster,
    bullet::{NeutronBundle, RadialVelocity},
    collision::{CircleCollider, CollideWithPlayer, Collider, EnemyCollideEvent},
    player::Player,
    regular::RegularPolygons,
    shaders::{materials::HeptaMaterial, Crimson},
    should_run_game, ChildOffset, CollisionDamage, Enemy, Parent, Velocity,
};

#[derive(Debug)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(EnemySpawner::new()).add_systems(
            Schedule::PostUpdate,
            (update_heading_towards_player, update_regular, spawn_enemies).run_if(should_run_game),
        );
    }
}

/// Facilitates "steering" behavior, giving enemies a feeling of momentum.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct Heading {
    pub direction: Radf,
    pub speed: f32,
}

impl Heading {
    pub fn new(speed: f32) -> Self {
        Self {
            direction: Default::default(),
            speed,
        }
    }
}

/// The turning speed for headings.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component, AsEgui)]
pub struct TurnSpeed(pub f32);

/// The turning speed for headings.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component, AsEgui)]
pub struct SpinSpeed(pub f32);

impl Heading {
    pub fn steer_towards(&mut self, turn_speed: f32, from: &Vec3f, to: &Vec3f) {
        use std::f32::consts::PI;

        // Calculate the desired direction
        let desired_direction = (*to - *from).normalize();

        // Convert the desired direction to an angle
        let desired_angle = desired_direction.y.atan2(desired_direction.x);

        // Calculate the smallest angle difference
        let mut angle_diff = (desired_angle - self.direction.0) % TAU;
        if angle_diff > PI {
            angle_diff = PI - angle_diff;
        } else if angle_diff < -PI {
            angle_diff = -PI - angle_diff;
        };

        // Gradually adjust the current direction
        self.direction.0 += angle_diff * turn_speed;

        // Normalize
        self.direction.0 %= TAU;
    }
}

pub fn update_heading_towards_player(
    mut q: Query<(Mut<Velocity>, Mut<Heading>, Transform, TurnSpeed)>,
    player: Query<Transform, With<Player>>,
    time: Res<DeltaTime>,
) {
    let Some(player) = player.iter().next() else {
        return;
    };

    for (Velocity(velocity), heading, transform, TurnSpeed(turn_speed)) in q.iter_mut() {
        heading.steer_towards(
            *turn_speed * time.delta,
            &transform.translation,
            &player.translation,
        );
        velocity.x = heading.speed * heading.direction.0.cos();
        velocity.y = heading.speed * heading.direction.0.sin();
    }
}

#[derive(Debug, Component, AsEgui)]
pub struct RegularEnemy(pub f32);

#[derive(Debug, Component, AsEgui)]
struct EnemyCloud(pub Vec<Entity>);

const REGULAR_RADIUS: f32 = 50.;

pub fn spawn_regular(
    position: Vec3f,
    polygons: &RegularPolygons,
    commands: &mut Commands,
    server: &AssetServer,
    audio: &mut AudioMaster,
    children: usize,
) {
    let mut enemy_cloud = Vec::new();
    let mut rng = rand::rngs::SmallRng::from_entropy();

    let parent = commands
        .spawn((
            Enemy,
            RegularEnemy(0.),
            Transform {
                translation: position,
                scale: Vec2f::new(0.5, 0.5),
                ..Default::default()
            },
            Velocity(Default::default()),
            Heading::new(rng.gen_range(1.5f32..4f32)),
            TurnSpeed(rng.gen_range(1f32..3f32)),
            CollisionDamage(1.),
            SpinSpeed(rng.gen_range(-2f32..2f32)),
            RadialVelocity::new(Radf(PI)),
            (
                polygons.0[0].clone(),
                HeptaMaterial {
                    modulation: Modulation(Crimson::color(0)),
                },
                CollideWithPlayer,
                Collider::Circle(CircleCollider {
                    radius: 30.,
                    position: Default::default(),
                }),
            ),
        ))
        .entity();

    let mut spawn_child = |angle: f32, radius: f32, cloud: &mut Vec<_>| {
        let position = Vec3f::new(angle.cos() * radius, angle.sin() * radius, 0.);

        let entity = AtomBundle::spawn(
            commands,
            Vec3f::zero(),
            None,
            None,
            polygons,
            0,
            server,
            audio,
        );
        // push_child(parent, entity, commands, parents);
        commands
            .get_entity(entity)
            .insert(ChildOffset(position))
            .insert(Parent(parent));
        cloud.push(entity);
    };

    // let count = 5;
    for i in 0..children {
        let angle = (i as f32 / children as f32) * TAU;
        spawn_child(angle, REGULAR_RADIUS, &mut enemy_cloud);
    }

    commands.get_entity(parent).insert(EnemyCloud(enemy_cloud));
}

fn update_regular(
    mut q: Query<(Entity, SpinSpeed, Transform, EnemyCloud, Mut<RegularEnemy>)>,
    children: Query<Mut<ChildOffset>>,
    player_bullet: Query<Entity, Without<CollideWithPlayer>>,
    parent_haver: Query<Parent>,
    velocity_haver: Query<Velocity>,
    time: Res<DeltaTime>,
    collision: EventReader<EnemyCollideEvent>,
    server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, spin, transform, cloud, angle) in q.iter_mut() {
        angle.0 += time.delta * spin.0;

        if let Some(event) = collision.peak_read().find(|e| e.enemy == entity) {
            commands.get_entity(entity).despawn();
            // let mut rng = rand::rngs::SmallRng::from_entropy();

            for i in 0..cloud.0.len() {
                let direction = ((i as f32 / cloud.0.len() as f32) + angle.0) * TAU;

                NeutronBundle::spawn(
                    &server,
                    Transform {
                        translation: transform.translation,
                        scale: Vec2f::one(),
                        ..Default::default()
                    },
                    Velocity(Vec3f::new(direction.cos(), direction.sin(), 0.) * 4.),
                    None,
                    true,
                    &mut commands,
                );
            }

            // for child in cloud.0.iter() {
            //     let mut e = commands.get_entity(*child);

            //     if parent_haver.get(*child).is_some() {
            //         e.remove::<Parent>();
            //     }

            //     if velocity_haver.get(*child).is_none() {
            //         e.insert(Velocity(Vec3f::new(
            //             rng.gen_range(-10f32..10f32),
            //             rng.gen_range(-10f32..10f32),
            //             0.,
            //         )));
            //     }
            // }

            continue;
        }

        let radius = REGULAR_RADIUS;

        let total_children = cloud.0.len();
        for (i, child) in cloud
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, e)| children.get_mut(*e).map(|c| (i, c)))
        {
            let angle = ((i as f32 / total_children as f32) + angle.0) * TAU;

            let position = Vec3f::new(angle.cos() * radius, angle.sin() * radius, 0.);
            child.0 = position;
        }
    }
}

use rand::rngs::ThreadRng;
use winny::{app::window::Window, prelude::*};

pub(super) fn random_outside_screen(window: Vec3f, size: Vec3f, rng: &mut ThreadRng) -> Vec3f {
    let w_half = size.x / 2.;
    let h_half = size.y / 2.;

    let mut x = rng.gen_range(0f32..size.x);
    let mut y = rng.gen_range(0f32..size.y);

    let (shiftx, shifty) = match rng.gen_range(0..3) {
        0 => (true, false),
        1 => (false, true),
        2 => (true, true),
        _ => unreachable!(),
    };

    if shiftx {
        if x < w_half {
            x -= w_half * 1.1;
        } else {
            x += w_half * 1.1;
        }
    }

    if shifty {
        if y < h_half {
            y -= h_half * 1.1;
        } else {
            y += h_half * 1.1;
        }
    }

    Vec3f::new(window.x + x - w_half, window.y + y - h_half, 0.)
}

#[derive(Debug, Resource)]
pub struct EnemySpawner {
    time_elapsed: f32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl EnemySpawner {
    pub fn new() -> Self {
        Self { time_elapsed: 0. }
    }
}

pub fn spawn_enemies(
    mut spawner: ResMut<EnemySpawner>,
    enemies: Query<Entity, With<RegularEnemy>>,
    camera: Query<Transform, With<Camera>>,
    time: Res<DeltaTime>,
    window: Res<Window>,
    collision: EventReader<EnemyCollideEvent>,
    server: Res<AssetServer>,
    mut commands: Commands,
    polygons: Res<RegularPolygons>,
    mut audio: ResMut<AudioMaster>,
) {
    spawner.time_elapsed += time.delta;
    let mut rng = rand::thread_rng();

    let Some(position) = camera.iter().next() else {
        return;
    };

    if enemies.iter().count() >= 30 {
        return;
    }

    let window = Vec3f::new(window.viewport.max.x, window.viewport.max.y, 0.);

    let probability = 2. * time.delta;
    let sample: f32 = rng.gen();

    if sample < probability {
        let position = random_outside_screen(position.translation, window, &mut rng);

        spawn_regular(
            position,
            &polygons,
            &mut commands,
            &server,
            &mut audio,
            rng.gen_range(3..7),
        )
    }
}
