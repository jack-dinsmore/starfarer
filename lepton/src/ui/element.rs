use std::rc::Rc;
use ash::vk;

use crate::shader::{builtin::UIPushConstants};
use super::{Font, Color, NUM_OPERATIONS};
use crate::model::{Model};

pub enum Element<D> {
    Text{ font: Rc<Font>, text: String, x: f32, y: f32, color: Color },
    Button{ font: Rc<Font>, blank: Rc<Model>, text: String, x: f32, y: f32, width: f32, height: f32, action: Box<dyn Fn(&mut D)->()> },
    Background{ blank: Rc<Model>, x: f32, y: f32, width: f32, height: f32 },
}


pub enum ElementData<D> {
    Text{ font: Rc<Font>, text: String, x: f32, y: f32, color: Color },
    Button{ pc: UIPushConstants, font: Rc<Font>, blank: Rc<Model>, text: String, x: f32, y: f32, width: f32, height: f32, font_x: f32, font_y: f32, action: Box<dyn Fn(&mut D)->()> },
    Background{ pc: UIPushConstants, blank: Rc<Model> },
}

impl<D> ElementData<D> {
    pub fn new(e: Element<D>, operation_index: &mut u32) -> Self {
        match e {
            Element::Text { font, text, x, y, color } => ElementData::Text { font, text, x, y, color },
            Element::Button { font, blank, text, x, y, width, height, action } => {
                let font_length = font.length(&text);
                let font_height = font.height();
                let data = ElementData::Button {
                    pc: UIPushConstants {
                        x: x - width / 2.0,
                        y: y - height / 2.0,
                        stretch_x: width,
                        stretch_y: height,
                        color: [1.0, 1.0, 1.0, 1.0],
                        depth: 0.5 - *operation_index as f32 / NUM_OPERATIONS / 2.0,
                    },
                    font, blank, text, action, width, height,
                    x: x - width / 2.0,
                    y: y - height / 2.0,
                    font_x: x - font_length as f32 / 2.0,
                    font_y: y - font_height as f32 / 2.0,
                };
                *operation_index += 1;
                data
            },
            Element::Background { blank, x, y, width, height } => {
                let data = ElementData::Background {
                    pc: UIPushConstants {
                        x: x - width / 2.0,
                        y: y - height / 2.0,
                        stretch_x: width,
                        stretch_y: height,
                        color: [0.1, 0.1, 0.1, 1.0],
                        depth: 0.5 - *operation_index as f32 / NUM_OPERATIONS / 2.0,
                    },
                    blank
                };
                *operation_index += 1;
                data
            }
        }
    }
}

impl<D> ElementData<D> {
    pub(crate) fn render_hard(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize) {
        match self {
            ElementData::Button{blank, pc, ..} => {
                let push_constant_bytes = crate::tools::struct_as_bytes(pc);
                blank.render(pipeline_layout, command_buffer, buffer_index, Some(push_constant_bytes));
            },
            ElementData::Background{blank, pc, ..} => {
                let push_constant_bytes =crate::tools::struct_as_bytes(pc);
                blank.render(pipeline_layout, command_buffer, buffer_index, Some(push_constant_bytes));
            }
            _ => (),
        }
    }

    pub(crate) fn render_soft(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize, operation_index: &mut u32) {
        match self {
            ElementData::Text{ font, text, x, y, color } => {
                font.render(pipeline_layout, command_buffer, buffer_index, text, *x, *y, color.clone(), operation_index);
            },
            ElementData::Button{font, text, font_x, font_y, ..} => {
                font.render(pipeline_layout, command_buffer, buffer_index, text, *font_x, *font_y, crate::ui::color::WHITE, operation_index);
            },
            _ => (),
        }
    }

    pub(crate) fn mouse_down(&self, data: &mut D, position: (f32, f32)) {
        match self {
            ElementData::Button{x, y, width, height, action, ..} => {
                if *x < position.0 && position.0 < *x + *width && *y < position.1 && position.1 < *y + *height {
                    action(data);
                }
            }
            _ => (),
        }
    }
}