use super::BloomTexture;
use std::marker::PhantomData;
use winny::{asset::server::AssetServer, prelude::*};

pub fn clear_frame_buf(
    view: Res<RenderView>,
    mut encoder: ResMut<RenderEncoder>,
    clear_color: Res<ClearColor>,
) {
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

pub fn set_bloom_buf(bloom: Res<BloomTexture>, mut view: ResMut<RenderView>) {
    *view = RenderView::new(bloom.0.create_view());
}

pub fn build_post_processing_pipeline<T: 'static + Sync + Send>(
    shader: &'static str,
    commands: &mut Commands,
    context: &RenderContext,
) {
    let frag_shader = FragmentShader({
        let shader = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        };
        context.device.create_shader_module(shader)
    });

    commands.insert_resource(PostProcessingPipeline::<T>::new(
        "post",
        &context,
        frag_shader,
    ));
}

pub fn build_post_processing_pipeline_with_texture<T: 'static + Sync + Send>(
    shader: &'static str,
    commands: &mut Commands,
    context: &RenderContext,
    texture: &Texture,
    sampler_type: SamplerFilterType,
) {
    let frag_shader = FragmentShader({
        let shader = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        };
        context.device.create_shader_module(shader)
    });

    commands.insert_resource(PostProcessingPipeline::<T>::new_with_texture(
        "post",
        &context,
        frag_shader,
        texture,
        sampler_type,
    ));
}

pub fn build_post_processing_pipeline_with_binding<T: 'static + Sync + Send>(
    shader: &'static str,
    commands: &mut Commands,
    context: &RenderContext,
    binding: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
) {
    let frag_shader = FragmentShader({
        let shader = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        };
        context.device.create_shader_module(shader)
    });

    commands.insert_resource(PostProcessingPipeline::<T>::new_with_binding(
        "post",
        &context,
        frag_shader,
        binding,
        layout,
    ));
}

#[derive(Resource)]
pub struct PostProcessingPipeline<T: 'static + Sync + Send> {
    pipeline: RenderPipeline2d,
    binding: Option<wgpu::BindGroup>,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Sync + Send> PostProcessingPipeline<T> {
    pub fn new_with_binding(
        label: &str,
        context: &RenderContext,
        frag_shader: FragmentShader,
        binding: wgpu::BindGroup,
        layout: wgpu::BindGroupLayout,
    ) -> Self {
        let vert_shader = VertexShader({
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../winny/res/shaders/post_processing_vert.wgsl").into(),
                ),
            };
            context.device.create_shader_module(shader)
        });

        let pipeline = RenderPipeline2d::new(
            label,
            context,
            &[&layout],
            &[],
            &vert_shader,
            &frag_shader,
            wgpu::BlendState::ALPHA_BLENDING,
            None,
        );

        Self {
            pipeline,
            binding: Some(binding),
            _phantom: PhantomData,
        }
    }

    pub fn new_with_texture(
        label: &str,
        context: &RenderContext,
        frag_shader: FragmentShader,
        texture: &Texture,
        sampler_type: SamplerFilterType,
    ) -> Self {
        let vert_shader = VertexShader({
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../winny/res/shaders/post_processing_vert.wgsl").into(),
                ),
            };
            context.device.create_shader_module(shader)
        });

        let texture = FragTexture::as_entire_binding(context, FragTexture(texture), sampler_type);

        let pipeline = RenderPipeline2d::new(
            label,
            context,
            &[&<FragTexture as AsBindGroup>::layout(context)],
            &[],
            &vert_shader,
            &frag_shader,
            wgpu::BlendState::ALPHA_BLENDING,
            None,
        );

        Self {
            pipeline,
            binding: Some(texture.into_bind_group()),
            _phantom: PhantomData,
        }
    }

    pub fn new(label: &str, context: &RenderContext, frag_shader: FragmentShader) -> Self {
        let vert_shader = VertexShader({
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../winny/res/shaders/post_processing_vert.wgsl").into(),
                ),
            };
            context.device.create_shader_module(shader)
        });

        let pipeline = RenderPipeline2d::new(
            label,
            context,
            &[],
            &[],
            &vert_shader,
            &frag_shader,
            wgpu::BlendState::ALPHA_BLENDING,
            None,
        );

        Self {
            pipeline,
            binding: None,
            _phantom: PhantomData,
        }
    }
}

pub fn render_pass<T: 'static + Sync + Send>(
    mut encoder: ResMut<RenderEncoder>,
    pipeline: Res<PostProcessingPipeline<T>>,
    view: Res<RenderView>,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("post processing"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    if let Some(binding) = &pipeline.binding {
        render_pass.set_bind_group(0, binding, &[])
    }
    render_pass.set_pipeline(&pipeline.pipeline.0);
    render_pass.draw(0..3, 0..1);
}

pub fn bloom_binding(
    context: &RenderContext,
    original: &wgpu::TextureView,
    blur: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = context
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

    let binding = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(original),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(blur),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

    (layout, binding)
}
