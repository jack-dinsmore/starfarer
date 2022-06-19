use std::rc::Rc;
use std::path::Path;
use vk_shader_macros::include_glsl;
use ash::vk;

use crate::Graphics;
use crate::model::{Model, VertexType, TextureType, primitives::Vertex2Tex};
use crate::shader::{Shader, builtin};

const N_COLS: usize = 12;
const N_ROWS: usize = 8;
const SIZE: usize = 48;



pub struct Font {
    pub(crate) model: Model,
    screen_width: f32,
    screen_height: f32,
    letter_width: f32,
    letter_height: f32,
}

impl Font {
    pub fn new(graphics: &Graphics, shader: &Shader) -> Font {
        let standard_width = SIZE as f32 / graphics.window_width as f32 * 2.0;
        let standard_height = SIZE as f32 / graphics.window_height as f32 * 2.0;

        let mut vertices = Vec::with_capacity((N_COLS + 1) * (N_ROWS + 1));
        let mut indices = Vec::with_capacity(6 * N_COLS * N_ROWS);
        let mut vertex_num = 0u32;
        for row in 0..N_ROWS {
            for col in 0..N_COLS {
                vertices.push(Vertex2Tex {
                    pos: [0.0, 0.0],
                    tex_coord: [col as f32 / N_COLS as f32, row as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [0.0, standard_height],
                    tex_coord: [col as f32 / N_COLS as f32, (row + 1) as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [standard_width, 0.0],
                    tex_coord: [(col + 1) as f32 / N_COLS as f32, row as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [standard_width, standard_height],
                    tex_coord: [(col + 1) as f32 / N_COLS as f32, (row + 1) as f32 / N_ROWS as f32],
                });
                indices.push(4 * vertex_num + 0);
                indices.push(4 * vertex_num + 1);
                indices.push(4 * vertex_num + 2);
                indices.push(4 * vertex_num + 1);
                indices.push(4 * vertex_num + 3);
                indices.push(4 * vertex_num + 2);
                vertex_num += 1;
            }
        }

        let model = Model::new::<builtin::UISignature>(graphics, shader, VertexType::Specified2Tex(vertices, indices),
            TextureType::Path(&Path::new("assets/fonts/rendered/font.png"))).expect("Could not find font");
        Self {
            model: Rc::try_unwrap(model).unwrap(),
            screen_width: graphics.window_width as f32,
            screen_height: graphics.window_height as f32,
            letter_width: standard_width,
            letter_height: standard_height,
        }
    }

    pub fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        text: &str, mut x: f32, y: f32) {
            for letter in text.chars() {
                self.render_char(pipeline_layout, command_buffer, frame_index, letter, x, y);
                x += self.letter_width;
            }
        }

    fn render_char(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        letter: char, x: f32, y: f32) {
        if (letter as usize) < 32 || (letter as usize) >= 128 {
            return;
        }
        let index = 6 * (letter as usize - 32);

        let push_constants = builtin::UIPushConstants {
            x,
            y,
            stretch_x: 1.0,
            stretch_y: 1.0,
            color: [0.0, 0.0, 0.0, 1.0],
        };
        let push_constant_bytes = unsafe { crate::tools::struct_as_bytes(&push_constants) };
        self.model.render_some(pipeline_layout, command_buffer, frame_index, Some(push_constant_bytes), index, 6);
    }
}