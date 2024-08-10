use crate::{
    loader::LoaderApp,
    player::{DirectionalVelocity, Player},
};
use rand::Rng;
use winny::{gfx::camera::Camera, math::vector::Vec3f, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<PlayerCamera>()
            .save_load_resource::<PlayerCamera>()
            .egui_resource::<PlayerCamera>()
            .egui_resource::<ScreenShake>()
            .insert_resource(ScreenShake {
                intensity: 42.0,
                duration: 0.5,
                start_time: 0.0,
            })
            .add_systems(Schedule::Update, shake_screen)
            .add_systems(Schedule::PostUpdate, update_camera);
    }
}

#[derive(Resource, AsEgui, Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct ScreenShake {
    intensity: f32,
    duration: f32,
    start_time: f32,
}

fn shake_screen(
    reader: EventReader<KeyInput>,
    mut camera: ResMut<PlayerCamera>,
    shake: Res<ScreenShake>,
    delta: Res<DeltaTime>,
) {
    for event in reader.peak_read() {
        if matches!(
            event,
            KeyInput {
                code: KeyCode::I,
                state: KeyState::Pressed,
                ..
            }
        ) {
            let mut shake = shake.clone();
            shake.start_time = delta.wrapping_elapsed_as_seconds();
            camera.push_screen_shake(shake)
        }
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

    player_camera.apply_screen_shake(&delta);
    player_camera.follow_player(vel, player, camera, &delta);
}

/// Handles main camera logic.
#[derive(Resource, AsEgui, Serialize, Deserialize)]
pub struct PlayerCamera {
    #[skip]
    screen_shake: Vec<ScreenShake>,
    max_smooth_factor: f32,
    min_smooth_factor: f32,
    max_distance: f32,
    delta: f32,
    lead_factor: f32,
    shake_offset: Vec3f,
    #[skip]
    follow_point: Vec3f,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            screen_shake: Vec::new(),
            max_smooth_factor: 1.0,
            min_smooth_factor: 1.0,
            max_distance: 100.0,
            delta: 10.0,
            lead_factor: 1.0,
            follow_point: Vec3f::new(0., 0., 0.),
            shake_offset: Vec3f::zero(),
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

        camera.translation = self.translation();
    }

    pub fn push_screen_shake(&mut self, shake: ScreenShake) {
        info!("pushing shake: {shake:?}");
        self.screen_shake.push(shake);
    }

    fn apply_screen_shake(&mut self, dt: &DeltaTime) {
        for shake in self.screen_shake.iter_mut() {
            let elapsed = dt.wrapping_elapsed_as_seconds() - shake.start_time;
            if elapsed < shake.duration {
                let remaining = 1.0 - (elapsed / shake.duration);
                let current_intensity = shake.intensity * remaining;

                let mut rng = rand::thread_rng();
                self.shake_offset = Vec3f::new(
                    rng.gen_range(-1.0..1.0) * current_intensity,
                    rng.gen_range(-1.0..1.0) * current_intensity,
                    0.0, // typically, we don't shake on the z-axis
                );
            } else {
                // Shake finished
                self.shake_offset = Vec3f::new(0.0, 0.0, 0.0);
                shake.start_time = 0.0;
            }
        }

        self.screen_shake.retain(|s| s.start_time > 0.0);
    }

    fn translation(&self) -> Vec3f {
        self.follow_point + self.shake_offset
    }
}
