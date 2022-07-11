pub mod font;
mod element;

use ash::vk;

pub use font::Font;
pub use element::*;

pub type Color = [f32; 3];
pub(crate) const NUM_OPERATIONS: f32 = 0xffff as f32;

pub mod color {
    use super::Color;
    pub const RED: Color = [1.0, 0.0, 0.0];
    pub const WHITE: Color = [1.0, 1.0, 1.0];
}

/// User interfaces hard-store UI layout. But what's actually shown on the ui, such as text or images,
/// is stored in the Elements themselves so that it can be updated regularly.
pub struct UserInterface<D> {
    pub data: D,
    pub elements: Vec<ElementData<D>>,
    operation_counter: u32,
}

pub trait UserInterfaceTrait {
    fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize);
}

impl<D> UserInterfaceTrait for UserInterface<D> {
    fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize) {
        for element in &self.elements {
            // Render everything but text
            element.render_hard(pipeline_layout, command_buffer, buffer_index);
        }
        let mut operation_counter = self.operation_counter;
        for element in &self.elements {
            element.render_soft(pipeline_layout, command_buffer, buffer_index, &mut operation_counter);
        }
    }
}

impl<D> UserInterface<D> {
    pub fn new(data: D) -> Self {
        Self {
            elements: Vec::new(),
            data,
            operation_counter: 0,
        }
    }

    pub fn add(mut self, element: Element<D>) -> Self {
        self.elements.push(ElementData::new(element, &mut self.operation_counter));
        self
    }

    pub fn mouse_down(&mut self, position: (f32, f32)) {
        for element in &self.elements {
            element.mouse_down(&mut self.data, position);
        }
    }
}