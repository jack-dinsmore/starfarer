#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::new_without_default)]
#![allow(clippy::unused_io_amount)]
pub mod shader;
pub mod model;
pub mod physics;
pub mod ui;
pub mod tools;
pub mod backend;
pub mod input;
mod graphics;

pub use backend::{InputReceiver, Renderer};
pub(crate) use graphics::{Graphics, GraphicsData, get_device};

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
}

pub mod prelude {
    pub use crate::{Renderer, InputReceiver,
        graphics::{Graphics},
        backend::{Backend, RenderTask, KeyTracker, VirtualKeyCode, MouseButton},
        physics::{Object, ObjectManager, RigidBody, PhysicsTask, Collider},
        model::{Model, DrawState},
        shader::{self, Shader, builtin, vertex},
        input::{InputType, Input, TextureType, VertexType},
        ui::{UserInterface, Element, ElementData, Font, color},
        tools,
    };
}