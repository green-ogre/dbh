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

pub struct ChildrenPlugin;

impl Plugin for ChildrenPlugin {
    fn build(&mut self, app: &mut App) {
        app.add_systems(Schedule::Update, (move_children, manage_children))
            .add_systems(Schedule::PostUpdate, manage_parents);
    }
}

/// Describes a parent-child relationship.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub struct Children(pub Vec<Entity>);

/// Describes a parent-child relationship.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component)]
pub struct Parent(pub Entity);

/// Add a parent-child relationship between the parent entity and the child entity.
pub fn push_child(
    parent: Entity,
    child: Entity,
    commands: &mut Commands,
    parents: &mut Query<Option<Mut<Children>>>,
) {
    match parents.get_mut(parent) {
        Some(Some(children)) => {
            children.0.push(child);
        }
        Some(None) => {
            commands.get_entity(parent).insert(Children(vec![child]));
        }
        None => {}
    }
}

fn move_children(
    parents: Query<Transform, With<Children>>,
    mut children: Query<(Parent, Mut<Transform>)>,
) {
    for (Parent(entity), transform) in children.iter_mut() {
        if let Some(parent) = parents.get(*entity) {
            *transform = *parent;
        }
    }
}

fn manage_children(
    parents: Query<Entity, With<Children>>,
    mut children: Query<(Entity, Mut<Parent>)>,
    mut commands: Commands,
) {
    for (child, Parent(parent)) in children.iter_mut() {
        if parents.get(*parent).is_none() {
            commands.get_entity(child).despawn();
        }
    }
}

pub fn manage_parents(
    mut parents: Query<(Entity, Mut<Children>)>,
    children: Query<Entity, With<Parent>>,
    mut commands: Commands,
) {
    for (parent, c) in parents.iter_mut() {
        c.0.retain(|e| children.get(*e).is_some());
        if c.0.is_empty() {
            commands.get_entity(parent).remove::<Children>();
        }
    }
}
