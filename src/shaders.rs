use std::ops::DerefMut;

use winny::{
    asset::server::AssetServer, gfx::render_pipeline::buffer::AsGpuBuffer, math::vector::Vec4f,
    prelude::*,
};

#[derive(Debug)]
pub struct ShaderArtPlugin;

impl Plugin for ShaderArtPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<Pixler>()
            .add_plugins(MaterialPlugin::<Nuclear>::new())
            .add_systems(Schedule::StartUp, startup)
            .add_systems(AppSchedule::PreRender, set_frame_buf)
            .add_systems(AppSchedule::PostRender, pixler_render_pass);
    }
}

fn startup(mut commands: Commands, context: Res<RenderContext>) {
    commands.insert_resource(Pixler::new(&context));
}

fn set_frame_buf(
    mut pixler: ResMut<Pixler>,
    mut encoder: ResMut<RenderEncoder>,
    mut view: ResMut<RenderView>,
) {
    *view = RenderView::new(pixler.frame_buf.take_texture_view());
    {
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("downscale pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }
}

/// Applies standard color properties, then downscales image
#[derive(Resource)]
pub struct Pixler {
    copy_pipeline: RenderPipeline2d,
    frame_buf: BindGroup,
    downsampled: BindGroup,
}

impl Pixler {
    pub fn new(context: &RenderContext) -> Self {
        let tex = Texture::empty(
            context.config.dimensions,
            context,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            context.config.format(),
        );
        let frame_buf =
            FragTexture::as_entire_binding(context, FragTexture(&tex), SamplerFilterType::Nearest);
        let d = context.config.dimensions;
        let tex = Texture::empty(
            Dimensions::new(d.width() / 2, d.height() / 2),
            context,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            context.config.format(),
        );
        let downsampled =
            FragTexture::as_entire_binding(context, FragTexture(&tex), SamplerFilterType::Nearest);

        let vert_shader = wgpu::ShaderModuleDescriptor {
            label: Some("particles vert"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../winny/res/shaders/copy_texture.wgsl").into(),
            ),
        };
        let vert_shader = VertexShader(context.device.create_shader_module(vert_shader));

        let frag_shader = wgpu::ShaderModuleDescriptor {
            label: Some("particles vert"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../winny/res/shaders/copy_texture.wgsl").into(),
            ),
        };
        let frag_shader = FragmentShader(context.device.create_shader_module(frag_shader));

        let copy_pipeline = RenderPipeline2d::new(
            "copy pixler",
            context,
            &[frame_buf.layout()],
            &[],
            &vert_shader,
            &frag_shader,
            wgpu::BlendState::ALPHA_BLENDING,
            None,
        );

        Self {
            copy_pipeline,
            frame_buf,
            downsampled,
        }
    }
}

fn pixler_render_pass(
    mut encoder: ResMut<RenderEncoder>,
    mut pixler: ResMut<Pixler>,
    output: Res<RenderOutput>,
    mut view: ResMut<RenderView>,
) {
    let mut out_view = RenderView::new(
        output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    std::mem::swap(view.deref_mut(), &mut out_view);
    pixler.frame_buf.insert_texture_view(out_view);

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("downscale pixler"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &pixler.downsampled.single_texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pixler.copy_pipeline.0);
        render_pass.set_bind_group(0, &pixler.frame_buf.binding(), &[]);
        render_pass.draw(0..3, 0..1);
    }

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("copy pixler"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pixler.copy_pipeline.0);
        render_pass.set_bind_group(0, &pixler.downsampled.binding(), &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Component, Debug, Clone)]
pub struct Nuclear {
    pub opacity: Opacity,
    pub saturation: Saturation,
    pub modulation: Modulation,
    pub texture: Handle<Image>,
}

impl Material for Nuclear {
    const BLEND_STATE: wgpu::BlendState = wgpu::BlendState::ALPHA_BLENDING;

    fn resource_state<'s, 'w>(
        &'s self,
        textures: &'w mut RenderAssets<Texture>,
        images: &Assets<Image>,
        context: &Res<RenderContext>,
    ) -> Option<<Self as AsWgpuResources>::State<'w>> {
        if let Some(image) = images.get(&self.texture) {
            Some(
                textures
                    .entry(self.texture.clone())
                    .or_insert_with(|| Texture::prepare_asset(image, &context)),
            )
        } else {
            None
        }
    }

    fn mesh_2d_fragment_shader(&self, server: &AssetServer) -> Handle<FragmentShaderSource> {
        server.load("res/shaders/nuclear.wgsl")
    }
}

impl AsWgpuResources for Nuclear {
    type State<'s> = &'s Texture;

    fn as_wgpu_resources<'s>(
        self,
        context: &RenderContext,
        label: &'static str,
        state: Self::State<'s>,
        _buffer_type: Option<BufferType>,
    ) -> Vec<WgpuResource> {
        let texture_resources =
            state.as_wgpu_resources(context, label, SamplerFilterType::Nearest, None);
        let uniform_resources = <&[RawNuclear] as AsWgpuResources>::as_wgpu_resources(
            &[self.as_raw()],
            context,
            label,
            wgpu::BufferUsages::UNIFORM,
            Some(BufferType::Init),
        );

        vec![texture_resources, uniform_resources]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl AsBindGroup for Nuclear {
    const LABEL: &'static str = "nuclear material";
    const BINDING_TYPES: &'static [wgpu::BindingType] =
        &[DEFAULT_TEXTURE_BINDING, DEFAULT_SAMPLER_BINDING, UNIFORM];
    const VISIBILITY: &'static [wgpu::ShaderStages] = &[wgpu::ShaderStages::FRAGMENT; 3];
}

impl Nuclear {
    pub(crate) fn as_raw(&self) -> RawNuclear {
        RawNuclear {
            modulation: self.modulation.clamp(),
            opacity: self.opacity.clamp(),
            saturation: self.saturation.clamp(),
        }
    }
}

/// Uniform of [`Nuclear`].
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawNuclear {
    modulation: Vec4f,
    opacity: f32,
    saturation: f32,
}

unsafe impl AsGpuBuffer for RawNuclear {}
