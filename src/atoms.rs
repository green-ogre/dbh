use crate::{
    audio::AudioMaster,
    bullet::{NeutronBundle, Progenitor, RadialVelocity},
    camera::{PlayerCamera, ScreenShake},
    collision::{CircleCollider, CollideWithPlayer, Collider, EnemyCollideEvent},
    regular::{PolygonMaterials, RegularPolygons},
    should_run_game, CollisionDamage, Enemy, GetOrLog, RandomDirectionIterator, Velocity,
};
use angle::Radf;
use fxhash::FxHashSet;
use mesh2d::Mesh2d;
use rand::{Rng, SeedableRng};
use server::AssetServer;
use std::f32::consts::{FRAC_PI_2, PI};
use vector::{Vec2f, Vec3f};
use winny::{ecs::sets::IntoSystemStorage, prelude::*};

#[derive(Debug)]
pub struct AtomPlugin;

impl Plugin for AtomPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(TotalEvents::default())
            .add_systems(Schedule::PostUpdate, handle_neutron.run_if(should_run_game));
    }
}

#[derive(Component)]
pub struct Atom;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct Events(pub u32);

#[derive(Bundle)]
pub struct AtomBundle {
    atom: Atom,
    enemy: Enemy,
    transform: Transform,
    collider: Collider,
    progenitor: Progenitor,
    events: Events,
    damage: CollisionDamage,
    mesh: Handle<Mesh2d>,
    radial: RadialVelocity,
    with_player: CollideWithPlayer,
}

impl AtomBundle {
    pub fn spawn(
        commands: &mut Commands,
        position: Vec3f,
        velocity: Option<Vec3f>,
        progenitor: Option<Entity>,
        polygons: &RegularPolygons,
        events: u32,
        server: &AssetServer,
        _audio: &mut AudioMaster,
    ) -> Entity {
        // audio.queue_bundle(AudioBundle {
        //     handle: server.load("res/RPG_Essentials_Free/10_Battle_SFX/77_flesh_02.wav"),
        //     playback_settings: PlaybackSettings::default().with_volume(10.0),
        // });
        if let Some(vel) = velocity {
            PolygonMaterials::spawn_with_material(
                commands,
                (
                    Self::new(position, progenitor, polygons, events),
                    Velocity(vel),
                ),
                6 - events as usize,
            )
        } else {
            PolygonMaterials::spawn_with_material(
                commands,
                Self::new(position, progenitor, polygons, events),
                6 - events as usize,
            )
        }
    }

    fn new(
        position: Vec3f,
        // velocity: Vec3f,
        progenitor: Option<Entity>,
        polygons: &RegularPolygons,
        events: u32,
    ) -> Self {
        AtomBundle {
            atom: Atom,
            enemy: Enemy,
            transform: Transform {
                translation: position,
                scale: Vec2f::new(0.5, 0.5),
                ..Default::default()
            },
            // velocity: Velocity(velocity),
            collider: Collider::Circle(CircleCollider {
                radius: 40.,
                ..Default::default()
            }),
            events: Events(events),
            progenitor: Progenitor(progenitor),
            damage: CollisionDamage(1.),
            mesh: polygons.0[6 - events as usize].clone(),
            radial: RadialVelocity::new(Radf(
                PI + rand::rngs::SmallRng::from_entropy().gen_range(-1f32..1f32),
            )),
            with_player: CollideWithPlayer,
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct TotalEvents(pub usize);

fn handle_neutron(
    q: Query<(Entity, Transform, Option<Velocity>, Progenitor, Events), With<Atom>>,
    bullets: Query<(Entity, Transform, Velocity, Progenitor)>,
    reader: EventReader<EnemyCollideEvent>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut total_events: ResMut<TotalEvents>,
    polygons: Res<RegularPolygons>,
    mut camera: ResMut<PlayerCamera>,
    delta: Res<DeltaTime>,
    mut audio: ResMut<AudioMaster>,
) {
    let mut already_handled = FxHashSet::default();

    for (
        (atom, atom_position, atom_velocity, atom_progenitor, events),
        (bullet, _bullet_transform, bullet_velocity, progenitor),
    ) in reader.peak_read().filter_map(|e| {
        bullets
            .get_or_log(e.with)
            .and_then(|b| Some((q.get_or_log(e.enemy)?, b)))
    }) {
        if !already_handled.insert(atom) {
            continue;
        }
        match (atom_progenitor.0, progenitor.0) {
            (Some(atom_progenitor), Some(progenitor)) if atom_progenitor == progenitor => continue,
            _ => {}
        }

        commands.get_entity(atom).despawn();
        commands.get_entity(bullet).despawn();
        total_events.0 += 1;

        if events.0 >= 6 {
            continue;
        }

        let direction = bullet_velocity.0 + atom_velocity.map_or(Default::default(), |v| v.0);
        // let direction =
        //     bullet_velocity.0 + (atom_position.translation - bullet_transform.translation) * 0.25;
        let direction = direction.normalize();

        let directions = RandomDirectionIterator::new(direction, Radf(FRAC_PI_2));

        for direction in directions.clone().take(2) {
            AtomBundle::spawn(
                &mut commands,
                atom_position.translation,
                Some(direction),
                Some(atom),
                &polygons,
                events.0 + 1,
                &server,
                &mut audio,
            );
        }

        for direction in directions.take(3) {
            NeutronBundle::spawn(
                &server,
                Transform {
                    translation: atom_position.translation,
                    scale: Vec2f::one(),
                    ..Default::default()
                },
                Velocity(direction * 2.),
                Some(atom),
                true,
                &mut commands,
            );
        }
    }

    if !already_handled.is_empty() {
        camera.push_screen_shake(ScreenShake::new(
            10.,
            0.15,
            delta.wrapping_elapsed_as_seconds(),
        ));
    }
}
