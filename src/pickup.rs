use crate::{
    bullet::RadialVelocity,
    collision::{CircleCollider, CollideWithPlayer, Collider, PlayerCollideEvent},
    enemy::random_outside_screen,
    player::{BulletCount, Player},
    regular::RegularPolygons,
    shaders::{materials::HeptaMaterial, Crimson},
};
use angle::Radf;
use camera::Camera;
use rand::Rng;
use std::f32::consts::PI;
use vector::{Vec2f, Vec3f};
use winny::prelude::*;

#[derive(Debug)]
pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(BulletSpawner::default())
            .add_systems(Schedule::PostUpdate, (spawn_bullet_stuff, player_pickup));
    }
}

#[derive(Debug, Component)]
pub struct BulletsPickup;

#[derive(Debug, Resource, Default)]
pub struct BulletSpawner {
    time_elapsed: f32,
}

pub fn spawn_bullets(position: Vec3f, polygons: &RegularPolygons, commands: &mut Commands) {
    commands
        .spawn((
            Transform {
                translation: position,
                scale: Vec2f::new(0.5, 0.5),
                ..Default::default()
            },
            RadialVelocity::new(Radf(PI)),
            BulletsPickup,
            polygons.0[0].clone(),
            HeptaMaterial {
                modulation: Modulation(Crimson::color(6)),
            },
            CollideWithPlayer,
            Collider::Circle(CircleCollider {
                radius: 30.,
                position: Default::default(),
            }),
        ))
        .entity();
}

fn spawn_bullet_stuff(
    mut spawner: ResMut<BulletSpawner>,
    camera: Query<Transform, With<Camera>>,
    time: Res<DeltaTime>,
    window: Res<Window>,
    // server: Res<AssetServer>,
    mut commands: Commands,
    polygons: Res<RegularPolygons>,
    // mut audio: ResMut<AudioMaster>,
) {
    spawner.time_elapsed += time.delta;
    let mut rng = rand::thread_rng();

    let Some(position) = camera.iter().next() else {
        return;
    };

    let window = Vec3f::new(window.viewport.max.x, window.viewport.max.y, 0.);

    let probability = 0.5 * time.delta;
    let sample: f32 = rng.gen();

    if sample < probability {
        let position = random_outside_screen(position.translation, window, &mut rng);

        spawn_bullets(position, &polygons, &mut commands)
    }
}

fn player_pickup(
    mut player: Query<(Mut<BulletCount>), With<Player>>,
    pickups: Query<Entity, With<BulletsPickup>>,
    events: EventReader<PlayerCollideEvent>,
    mut commands: Commands,
) {
    let Some(bullets) = player.iter_mut().next() else {
        return;
    };

    for event in events.peak_read().filter(|e| pickups.get(e.with).is_some()) {
        bullets.0 = (bullets.0 + 5).clamp(0, 10);
        commands.get_entity(event.with).despawn();
    }
}
