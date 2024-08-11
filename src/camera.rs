use std::f32::consts::TAU;

use crate::{
    loader::LoaderApp,
    player::{DirectionalVelocity, Player},
};
use noise::NoiseFn;
use rand::{Rng, SeedableRng};
use vector::Vec2f;
use winny::{gfx::camera::Camera, math::vector::Vec3f, prelude::*};

#[derive(Debug)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<PlayerCamera>()
            .save_load_resource::<PlayerCamera>()
            .egui_resource::<PlayerCamera>()
            // .egui_resource::<ScreenShake>()
            // .insert_resource(ScreenShake {
            //     intensity: 42.0,
            //     duration: 0.5,
            //     start_time: 0.0,
            // })
            // .add_systems(Schedule::Update, shake_screen)
            .add_systems(Schedule::PostUpdate, update_camera);
    }
}

#[derive(Resource, AsEgui, Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct ScreenShake {
    intensity: f32,
    duration: f32,
    start_time: f32,
    x_offset: Vec2f,
    y_offset: Vec2f,
}

impl ScreenShake {
    pub fn new(intensity: f32, duration: f32, start_time: f32) -> Self {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let mut offset = move || {
            Vec2f::new(
                rng.gen_range(-1000f32..1000f32),
                rng.gen_range(-1000f32..1000f32),
            )
        };

        Self {
            intensity,
            duration,
            start_time,
            x_offset: offset(),
            y_offset: offset(),
        }
    }
}

// fn shake_screen(
//     reader: EventReader<KeyInput>,
//     mut camera: ResMut<PlayerCamera>,
//     shake: Res<ScreenShake>,
//     delta: Res<DeltaTime>,
// ) {
//     for event in reader.peak_read() {
//         if matches!(
//             event,
//             KeyInput {
//                 code: KeyCode::I,
//                 state: KeyState::Pressed,
//                 ..
//             }
//         ) {
//             let mut shake = *shake;
//             shake.start_time = delta.wrapping_elapsed_as_seconds();
//             camera.push_screen_shake(shake)
//         }
//     }
// }

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
    #[skip]
    noise: Noise,
}

#[derive(Debug)]
pub struct Noise(noise::OpenSimplex);

impl egui_widget::Widget for Noise {
    fn display(&mut self, ui: &mut egui::Ui) {}
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
            noise: Noise(noise::OpenSimplex::new(1)),
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
        self.screen_shake.push(shake);
    }

    fn apply_screen_shake(&mut self, dt: &DeltaTime) {
        for shake in self.screen_shake.iter_mut() {
            let elapsed = dt.wrapping_elapsed_as_seconds() - shake.start_time;
            if elapsed < shake.duration {
                let remaining = 1.0 - (elapsed / shake.duration);
                let current_intensity = shake.intensity * remaining;

                // Sample the simplex space in a circle
                let radius = 5.;

                let x_noise = self.noise.0.get([
                    (shake.x_offset.x + (remaining * TAU).cos() * radius) as f64,
                    (shake.x_offset.y + (remaining * TAU).sin() * radius) as f64,
                ]);

                let y_noise = self.noise.0.get([
                    (shake.y_offset.x + (remaining * TAU).cos() * radius) as f64,
                    (shake.y_offset.y + (remaining * TAU).sin() * radius) as f64,
                ]);

                self.shake_offset = Vec3f::new(
                    x_noise as f32 * current_intensity,
                    y_noise as f32 * current_intensity,
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
