use winny::{gfx::transform::Transform, math::vector::Vec3f, prelude::*};

#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec3f);

#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct CollisionDamage(pub f32);

/// Represents health.
///
/// The members are private to preserve their invariants.
#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct Health {
    total: f32,
    current: f32,
    bar_offset: f32,
}

impl Health {
    pub fn new(total: f32, offset: f32) -> Self {
        Self {
            total,
            current: total,
            bar_offset: offset,
        }
    }

    pub fn total(&self) -> f32 {
        self.total
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn set_current(&mut self, value: f32) {
        self.current = value.clamp(0., self.total);
    }

    pub fn is_depleted(&self) -> bool {
        self.current <= 0f32
    }

    pub fn is_full(&self) -> bool {
        self.current == self.total
    }

    pub fn ratio(&self) -> f32 {
        self.current / self.total
    }

    pub fn offset(&self) -> f32 {
        self.bar_offset
    }
}

/// Describes a parent-child relationship.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component)]
pub struct ChildOf(pub Entity);

pub fn move_children(
    parents: Query<Transform, Without<ChildOf>>,
    mut children: Query<(ChildOf, Mut<Transform>)>,
) {
    for (ChildOf(entity), transform) in children.iter_mut() {
        if let Some(parent) = parents.get(*entity) {
            *transform = *parent;
        }
    }
}

pub fn cull_children(
    parents: Query<Entity>,
    mut children: Query<(Entity, Mut<ChildOf>)>,
    mut commands: Commands,
) {
    for (child, ChildOf(parent)) in children.iter_mut() {
        if parents.get(*parent).is_none() {
            commands.get_entity(child).despawn();
        }
    }
}
