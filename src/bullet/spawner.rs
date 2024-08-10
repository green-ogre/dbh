use crate::audio::AudioMaster;
use std::sync::Arc;
use winny::{asset::server::AssetServer, math::vector::Vec3f, prelude::*};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_timer::<BulletEvent>().add_systems(
            Schedule::Update,
            (
                initial_emit_bullet,
                bullet_timer,
                bullet_remover,
                bullet_lifetime,
            ),
        );
    }
}

/// Generate a vector to the nearest position.
pub fn to_nearest<'a>(position: &Vec3f, enemies: impl Iterator<Item = &'a Vec3f>) -> Option<Vec3f> {
    enemies
        .min_by(|a, b| {
            let dista = a.dist2(position);
            let distb = b.dist2(position);
            dista.total_cmp(&distb)
        })
        .map(|p| *p - *position)
}

/// A marker type that indicates an entity should be removed on collision with an enemy.
#[derive(Debug, Component)]
pub struct RemoveOnCollision;

/// The time a bullet should last in seconds.
#[derive(Debug, Component, Copy, Clone, PartialEq)]
pub struct Lifespan(pub f32);

/// The time a bullet has been around in seconds.
#[derive(Debug, Component, Copy, Clone, PartialEq)]
pub struct Uptime(pub f32);

pub type BulletSpawnerFn =
    Arc<dyn Fn(&Transform, &mut Commands, &AssetServer, &mut AudioMaster) + Send + Sync>;

#[derive(Event)]
pub struct BulletEvent {
    trigger: Entity,
    spawner: BulletSpawnerFn,
    retrigger: bool,
}

#[derive(Component)]
pub struct BulletSpawner {
    spawner: BulletSpawnerFn,
    spawn_period: f32,
    has_emitted: bool,
}

impl BulletSpawner {
    pub fn new<F>(period: f32, spawner: F) -> Self
    where
        F: Fn(&Transform, &mut Commands, &AssetServer, &mut AudioMaster) + Send + Sync + 'static,
    {
        Self {
            spawner: Arc::new(spawner),
            spawn_period: period,
            has_emitted: false,
        }
    }

    pub fn spawn(&self, id: Entity, commands: &mut Commands) {
        commands.spawn(Timer::new(
            self.spawn_period,
            BulletEvent {
                spawner: Arc::clone(&self.spawner),
                trigger: id,
                retrigger: true,
            },
        ));
    }
}

pub fn initial_emit_bullet(
    mut spawners: Query<(Entity, Mut<BulletSpawner>)>,
    mut commands: Commands,
) {
    for (id, spawner) in spawners.iter_mut() {
        if !spawner.has_emitted {
            spawner.has_emitted = true;
            spawner.spawn(id, &mut commands);
        }
    }
}

pub fn bullet_timer(
    spawners: Query<(Entity, BulletSpawner, Transform)>,
    reader: EventReader<BulletEvent>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut audio_master: ResMut<AudioMaster>,
) {
    for BulletEvent {
        trigger,
        spawner,
        retrigger,
    } in reader.read()
    {
        let Some((entity, trigger, transform)) = spawners.get(trigger) else {
            continue;
        };

        spawner(transform, &mut commands, &server, &mut audio_master);
        if retrigger {
            trigger.spawn(entity, &mut commands);
        }
    }
}

pub fn bullet_remover(
    bullets: Query<RemoveOnCollision>,
    // events: EventReader<EnemyCollideEvent>,
    mut commands: Commands,
) {
    // for event in events.peak_read() {
    //     if bullets.get(event.with).is_some() {
    //         commands.get_entity(event.with).despawn();
    //     }
    // }
}

pub fn bullet_lifetime(
    mut bullets: Query<(Entity, Mut<Uptime>, Lifespan)>,
    mut commands: Commands,
    delta: Res<DeltaTime>,
) {
    for (entity, uptime, lifespan) in bullets.iter_mut() {
        uptime.0 += delta.delta;
        if uptime.0 >= lifespan.0 {
            commands.get_entity(entity).despawn();
        }
    }
}
