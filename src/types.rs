use angle::Radf;
use rand::Rng;
use winny::{gfx::transform::Transform, math::vector::Vec3f, prelude::*};

#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec3f);

#[derive(Debug, Default, Component, Clone, Copy, PartialEq)]
pub struct CollisionDamage(pub f32);

/// We'll consider this to be anything other than the player that should be collided with.
#[derive(Debug, Component)]
pub struct Enemy;

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
        app.add_systems(Schedule::Update, (manage_children,))
            .add_systems(Schedule::PostUpdate, (manage_parents, move_children));
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

pub trait GetOrLog {
    type Output<'a>
    where
        Self: 'a;

    fn get_or_log(&self, entity: Entity) -> Option<Self::Output<'_>>;
}

impl<T, F> GetOrLog for Query<'_, '_, T, F>
where
    T: QueryData,
    F: Filter,
{
    type Output<'a> = <<T as QueryData>::ReadOnly as WorldQuery>::Item<'a> where Self: 'a;

    fn get_or_log(&self, entity: Entity) -> Option<Self::Output<'_>> {
        match self.get(entity) {
            None => {
                let tie = std::any::type_name::<<T as QueryData>::ReadOnly>();
                tracing_log::log::warn!("expected a value of type {tie}, but found None");
                None
            }
            value => value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RandomDirectionIterator {
    direction: Vec3f,
    angle: f32,
    rng: rand::rngs::ThreadRng,
}

impl RandomDirectionIterator {
    pub fn new(direction: Vec3f, angle: Radf) -> Self {
        RandomDirectionIterator {
            direction: direction.normalize(),
            angle: angle.0,
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for RandomDirectionIterator {
    type Item = Vec3f;

    fn next(&mut self) -> Option<Self::Item> {
        // Generate a random rotation angle within the specified arc
        let random_angle = self.rng.gen_range(0.0..self.angle);
        let random_rotation = self.rng.gen_range(0.0..std::f32::consts::TAU);

        // Create an orthonormal basis
        let u = if self.direction.x.abs() < 0.9 {
            Vec3f::new(1.0, 0.0, 0.0)
        } else {
            Vec3f::new(0.0, 1.0, 0.0)
        };
        let v = self.direction.cross(&u).normalize();
        let w = self.direction.cross(&v);

        // Compute the rotated vector
        let rotated = self.direction * random_angle.cos()
            + (v * random_rotation.cos() + w * random_rotation.sin()) * random_angle.sin();

        Some(rotated.normalize())
    }
}
