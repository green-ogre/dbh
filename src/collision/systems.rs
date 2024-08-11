use super::*;
use crate::{player::Player, Enemy};
use spatial::{SpatialData, SpatialHash};

pub fn update_player_collision(
    colliders: Query<(Entity, Transform, Collider), With<CollideWithPlayer>>,
    player: Query<(Transform, Collider), With<Player>>,
    mut map: ResMut<PlayerCollisionMap>,
    mut writer: EventWriter<PlayerCollideEvent>,
) {
    let Some((p_trans, p_coll)) = player.iter().next() else {
        return;
    };
    let p_coll = p_coll.absolute(p_trans);
    let set = &mut map.0;

    for (entity, transform, collider) in colliders.iter() {
        let absolute_enemy = collider.absolute(transform);

        let collision = absolute_enemy.collides_with(&p_coll);

        if collision {
            if set.insert(entity) {
                writer.send(PlayerCollideEvent { with: entity });
                info!("Player collision entered!");
            }
        } else if set.remove(&entity) {
            // info!("Player collision exited!");
        }
    }
}

pub fn update_enemy_collision(
    colliders: Query<(Entity, Transform, Collider), With<CollideWithEnemy>>,
    enemies: Query<(Entity, Transform, Collider), With<Enemy>>,
    mut map: ResMut<EnemyCollisionMap>,
    mut writer: EventWriter<EnemyCollideEvent>,
) {
    // the grid size is very small because there's not much penalty for sparse grid distribution
    let mut spatial = SpatialHash::new(100.);

    // first fill the spatial hash grid. We insert the colliders because we expect there to be more of them.
    for (entity, transform, collider) in colliders.iter() {
        let absolute = collider.absolute(transform);
        spatial.insert(SpatialData {
            entity,
            position: absolute.position(),
            collider: absolute,
        });
    }

    // Then do collision checking on them
    for (entity, transform, collider) in enemies.iter() {
        let absolute = collider.absolute(transform);
        let entry = map.0.entry(entity).or_default();

        for SpatialData {
            entity: se,
            collider: sc,
            ..
        } in spatial.nearby_objects(&absolute.position())
        {
            let collision = absolute.collides_with(sc);

            if collision {
                if entry.insert(entity) {
                    writer.send(EnemyCollideEvent {
                        enemy: entity,
                        with: *se,
                    });
                }
            } else if entry.remove(&entity) {
                // info!("Enemy collision exited!");
            }
        }
    }
}
