use crate::{Graphics, RenderData};

/// A user-end trait which enables rendering and response to key presses
pub trait Renderer: 'static {
    /// Render the scene by updating all inputs and rendering all patterns. The update function should
    /// be preferred over the render function for all non-rendering code because render happens during 
    /// GPU idle time and update can be called at any time from any thread.
    fn render(&mut self, graphics: &Graphics, render_data: &mut RenderData);

    /// Update all the objects.
    fn update(&mut self, _delta_time: f32) {}

    /// Called only on window resize. Record any static patterns again. There is no need to 
    /// record patterns that are recorded every frame, hence why this function is initially empty.
    fn resize(&mut self, _graphics: &Graphics) {}

    /// Retuns true if the application should quit now.
    fn should_quit(&self) -> bool {false}
}