use camera::Camera;
use vector::Vec2f;
use winny::prelude::*;

#[derive(Debug)]
pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&mut self, app: &mut App) {
        app.insert_resource(MousePosition::default())
            .insert_resource(LastMousePos::default())
            .add_systems(Schedule::Update, mouse_position);
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Resource)]
struct LastMousePos(Vec2f);

#[derive(Debug, Default, Copy, Clone, PartialEq, Resource)]
pub struct MousePosition(pub Vec2f);

fn mouse_position(
    reader: EventReader<MouseMotion>,
    mut position: ResMut<MousePosition>,
    mut last_position: ResMut<LastMousePos>,
    window: Res<Window>,
    camera: Query<(Transform, Camera)>,
) {
    if let Some(event) = reader.peak_read().last() {
        last_position.0 = Vec2f::new(event.0 as f32, event.1 as f32);
    }

    let Ok((transform, _camera)) = camera.get_single() else {
        return;
    };

    let viewport = window.viewport;

    let size = window.winit_window.inner_size();
    let proportion = Vec2f::new(
        last_position.0.x / size.width as f32,
        last_position.0.y / size.height as f32,
    );
    let world = Vec2f::new(
        proportion.x * viewport.width() + transform.translation.x - viewport.width() / 2.,
        proportion.y * viewport.height() + transform.translation.y - viewport.height() / 2.,
    );

    position.0 = world;
}
