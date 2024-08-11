use fxhash::FxHashMap;
use vector::Vec3f;
use winny::prelude::*;

use super::AbsoluteCollider;

#[derive(Debug, Clone, Copy)]
pub struct SpatialData {
    pub entity: Entity,
    pub position: Vec3f,
    pub collider: AbsoluteCollider,
}

pub struct SpatialHash {
    cell_size: f32,
    objects: FxHashMap<(i32, i32), Vec<SpatialData>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        SpatialHash {
            cell_size,
            objects: FxHashMap::default(),
        }
    }

    fn hash(&self, position: &Vec3f) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, data: SpatialData) {
        let cell = self.hash(&data.position);
        self.objects.entry(cell).or_default().push(data);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn nearby_objects<'a>(
        &'a self,
        position: &Vec3f,
    ) -> impl Iterator<Item = &'a SpatialData> + 'a {
        let cell = self.hash(position);

        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                self.objects
                    .get(&(cell.0 + dx, cell.1 + dy))
                    .into_iter()
                    .flatten()
            })
        })
    }
}
