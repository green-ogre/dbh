use crate::{
    loader::LoaderApp,
    player::{DirectionalVelocity, Player},
};
use winny::{gfx::camera::Camera, math::vector::Vec3f, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<PlayerCamera>()
            .save_load_resource::<PlayerCamera>()
            .egui_resource::<PlayerCamera>()
            .add_systems(Schedule::PostUpdate, update_camera);
    }
}

fn update_camera(
    mut player_camera: ResMut<PlayerCamera>,
    player: Query<(Transform, DirectionalVelocity), With<Player>>,
    mut camera: Query<Mut<Transform>, With<Camera>>,
    delta: Res<DeltaTime>,
) {
    let Ok(camera) = camera.get_single_mut() else {
        return;
    };

    let Ok((player, vel)) = player.get_single() else {
        return;
    };

    player_camera.follow_player(vel, player, camera, &delta);
}

/// Handles main camera logic.
#[derive(Resource, AsEgui, Serialize, Deserialize)]
pub struct PlayerCamera {
    screen_shake: f32,
    max_smooth_factor: f32,
    min_smooth_factor: f32,
    max_distance: f32,
    delta: f32,
    lead_factor: f32,
    #[skip]
    follow_point: Vec3f,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            screen_shake: 1.0,
            max_smooth_factor: 1.0,
            min_smooth_factor: 1.0,
            max_distance: 100.0,
            delta: 10.0,
            lead_factor: 1.0,
            follow_point: Vec3f::new(0., 0., 0.),
        }
    }
}

impl PlayerCamera {
    pub fn follow_player(
        &mut self,
        velocity: &DirectionalVelocity,
        player: &Transform,
        camera: &mut Transform,
        dt: &DeltaTime,
    ) {
        let distance_to_target = (player.translation - self.follow_point).magnitude();

        let target: Vec3f = Vec3f::from(*velocity).normalize() + player.translation;

        if distance_to_target < self.delta {
            self.follow_point = target;
        } else {
            // Calculate a dynamic smooth factor based on the distance
            let smooth_factor = if distance_to_target > self.max_distance {
                self.max_smooth_factor
            } else {
                let t = distance_to_target / self.max_distance;
                self.min_smooth_factor + (self.max_smooth_factor - self.min_smooth_factor) * t
            };

            // Move the camera smoothly towards the target position
            self.follow_point += (target - self.follow_point) * smooth_factor * dt.delta;
        }

        camera.translation = self.follow_point;
    }
}
