use super::Collider;
use crate::Parent;
use server::AssetServer;
use vector::{Vec2f, Vec3f};
use winny::{
    gfx::{
        render_pipeline::material::Material2d,
        sprite::{Sprite, SpriteBundle},
        transform::Transform,
    },
    prelude::*,
};

#[derive(Debug, Component)]
pub struct Indicator;

#[derive(Debug, Component)]
pub struct Indicated;

#[derive(Debug, Resource)]
pub struct ShowIndicators(pub bool);

#[derive(Debug, Bundle)]
pub struct IndicatorBundle {
    transform: Transform,
    hierarchy: Parent,
    indicator: Indicator,
}

fn pos_from_collider(collider: &Collider) -> Vec3f {
    match collider {
        Collider::Rect(rect) => Vec3f::new(
            // rect.tl.x - rect.size.x / 2.,
            // rect.tl.y - rect.size.y / 2.,
            rect.tl.x, rect.tl.y, 0f32,
        ),
        Collider::Circle(circle) => circle.position,
    }
}

fn scale_from_collider(collider: &Collider) -> Vec2f {
    match collider {
        Collider::Rect(rect) => {
            let x_scale = rect.size.x / 256.;
            let y_scale = rect.size.y / 256.;

            Vec2f::new(x_scale, y_scale)
        }
        Collider::Circle(circle) => {
            let scale = circle.radius / 128.;
            Vec2f::new(scale, scale)
        }
    }
}

impl IndicatorBundle {
    pub fn new(
        parent: Entity,
        parent_transform: Transform,
        collider: Collider,
        server: &mut AssetServer,
    ) -> (Self, SpriteBundle) {
        let sprite_source = match collider {
            Collider::Rect(_) => "res/textures/rect.png",
            Collider::Circle(_) => "res/textures/circle.png",
        };
        let sprite_position = pos_from_collider(&collider);
        let sprite_scale = scale_from_collider(&collider);

        (
            Self {
                transform: parent_transform,
                hierarchy: Parent(parent),
                indicator: Indicator,
            },
            SpriteBundle {
                material: Material2d {
                    texture: server.load(sprite_source),
                    ..Default::default()
                },
                sprite: Sprite {
                    position: sprite_position,
                    scale: sprite_scale,
                    z: 1000,
                    ..Default::default()
                },
            },
        )
    }
}

pub fn manage_indicators(
    non_indicated: Query<(Entity, Collider, Transform), Without<Indicated>>,
    indicated: Query<Entity, (With<Indicated>, Without<Indicator>)>,
    indicators: Query<Entity, With<Indicator>>,
    mut commands: Commands,
    input: EventReader<KeyInput>,
    mut server: ResMut<AssetServer>,
    mut show: ResMut<ShowIndicators>,
) {
    if input.peak_read().any(|k| {
        matches!(
            k,
            KeyInput {
                code: KeyCode::H,
                state: KeyState::Pressed,
                ..
            }
        )
    }) {
        show.0 = !show.0;
    }

    if show.0 {
        for (entity, collider, transform) in non_indicated.iter() {
            let child = commands
                .spawn(IndicatorBundle::new(
                    entity,
                    *transform,
                    *collider,
                    &mut server,
                ))
                .entity();
            commands.get_entity(entity).insert(Indicated);
            // push_child(entity, child, &mut commands, &mut parents);
        }
    } else {
        for entity in indicated.iter() {
            commands.get_entity(entity).remove::<Indicated>();
        }

        for entity in indicators.iter() {
            commands.get_entity(entity).despawn();
        }
    }
}
