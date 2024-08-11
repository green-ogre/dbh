use std::io::{BufReader, Cursor};

use crate::{
    player::Player, shaders::post_processing::texture_with_uniform_binding, should_run_game, Health,
};

use self::{
    downscale::Pixler,
    materials::{
        HeptaMaterial, HexaMaterial, NeutronMaterial, NonagonMaterial, OctagonMaterial,
        PentagonMaterial, PlayerMaterial, QuadrilateralMaterial, TriangleMaterial,
    },
    post_processing::{
        bloom_binding, build_post_processing_pipeline_with_binding,
        build_post_processing_pipeline_with_texture, PostProcessingPipeline,
    },
};
use winny::{ecs::sets::IntoSystemStorage, math::vector::Vec4f, prelude::*};

pub mod downscale;
pub mod materials;
pub mod post_processing;

#[derive(Debug)]
pub struct ShaderArtPlugin;

impl Plugin for ShaderArtPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<Pixler>()
            .add_plugins(MaterialPlugin::<NonagonMaterial>::new())
            .add_plugins(MaterialPlugin::<OctagonMaterial>::new())
            .add_plugins(MaterialPlugin::<HeptaMaterial>::new())
            .add_plugins(MaterialPlugin::<HexaMaterial>::new())
            .add_plugins(MaterialPlugin::<PentagonMaterial>::new())
            .add_plugins(MaterialPlugin::<QuadrilateralMaterial>::new())
            .add_plugins(MaterialPlugin::<TriangleMaterial>::new())
            .add_plugins(MaterialPlugin::<PlayerMaterial>::new())
            .add_plugins(MaterialPlugin::<NeutronMaterial>::new())
            .register_resource::<PostProcessingPipeline<BrightnessThreshold>>()
            .register_resource::<PostProcessingPipeline<GaussianBlurH>>()
            .register_resource::<PostProcessingPipeline<GaussianBlurV>>()
            .register_resource::<PostProcessingPipeline<Bloom>>()
            .register_resource::<PostProcessingPipeline<Background>>()
            .register_resource::<BackgroundBuffer>()
            .register_resource::<BloomTexture>()
            .add_systems(Schedule::StartUp, startup)
            .add_systems(
                AppSchedule::PreRender,
                (
                    // update_player_health_buffer,
                    downscale::set_frame_buf,
                    post_processing::clear_frame_buf,
                )
                    .run_if(should_run_game),
            )
            .add_systems(
                AppSchedule::PreRender,
                (
                    update_background_uniform,
                    post_processing::render_pass::<Background>,
                ),
            )
            .add_systems(
                AppSchedule::PostRender,
                (
                    downscale::buf_to_downsample,
                    downscale::set_frame_buf,
                    downscale::downsample_to_view,
                    post_processing::set_bloom_buf,
                    post_processing::clear_frame_buf,
                    post_processing::render_pass::<BrightnessThreshold>,
                    post_processing::render_pass::<GaussianBlurV>,
                    post_processing::render_pass::<GaussianBlurH>,
                    downscale::reset_output_view,
                    post_processing::render_pass::<Bloom>,
                    // post_processing::render_pass::<PlayerHealth>,
                )
                    .run_if(should_run_game),
            );
    }
}

struct Background;
#[derive(Resource)]
pub struct BackgroundTexture(Texture);
#[derive(Resource)]
pub struct BackgroundBuffer(wgpu::Buffer);
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BackGroundUniform {
    time: f32,
    scroll_speed: f32,
}
const BACKGROUND_SCROLL_SPEED: f32 = 0.2;

fn update_background_uniform(
    context: Res<RenderContext>,
    buffer: Res<BackgroundBuffer>,
    dt: Res<DeltaTime>,
) {
    context.queue.write_buffer(
        &buffer.0,
        0,
        bytemuck::cast_slice(&[BackGroundUniform {
            time: dt.wrapping_elapsed_as_seconds(),
            scroll_speed: BACKGROUND_SCROLL_SPEED,
        }]),
    );
}

// fn update_player_health_buffer(
//     context: Res<RenderContext>,
//     buffer: Res<PlayerHealthBuffer>,
//     player: Query<Health, With<Player>>,
// ) {
//     let Ok(health) = player.get_single() else {
//         return;
//     };
//
//     context.queue.write_buffer(
//         &buffer.0,
//         0,
//         bytemuck::cast_slice(&[PlayerHealthUniform {
//             health: health.ratio(),
//         }]),
//     );
// }

struct BrightnessThreshold;
struct GaussianBlurH;
struct GaussianBlurV;
struct Bloom;
#[derive(Resource)]
pub struct BloomTexture(Texture);

fn startup(mut commands: Commands, context: Res<RenderContext>) {
    let pixler = Pixler::new(&context);

    let bytes = include_bytes!("../../res/textures/nuclear_background.png");
    let reader = ByteReader::new(BufReader::new(Cursor::new(bytes.to_vec())));
    let img = Image::new(reader, ImageSettings::default()).unwrap();
    let background_texture = Texture::from_image(&context.device, &context.queue, &img);

    use wgpu::util::DeviceExt;
    let buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[BackGroundUniform {
                time: 0.0,
                scroll_speed: BACKGROUND_SCROLL_SPEED,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let (layout, binding) = texture_with_uniform_binding(
        &context,
        &background_texture.create_view(),
        &Texture::create_sampler(&context, &SamplerFilterType::Linear),
        &buffer,
    );

    build_post_processing_pipeline_with_binding::<Background>(
        include_str!("../../res/shaders/background.wgsl"),
        &mut commands,
        &context,
        binding,
        layout,
    );

    let bloom_texture = Texture::empty(
        context.config.dimensions,
        &context,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        context.config.format,
    );
    build_post_processing_pipeline_with_texture::<BrightnessThreshold>(
        include_str!("../../res/shaders/brightness_threshold.wgsl"),
        &mut commands,
        &context,
        &pixler.frame_tex,
        SamplerFilterType::Linear,
    );
    build_post_processing_pipeline_with_texture::<GaussianBlurV>(
        include_str!("../../res/shaders/blur_vertical.wgsl"),
        &mut commands,
        &context,
        &pixler.frame_tex,
        SamplerFilterType::Linear,
    );
    build_post_processing_pipeline_with_texture::<GaussianBlurH>(
        include_str!("../../res/shaders/blur_horizontal.wgsl"),
        &mut commands,
        &context,
        &pixler.frame_tex,
        SamplerFilterType::Linear,
    );
    let (layout, binding) = bloom_binding(
        &context,
        &pixler.frame_buf.single_texture_view(),
        &bloom_texture.create_view(),
        &Texture::create_sampler(&context, &SamplerFilterType::Linear),
    );
    build_post_processing_pipeline_with_binding::<Bloom>(
        include_str!("../../res/shaders/bloom.wgsl"),
        &mut commands,
        &context,
        binding,
        layout,
    );

    commands
        .insert_resource(BloomTexture(bloom_texture))
        .insert_resource(BackgroundTexture(background_texture))
        .insert_resource(BackgroundBuffer(buffer))
        .insert_resource(pixler);
}

pub trait ColorPalette {
    const BLUE: Vec4f = Vec4f::new(0.122, 0.141, 0.294, 1.0);
    const PURPLE: Vec4f = Vec4f::new(0.396, 0.251, 0.325, 1.0);
    const BROWN: Vec4f = Vec4f::new(0.659, 0.376, 0.365, 1.0);
    const PALE_ORANGE: Vec4f = Vec4f::new(0.82, 0.651, 0.494, 1.0);
    const YELLOW: Vec4f = Vec4f::new(0.965, 0.906, 0.612, 1.0);
    const PALE_GREEN: Vec4f = Vec4f::new(0.714, 0.812, 0.557, 1.0);
    const GREEN: Vec4f = Vec4f::new(0.376, 0.682, 0.482, 1.0);
    const BLUE_GREEN: Vec4f = Vec4f::new(0.235, 0.42, 0.392, 1.0);
}

pub struct Paper8;

impl ColorPalette for Paper8 {}

pub struct SpaceHaze;

impl SpaceHaze {
    pub fn dark_blue() -> Vec4f {
        Vec4f::new(11.0 / 255.0, 6.0 / 255.0, 48.0 / 255.0, 1.0)
    }

    pub fn white() -> Vec4f {
        Vec4f::new(248.0 / 255.0, 227.0 / 255.0, 196.0 / 255.0, 1.0)
    }

    pub fn purple() -> Vec4f {
        Vec4f::new(106.0 / 255.0, 31.0 / 255.0, 177.0 / 255.0, 1.0)
    }

    pub fn pink() -> Vec4f {
        Vec4f::new(204.0 / 255.0, 52.0 / 255.0, 149.0 / 255.0, 1.0)
    }
}

fn hex_to_vec4(hex: u32) -> Vec4f {
    Vec4f::new(
        ((hex >> 16) & 0xFF) as f32,
        ((hex >> 8) & 0xFF) as f32,
        (hex & 0xFF) as f32,
        1.0,
    )
}

pub struct Crimson;

impl Crimson {
    pub fn color(index: usize) -> Vec4f {
        match index.clamp(0, 9) {
            0 => hex_to_vec4(0xff0546),
            1 => hex_to_vec4(0x9c173b),
            2 => hex_to_vec4(0x660f31),
            3 => hex_to_vec4(0x450327),
            4 => hex_to_vec4(0x270022),
            5 => hex_to_vec4(0x17001d),
            6 => hex_to_vec4(0x09010d),
            7 => hex_to_vec4(0x0ce6f2),
            8 => hex_to_vec4(0x0098db),
            9 => hex_to_vec4(0x1e579c),
            _ => unreachable!(),
        }
    }
}
