use crate::{player::Player, Enemy};

use super::*;

pub fn update_player_collision(
    colliders: Query<(Entity, Transform, Collider), With<CollideWithPlayer>>,
    player: Query<(Transform, Collider), With<Player>>,
    mut map: ResMut<PlayerCollisionMap>,
    mut writer: EventWriter<PlayerCollideEvent>,
) {
    let Some((p_trans, p_coll)) = player.iter().next() else {
        return;
    };
    let p_coll = p_coll.absolute(p_trans, None);
    let set = &mut map.0;

    for (entity, transform, collider) in colliders.iter() {
        let absolute_enemy = collider.absolute(transform, None);

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
    let log = colliders.iter().next().is_some();

    for (enemy, e_trans, e_coll) in enemies.iter() {
        let absolute_enemy = e_coll.absolute(e_trans, log.then_some("Enemy"));
        let entry = map.0.entry(enemy).or_default();

        for (entity, transform, collider) in colliders.iter() {
            let absolute = collider.absolute(transform, log.then_some("Player"));

            let collision = absolute.collides_with(&absolute_enemy);

            if collision {
                if entry.insert(entity) {
                    writer.send(EnemyCollideEvent {
                        enemy,
                        with: entity,
                    });
                    // info!("Enemy collision entered!");
                }
            } else if entry.remove(&entity) {
                // info!("Enemy collision exited!");
            }
        }
    }
}
