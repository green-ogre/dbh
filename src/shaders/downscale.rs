use std::ops::DerefMut;
use winny::prelude::*;

/// Downscales image.
#[derive(Resource)]
pub struct Pixler {
    copy_pipeline: RenderPipeline2d,
    frame_buf: BindGroup,
    downsampled: BindGroup,
}

const DOWNSCALE_FACTOR: u32 = 4;

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
            Dimensions::new(d.width() / DOWNSCALE_FACTOR, d.height() / DOWNSCALE_FACTOR),
            context,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            context.config.format(),
        );
        let downsampled =
            FragTexture::as_entire_binding(context, FragTexture(&tex), SamplerFilterType::Nearest);

        let vert_shader = wgpu::ShaderModuleDescriptor {
            label: Some("particles vert"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../winny/res/shaders/copy_texture.wgsl").into(),
            ),
        };
        let vert_shader = VertexShader(context.device.create_shader_module(vert_shader));

        let frag_shader = wgpu::ShaderModuleDescriptor {
            label: Some("particles vert"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../winny/res/shaders/copy_texture.wgsl").into(),
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

pub fn set_frame_buf(
    mut pixler: ResMut<Pixler>,
    mut encoder: ResMut<RenderEncoder>,
    mut view: ResMut<RenderView>,
    clear_color: Res<ClearColor>,
) {
    *view = RenderView::new(pixler.frame_buf.take_texture_view());
    {
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("downscale pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color.0.wgpu_color()),
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

pub fn pixler_render_pass(
    mut encoder: ResMut<RenderEncoder>,
    mut pixler: ResMut<Pixler>,
    output: Res<RenderOutput>,
    mut view: ResMut<RenderView>,
    clear_color: Res<ClearColor>,
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
                    load: wgpu::LoadOp::Clear(clear_color.0.wgpu_color()),
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
                    load: wgpu::LoadOp::Clear(clear_color.0.wgpu_color()),
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
