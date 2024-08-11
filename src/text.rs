use std::ops::Range;
use winny::{ecs::sets::IntoSystemStorage, math::vector::Vec2f, prelude::*};

use crate::{player::Player, should_run_game, Health};

#[derive(Debug)]
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<TypeWriter>()
            .register_timer::<TypeWriterTimeout>()
            .add_systems(
                Schedule::Update,
                (increment_type_writer, display_type_writer, display_health)
                    .run_if(should_run_game),
            );
    }
}

#[derive(Debug, Event, Default)]
struct TypeWriterTimeout;

#[derive(Resource)]
pub struct TypeWriter {
    string: String,
    speed: f32,
    position: Vec2f,
    slice_range: Range<usize>,
    last_len: usize,
    scale: f32,
    color: Modulation,
}

impl TypeWriter {
    pub fn new(string: String, speed: f32, position: Vec2f, scale: f32, color: Modulation) -> Self {
        Self {
            string,
            speed,
            scale,
            color,
            position,
            slice_range: 0..0,
            last_len: 0,
        }
    }

    pub fn start(&self, commands: &mut Commands) {
        commands.spawn(Timer::<TypeWriterTimeout>::new(
            self.speed,
            TypeWriterTimeout,
        ));
    }
}

fn increment_type_writer(
    mut commands: Commands,
    mut type_writer: ResMut<TypeWriter>,
    type_next_char: EventReader<TypeWriterTimeout>,
) {
    let mut recieved_event = false;
    for _ in type_next_char.read() {
        if type_writer.last_len == type_writer.string.len() {
            type_writer.last_len = 0;
            type_writer.slice_range = 0..0;
        } else {
            type_writer.last_len += 1;
            type_writer.slice_range = 0..type_writer.last_len;
        }
        recieved_event = true;
    }

    if recieved_event {
        commands.spawn(Timer::<TypeWriterTimeout>::new(
            type_writer.speed,
            TypeWriterTimeout,
        ));
    }
}

fn display_type_writer(
    context: Res<RenderContext>,
    mut text_renderer: ResMut<TextRenderer>,
    type_writer: Res<TypeWriter>,
) {
    use winny::gfx::wgpu_text::glyph_brush::*;

    text_renderer.draw(&context, || {
        let color: [f32; 4] = type_writer.color.into();
        let middle = Section::default()
            .add_text(
                Text::new(&type_writer.string[type_writer.slice_range.clone()])
                    .with_scale(type_writer.scale)
                    .with_color(color),
            )
            .with_bounds((
                context.config.width() as f32,
                context.config.height() as f32,
            ))
            .with_screen_position((type_writer.position.x, type_writer.position.y))
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
            );

        vec![middle]
    });
}

fn display_health(
    context: Res<RenderContext>,
    mut text_renderer: ResMut<TextRenderer>,
    player: Query<Health, With<Player>>,
) {
    use winny::gfx::wgpu_text::glyph_brush::*;
    let Ok(player_health) = player.get_single() else {
        return;
    };

    let mut string = String::new();
    string.push_str("[");
    let hit_points = (player_health.ratio() * 10.0) as usize;
    let hit_points = 5;
    for i in 0..10 {
        if hit_points > i {
            string.push_str(" * ");
        } else {
            string.push_str("   ");
        }
    }
    string.push_str("]");

    text_renderer.draw(&context, || {
        let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let middle = Section::default()
            .add_text(Text::new(&string).with_scale(30.).with_color(color))
            .with_bounds((
                context.config.width() as f32,
                context.config.height() as f32,
            ))
            .with_screen_position((context.config.width() as f32 / 2.0, 30.0))
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
            );

        vec![middle]
    });
}
