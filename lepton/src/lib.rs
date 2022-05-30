pub mod shader;
pub mod model;
mod pattern;
mod graphics;
mod control;
mod fps_limiter;

pub use pattern::*;
pub use graphics::*;
pub use control::*;

pub type VirtualKeyCode = winit::event::VirtualKeyCode;
pub type ElementState = winit::event::ElementState;


/// A module to contain all of the constants which are set within the crate.
mod constants {
    use ash::vk::{ClearValue, ClearColorValue, ClearDepthStencilValue};

    pub(crate) const APPLICATION_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    pub(crate) const ENGINE_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    pub(crate) const API_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 92);

    pub(crate) const CLEAR_VALUES: [ClearValue; 2] = [
        ClearValue { color: ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] } },
        ClearValue { depth_stencil: ClearDepthStencilValue { depth: 1.0, stencil: 0, } },
    ];
}

pub(crate) trait Unload {
    fn unload(&mut self, device: &ash::Device); 
}