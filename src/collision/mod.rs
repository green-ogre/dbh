use self::systems::update_player_collision;
use fxhash::{FxHashMap, FxHashSet};
use winny::{
    gfx::cgmath::{Matrix4, Quaternion, Zero},
    math::{
        matrix::{scale_matrix4x4f, translation_matrix4x4f, Matrix4x4f},
        vector::{Vec3f, Vec4f},
    },
    prelude::*,
};

// pub mod indicators;
mod systems;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(PlayerCollisionMap::default())
            .insert_resource(EnemyCollisionMap::default())
            // .insert_resource(indicators::ShowIndicators(false))
            .register_event::<EnemyCollideEvent>()
            .register_event::<PlayerCollideEvent>()
            .add_systems(
                Schedule::Update,
                (update_player_collision,), //update_enemy_collision), // indicators::manage_indicators,
            );
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct PlayerCollideEvent {
    /// The entity the player collided with.
    pub with: Entity,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct EnemyCollideEvent {
    pub enemy: Entity,
    /// The entity this enemy collided with.
    pub with: Entity,
}

/// A marker component that indicates this entity should
/// collide with the player.
#[derive(Debug, Component, Clone, Copy)]
pub struct CollideWithPlayer;

/// A marker component that indicates this entity should
/// collide with enemies.
#[derive(Debug, Component, Clone, Copy)]
pub struct CollideWithEnemy;

/// Keeps track of what entities are
/// colliding with the platyer.
#[derive(Debug, Default, Clone, Resource)]
pub struct PlayerCollisionMap(FxHashSet<Entity>);

/// Keeps track of what entities (value) are
/// colliding with the enemy (key).
#[derive(Debug, Default, Clone, Resource)]
pub struct EnemyCollisionMap(FxHashMap<Entity, FxHashSet<Entity>>);

pub trait CollidesWith<T> {
    fn collides_with(&self, other: &T) -> bool;
}

/// To check for collisions, first convert this enum into an [AbsoluteCollider]
/// with [Collider::absolute].
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Collider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl Collider {
    pub fn absolute(&self, transform: &Transform, log: Option<&'static str>) -> AbsoluteCollider {
        match self {
            Self::Rect(rect) => {
                let mut abs = *rect;
                abs.tl += transform.translation;
                abs.size.x *= transform.scale.x;
                abs.size.y *= transform.scale.y;

                if transform.rotation != Quaternion::zero() {
                    panic!("Don't rotate AABBs!");
                }

                AbsoluteCollider::Rect(abs)
            }
            Self::Circle(circle) => {
                let homogenous_circle_position = Vec4f::to_homogenous(circle.position);
                let scale = scale_matrix4x4f(transform.scale);
                let rotation = Matrix4::from(transform.rotation);
                let rotation = Matrix4x4f { m: rotation.into() };
                let translation =
                    translation_matrix4x4f(Vec4f::to_homogenous(transform.translation));

                // Apply the transformation of the entity's Transform to the collider position
                let transformed_position =
                    translation * scale * rotation * homogenous_circle_position;
                let radius = circle.radius * (transform.scale.x + transform.scale.y) / 2f32;

                if let Some(log) = log {
                    trace!(
                        "{log}: originial: {homogenous_circle_position:?}, transformed: {transformed_position:?}, transform: {transform:?}"
                    );
                }

                AbsoluteCollider::Circle(CircleCollider {
                    position: transformed_position.into(),
                    radius,
                })
            }
        }
    }
}

pub enum AbsoluteCollider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl CollidesWith<Self> for AbsoluteCollider {
    fn collides_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rect(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Rect(s), Self::Circle(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Circle(o)) => s.collides_with(o),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct RectCollider {
    pub tl: Vec3f,
    pub size: Vec3f,
}

impl RectCollider {
    pub fn br(&self) -> Vec3f {
        self.tl + self.size
    }
}

impl CollidesWith<Self> for RectCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let not_collided = other.tl.y > self.br().y
            || other.tl.x > self.br().x
            || other.br().y < self.tl.y
            || other.br().x < self.tl.x;

        !not_collided
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct CircleCollider {
    pub position: Vec3f,
    pub radius: f32,
}

impl CollidesWith<Self> for CircleCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let distance = self.position.dist2(&other.position);
        let combined_radii = self.radius.powi(2) + other.radius.powi(2);

        distance <= combined_radii
    }
}

impl CollidesWith<RectCollider> for CircleCollider {
    fn collides_with(&self, other: &RectCollider) -> bool {
        let dist_x = (self.position.x - (other.tl.x - other.size.x * 0.5)).abs();
        let dist_y = (self.position.y - (other.tl.y - other.size.y * 0.5)).abs();

        if dist_x > other.size.x * 0.5 + self.radius {
            return false;
        }

        if dist_y > other.size.y * 0.5 + self.radius {
            return false;
        }

        if dist_x <= other.size.x * 0.5 {
            return true;
        }

        if dist_y <= other.size.y * 0.5 {
            return true;
        }

        let corner_dist =
            (dist_x - other.size.x * 0.5).powi(2) + (dist_y - other.size.y * 0.5).powi(2);

        corner_dist <= self.radius.powi(2)
    }
}

impl CollidesWith<CircleCollider> for RectCollider {
    fn collides_with(&self, other: &CircleCollider) -> bool {
        other.collides_with(self)
    }
}
