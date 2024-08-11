use mesh2d::Mesh2d;
use server::AssetServer;
use vector::{Vec2f, Vec3f};
use winny::prelude::*;

use crate::{
    regular::RegularPolygons,
    shaders::{atoms::NuclearAtom, SpaceHaze},
    CollisionDamage, Enemy, Velocity,
};

#[derive(Debug)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&mut self, app: &mut App) {
        app.add_systems(
            AppSchedule::PostStartUp,
            |mut commands: Commands, server: Res<AssetServer>, polygons: Res<RegularPolygons>| {
                commands.spawn(EnemyBundle::new(
                    Default::default(),
                    Default::default(),
                    &polygons,
                    &server,
                ));
            },
        );
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    transform: Transform,
    velocity: Velocity,
    // collider: Collider,
    damage: CollisionDamage,
    mesh: Handle<Mesh2d>,
    material: NuclearAtom,
}

impl EnemyBundle {
    pub fn new(
        position: Vec3f,
        velocity: Vec3f,
        polygons: &RegularPolygons,
        server: &AssetServer,
    ) -> Self {
        EnemyBundle {
            enemy: Enemy,
            transform: Transform {
                translation: position,
                scale: Vec2f::new(0.5, 0.5),
                ..Default::default()
            },
            velocity: Velocity(velocity),
            damage: CollisionDamage(1.),
            mesh: polygons.0[3].clone(),
            material: NuclearAtom {
                modulation: Modulation(SpaceHaze::purple()),
                texture: server.load("res/noise/noise.png"),
            },
        }
    }
}
