use mesh2d::{Mesh2d, Points};
use std::f32::consts::TAU;
use vector::Vec2f;
use winny::prelude::*;

use crate::shaders::{
    materials::{
        HeptaMaterial, HexaMaterial, NonagonMaterial, OctagonMaterial, PentagonMaterial,
        QuadrilateralMaterial, TriangleMaterial,
    },
    Crimson, SpaceHaze,
};

#[derive(Debug)]
pub struct RegularPolygonsPlugin;

impl Plugin for RegularPolygonsPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<RegularPolygons>();
    }
}

#[derive(Debug, Resource, Clone)]
pub struct RegularPolygons(pub [Handle<Mesh2d>; 7]);

impl RegularPolygons {
    pub fn new(radius: f32, assets: &mut Assets<Mesh2d>) -> Self {
        let make = move |sides: usize| {
            let mut points = Points::default();
            for i in 0..sides {
                let theta = i as f32 * (TAU / sides as f32);
                points.add(Vec2f::new(radius * theta.cos(), radius * theta.sin()));
            }
            Mesh2d::from_points(points).unwrap()
        };

        let polygons = std::array::from_fn(|i| assets.add(make(i + 3)));

        RegularPolygons(polygons)
    }
}

#[derive(Debug, Resource, Clone)]
pub struct PolygonMaterials;

impl PolygonMaterials {
    pub fn spawn_with_material(
        commands: &mut Commands,
        bundle: impl Bundle,
        index: usize,
    ) -> Entity {
        match index {
            6 => commands
                .spawn((
                    bundle,
                    NonagonMaterial {
                        modulation: Modulation(Crimson::color(0)),
                    },
                ))
                .entity(),
            5 => commands
                .spawn((
                    bundle,
                    OctagonMaterial {
                        modulation: Modulation(Crimson::color(1)),
                    },
                ))
                .entity(),
            4 => commands
                .spawn((
                    bundle,
                    HeptaMaterial {
                        modulation: Modulation(Crimson::color(2)),
                    },
                ))
                .entity(),
            3 => commands
                .spawn((
                    bundle,
                    HexaMaterial {
                        modulation: Modulation(Crimson::color(3)),
                    },
                ))
                .entity(),
            2 => commands
                .spawn((
                    bundle,
                    PentagonMaterial {
                        modulation: Modulation(Crimson::color(9)),
                    },
                ))
                .entity(),
            1 => commands
                .spawn((
                    bundle,
                    QuadrilateralMaterial {
                        modulation: Modulation(Crimson::color(8)),
                    },
                ))
                .entity(),
            0 => commands
                .spawn((
                    bundle,
                    TriangleMaterial {
                        modulation: Modulation(Crimson::color(7)),
                    },
                ))
                .entity(),
            _ => {
                unreachable!()
            }
        }
    }
}
