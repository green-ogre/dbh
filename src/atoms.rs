use crate::{
    bullet::{NeutronBundle, Progenitor},
    collision::{CircleCollider, Collider, EnemyCollideEvent},
    shaders::{atoms::NuclearAtom, ColorPalette, Paper8, SpaceHaze},
    CollisionDamage, Enemy, GetOrLog, RandomDirectionIterator, Velocity,
};
use angle::Radf;
use fxhash::FxHashSet;
use mesh2d::Mesh2d;
use rand::Rng;
use server::AssetServer;
use std::f32::consts::FRAC_PI_2;
use vector::{Vec2f, Vec3f};
use winny::prelude::*;

#[derive(Debug)]
pub struct AtomPlugin;

impl Plugin for AtomPlugin {
    fn build(&mut self, app: &mut App) {
        app.add_systems(
            Schedule::StartUp,
            |mut commands: Commands, server: Res<AssetServer>| {
                let mut rng = rand::thread_rng();
                for _ in 0..50 {
                    let x = rng.gen_range(-500f32..500f32);
                    let y = rng.gen_range(-500f32..500f32);

                    let velocity =
                        Vec3f::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5), 0.);

                    commands.spawn(AtomBundle::new(
                        Vec3f::new(x, y, 0.),
                        velocity,
                        None,
                        &server,
                        0,
                    ));
                }
            },
        )
        .add_systems(Schedule::PostUpdate, handle_neutron);
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
    velocity: Velocity,
    collider: Collider,
    progenitor: Progenitor,
    events: Events,
    // collides: CollideWithEnemy,
    damage: CollisionDamage,
    // lifespan: Lifespan,
    // uptime: Uptime,
    mesh: Handle<Mesh2d>,
    material: NuclearAtom,
}

impl AtomBundle {
    pub fn new(
        position: Vec3f,
        velocity: Vec3f,
        progenitor: Option<Entity>,
        server: &AssetServer,
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
            velocity: Velocity(velocity),
            collider: Collider::Circle(CircleCollider {
                radius: 40.,
                ..Default::default()
            }),
            events: Events(events),
            progenitor: Progenitor(progenitor),
            damage: CollisionDamage(1.),
            mesh: server.load("res/saved/bullet_1_mesh.msh"),
            material: NuclearAtom {
                modulation: Modulation(SpaceHaze::purple()),
                texture: server.load("res/noise/noise.png"),
            },
        }
    }
}

fn handle_neutron(
    q: Query<(Entity, Transform, Velocity, Progenitor, Events), With<Atom>>,
    bullets: Query<(Entity, Transform, Velocity, Progenitor)>,
    reader: EventReader<EnemyCollideEvent>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let mut already_handled = FxHashSet::default();

    for (
        (atom, atom_position, atom_velocity, atom_progenitor, events),
        (bullet, bullet_transform, bullet_velocity, progenitor),
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

        if events.0 >= 6 {
            continue;
        }

        let direction = bullet_velocity.0 + atom_velocity.0;
        // let direction =
        //     bullet_velocity.0 + (atom_position.translation - bullet_transform.translation) * 0.25;
        let direction = direction.normalize();

        let directions = RandomDirectionIterator::new(direction, Radf(FRAC_PI_2));

        for direction in directions.clone().take(2) {
            commands.spawn(AtomBundle::new(
                atom_position.translation,
                direction,
                Some(atom),
                &server,
                events.0 + 1,
            ));
        }

        for direction in directions.take(3) {
            commands.spawn(NeutronBundle::new(
                &server,
                Transform {
                    translation: atom_position.translation,
                    scale: Vec2f::one(),
                    ..Default::default()
                },
                Velocity(direction * 2.),
                Some(atom),
            ));
        }
    }
}
