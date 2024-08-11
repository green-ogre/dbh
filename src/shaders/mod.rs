use self::{atoms::NuclearAtom, downscale::Pixler, neutrons::NuclearNeutron, player::Nuclear};
use winny::{math::vector::Vec4f, prelude::*};

pub mod atoms;
pub mod downscale;
pub mod neutrons;
pub mod player;

#[derive(Debug)]
pub struct ShaderArtPlugin;

impl Plugin for ShaderArtPlugin {
    fn build(&mut self, app: &mut App) {
        app.register_resource::<Pixler>()
            .add_plugins(MaterialPlugin::<Nuclear>::new())
            .add_plugins(MaterialPlugin::<NuclearAtom>::new())
            .add_plugins(MaterialPlugin::<NuclearNeutron>::new())
            .add_systems(Schedule::StartUp, startup)
            .add_systems(AppSchedule::PreRender, downscale::set_frame_buf)
            .add_systems(AppSchedule::PostRender, downscale::pixler_render_pass);
    }
}

fn startup(mut commands: Commands, context: Res<RenderContext>) {
    commands.insert_resource(Pixler::new(&context));
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
