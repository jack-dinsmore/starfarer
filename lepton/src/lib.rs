#![allow(dead_code)]
pub mod shader;
pub mod model;
pub mod physics;
pub mod ui;
pub mod tools;
pub mod backend;
mod graphics;

pub use backend::{InputReceiver, Renderer};
pub(crate) use graphics::*;

/// A module to contain all of the constants which are set within the crate.
mod constants {
    use ash::vk::{ClearValue, ClearColorValue, ClearDepthStencilValue};

    pub(crate) const APPLICATION_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    pub(crate) const ENGINE_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    pub(crate) const API_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 92);

    pub(crate) const CLEAR_VALUES: [ClearValue; 2] = [
        ClearValue { color: ClearColorValue { float32: [0.0, 0.1, 0.2, 1.0] } },
        ClearValue { depth_stencil: ClearDepthStencilValue { depth: 1.0, stencil: 0, } },
    ];
    pub(crate) const PI: f64 = 3.141592653589793238462643383;
}

pub mod prelude {
    pub use crate::{Renderer, InputReceiver,
        graphics::{Graphics, Pattern, RenderData, Action},
        backend::{Backend, KeyTracker, VirtualKeyCode, MouseButton},
        physics::{Physics, Object},
        model::{Model, TextureType, VertexType, vertex},
        shader::{InputType, Shader, Camera, Lights, LightFeatures, builtin},
        ui::{UserInterface, Element, ElementData, Font},
    };
}