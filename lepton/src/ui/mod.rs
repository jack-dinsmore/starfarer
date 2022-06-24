pub mod font;

use ash::vk;
use std::rc::Rc;

use crate::shader::{builtin};
use crate::model::{Model};
pub use font::Font;

pub type Color = [f64; 3];
pub(crate) const NUM_OPERATIONS: f32 = 0xffff as f32;

pub mod color {
    use super::Color;

    const RED: Color = [1.0, 0.0, 0.0];
}

pub trait UserInterface {
    fn get_elements(&self) -> &Vec<Element>;
}

pub enum Element {
    Text(Rc<Font>, String, f32, f32),
    Button(Rc<Font>, Rc<Model>, String, f32, f32, f32, f32, f32, f32),
    Background(Rc<Model>, f32, f32, f32, f32),
}

impl Element {
    pub(crate) fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize, operation_index: &mut u32) {
        match self {
            Element::Text(font, text, x, y) => {
                font.render(pipeline_layout, command_buffer, buffer_index, text, *x, *y, operation_index);
            },
            Element::Button(font, blank, text, fontx, fonty, x, y, width, height) => {
                let push_constants = builtin::UIPushConstants {
                    x: *x,
                    y: *y,
                    stretch_x: *width,
                    stretch_y: *height,
                    color: [1.0, 1.0, 1.0, 1.0],
                    depth: 0.5 - *operation_index as f32 / NUM_OPERATIONS / 2.0,
                };
                let push_constant_bytes = unsafe { crate::tools::struct_as_bytes(&push_constants) };
                blank.render(pipeline_layout, command_buffer, buffer_index, Some(push_constant_bytes));
                *operation_index += 1;
                font.render(pipeline_layout, command_buffer, buffer_index, text, *fontx, *fonty, operation_index);
            },
            Element::Background(blank, x, y, width, height) => {
                let push_constants = builtin::UIPushConstants {
                    x: *x,
                    y: *y,
                    stretch_x: *width,
                    stretch_y: *height,
                    color: [0.1, 0.1, 0.1, 1.0],
                    depth: 0.5 - *operation_index as f32 / NUM_OPERATIONS / 2.0,
                };
                let push_constant_bytes = unsafe { crate::tools::struct_as_bytes(&push_constants) };
                blank.render(pipeline_layout, command_buffer, buffer_index, Some(push_constant_bytes));
                *operation_index += 1;
            }
        }
    }

    pub fn new_text(font: Rc<Font>, text: String, x: f32, y: f32) -> Element {
        Element::Text(font, text, x, y)
    }

    pub fn new_button(font: Rc<Font>, blank: Rc<Model>, text: String, x: f32, y: f32, width: f32, height: f32) -> Element {
        let text_length = font.length(&text);
        let text_height = font.height();
        Element::Button(font, blank, text, x - text_length / 2.0 - width / 2.0, y - text_height / 2.0 - height / 2.0,
            x - width / 2.0, y - height / 2.0, width, height)
    }

    pub fn new_background(blank: Rc<Model>, x: f32, y: f32, width: f32, height: f32) -> Element {
        Element::Background(blank, x - width / 2.0, y - height / 2.0, width, height)
    }
}


pub(crate) fn render_user_interface(ui: &dyn UserInterface, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize) {
    let mut operation_index = 0;
    for element in ui.get_elements() {
        element.render(pipeline_layout, command_buffer, buffer_index, &mut operation_index);
    }
}