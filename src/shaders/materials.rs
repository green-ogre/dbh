use winny::{
    asset::server::AssetServer, gfx::render_pipeline::buffer::AsGpuBuffer, math::vector::Vec4f,
    prelude::*,
};

macro_rules! impl_material {
    ($ty:ident, $raw_ty:ident) => {
        #[derive(Component, AsEgui, Debug, Clone)]
        pub struct $ty {
            pub modulation: Modulation,
        }

        impl Material for $ty {
            const BLEND_STATE: wgpu::BlendState = wgpu::BlendState::ALPHA_BLENDING;

            fn resource_state<'s, 'w>(
                &'s self,
                _textures: &'w mut RenderAssets<Texture>,
                _images: &Assets<Image>,
                _context: &Res<RenderContext>,
            ) -> Option<<Self as AsWgpuResources>::State<'w>> {
                Some(())
            }

            fn mesh_2d_fragment_shader(
                &self,
                server: &AssetServer,
            ) -> Handle<FragmentShaderSource> {
                server.load("res/shaders/nuclear.wgsl")
            }

            fn update(&self, context: &RenderContext, binding: &wgpu::Buffer) {
                context
                    .queue
                    .write_buffer(binding, 0, bytemuck::cast_slice(&[self.as_raw()]));
            }
        }

        impl AsWgpuResources for $ty {
            type State<'s> = ();

            fn as_wgpu_resources<'s>(
                self,
                context: &RenderContext,
                label: &'static str,
                _state: Self::State<'s>,
                _buffer_type: Option<BufferType>,
            ) -> Vec<WgpuResource> {
                let uniform_resources = <&[$raw_ty] as AsWgpuResources>::as_wgpu_resources(
                    &[self.as_raw()],
                    context,
                    label,
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    Some(BufferType::Init),
                );

                uniform_resources.into_iter().collect()
            }
        }

        impl AsBindGroup for $ty {
            const LABEL: &'static str = "macro material";
            const BINDING_TYPES: &'static [wgpu::BindingType] =
                &[winny::gfx::render_pipeline::bind_group::UNIFORM];
            const VISIBILITY: &'static [wgpu::ShaderStages] = &[wgpu::ShaderStages::FRAGMENT];
        }

        impl $ty {
            pub(crate) fn as_raw(&self) -> $raw_ty {
                $raw_ty {
                    modulation: self.modulation.clamp(),
                }
            }
        }

        #[repr(C)]
        #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $raw_ty {
            modulation: Vec4f,
        }

        unsafe impl AsGpuBuffer for $raw_ty {}
    };
}

impl_material!(NonagonMaterial, RawNonagonMaterial);
impl_material!(OctagonMaterial, RawOctagonMaterial);
impl_material!(HeptaMaterial, RawHeptaMaterial);
impl_material!(HexaMaterial, RawHexaMaterial);
impl_material!(PentagonMaterial, RawPentagonMaterial);
impl_material!(QuadrilateralMaterial, RawQuadrilateralMaterial);
impl_material!(TriangleMaterial, RawTriangleMaterial);
impl_material!(PlayerMaterial, RawPlayerMaterial);
impl_material!(NeutronMaterial, RawNeutronMaterial);
