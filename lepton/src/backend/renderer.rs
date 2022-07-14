use std::rc::Rc;
use std::collections::HashMap;

use crate::Graphics;
use crate::model::Model;
use crate::physics::{Object, RigidBody};
use crate::shader::ShaderTrait;
use crate::ui::UserInterfaceTrait;

pub enum RenderTask<'a> {
    DrawObject(Object),
    LoadShader(&'a dyn ShaderTrait),
    DrawModel(&'a Rc<Model>),
    DrawUI(&'a dyn UserInterfaceTrait),
    ClearDepthBuffer,
}

/// A user-end trait which enables rendering and response to key presses
pub trait Renderer: 'static {
    /// Render the scene by updating all inputs and returning the render tasks to be performed
    /// by the backend. This function is only executed during GPU idle time, unlike the update
    /// function which is executed any time. Therefore, this render function should be reserved
    /// only for operations that must be performed when the GPU is idle.
    fn render(&mut self, graphics: &Graphics, buffer_index: usize) -> Vec<RenderTask>;

    /// Update all the objects. All game logic should be performed in this function call, with
    /// the render function reserved for tasks that can only be accomplished during GPU idle
    /// time.
    fn update(&mut self, _delta_time: f32) {}

    /// Called only on window resize.
    fn resize(&mut self, _graphics: &Graphics) {}

    /// Retuns true if the application should quit now.
    fn should_quit(&self) -> bool {false}

    /// Load all the models that will be used throughout the game and return them, paired with
    /// their objects.
    fn load_models(&mut self, graphics: &Graphics) -> HashMap<Object, Vec<Rc<Model>>>;

    /// Load all the rigid bodies for objects and return them with, paired with their objects.
    fn load_rigid_bodies(&mut self) -> HashMap<Object, RigidBody>;

    /// Do whatever is appropriate to prepare the struct before the render loops start.
    fn prepare(&mut self, _graphics: &Graphics) {}
}