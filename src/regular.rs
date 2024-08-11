use mesh2d::{Mesh2d, Points};
use std::f32::consts::TAU;
use vector::Vec2f;
use winny::prelude::*;

#[derive(Debug)]
pub struct RegularPolygonsPlugin;

impl Plugin for RegularPolygonsPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<RegularPolygons>().add_systems(
            Schedule::StartUp,
            |mut assets: ResMut<Assets<Mesh2d>>, mut commands: Commands| {
                commands.insert_resource(RegularPolygons::new(40., &mut assets));
            },
        );
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
