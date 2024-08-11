use winny::{
    asset::server::AssetServer, gfx::render_pipeline::buffer::AsGpuBuffer, math::vector::Vec4f,
    prelude::*,
};

#[derive(Component, Debug, Clone)]
pub struct NuclearAtom {
    pub modulation: Modulation,
    pub texture: Handle<Image>,
}

impl Asset for NuclearAtom {}

impl Material for NuclearAtom {
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

    fn update(&self, context: &RenderContext, binding: &BindGroup) {
        RawNuclearAtom::write_buffer(context, binding.single_buffer(), &[self.as_raw()]);
    }
}

impl AsWgpuResources for NuclearAtom {
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
        let uniform_resources = <&[RawNuclearAtom] as AsWgpuResources>::as_wgpu_resources(
            &[self.as_raw()],
            context,
            label,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some(BufferType::Init),
        );

        vec![texture_resources, uniform_resources]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl AsBindGroup for NuclearAtom {
    const LABEL: &'static str = "nuclear material";
    const BINDING_TYPES: &'static [wgpu::BindingType] =
        &[DEFAULT_TEXTURE_BINDING, DEFAULT_SAMPLER_BINDING, UNIFORM];
    const VISIBILITY: &'static [wgpu::ShaderStages] = &[wgpu::ShaderStages::FRAGMENT; 3];
}

impl NuclearAtom {
    pub(crate) fn as_raw(&self) -> RawNuclearAtom {
        RawNuclearAtom {
            modulation: self.modulation.clamp(),
        }
    }
}

/// Uniform of [`NuclearAtom`].
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawNuclearAtom {
    modulation: Vec4f,
}

unsafe impl AsGpuBuffer for RawNuclearAtom {}
