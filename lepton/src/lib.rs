pub mod shader;
pub mod model;
pub mod physics;
pub mod ui;
mod graphics;
mod control;
mod tools;
mod fps_limiter;

pub use graphics::*;
pub use control::*;
pub use winit::event::VirtualKeyCode;

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
    pub use crate::{Lepton, Graphics, Control, Pattern, KeyTracker, VirtualKeyCode, RenderData,
        physics::Physics,
        model::{Model, TextureType, VertexType},
        shader::{self, Camera, Lights, Object, builtin},
        ui::{UserInterface},
    };
}

/// A user-end trait which enables rendering and response to key presses
pub trait Lepton: 'static {
    /// Respond to a key press. Returns true if the program is to exit.
    fn keydown(&mut self, _keycode: winit::event::VirtualKeyCode) -> bool {false}

    /// Respond to a key release. Returns true if the program is to exit.
    fn keyup(&mut self, _keycode: winit::event::VirtualKeyCode) -> bool {false}

    /// Respond to mouse motion. True if the mouse pointer is to be reset to the center.
    fn mouse_motion(&mut self, _delta: (f64, f64)) -> bool {false}

    /// Execute all the patterns
    fn render(&mut self, render_data: &mut RenderData);

    /// Update all the objects
    fn update(&mut self, delta_time: f32) {}

    fn check_reload(&mut self, graphics: &Graphics);
}

