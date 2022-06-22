use std::rc::Rc;
use std::path::Path;
use ash::vk;
use std::fs::File;
use std::io::prelude::*;

use crate::Graphics;
use crate::model::{Model, VertexType, TextureType, primitives::Vertex2Tex};
use crate::shader::{Shader, builtin};

const N_COLS: usize = 12;
const N_ROWS: usize = 8;
const N_CHARS: usize = N_COLS * N_ROWS;


pub struct Font {
    pub(crate) model: Model,
    kerns: Vec<f32>,
    screen_width: f32,
    screen_height: f32,
    letter_width: f32,
    letter_height: f32,
}

impl Font {
    pub fn new(graphics: &Graphics, shader: &Shader, font_name: &str, size: usize) -> Font {
        let standard_width = size as f32 / graphics.window_width as f32 * 2.0;
        let standard_height = size as f32 / graphics.window_height as f32 * 2.0;

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
            TextureType::Monochrome(&Path::new(&format!("assets/fonts/rendered/{}-{}.png", font_name, size)))).expect("Could not find font");

        let kerns = {
            let mut file = File::open(&Path::new(&format!("assets/fonts/rendered/{}-{}.dat", font_name, size))).expect("Could not find the kerning file");
            // read the same file back into a Vec of bytes
            let mut buffer = Vec::<u8>::with_capacity(N_CHARS * N_CHARS);
            file.read_to_end(&mut buffer).expect("Could not read kerning file");
            // Leak the buffer
            let i8_buffer = unsafe {
                // Leak buffer
                let mut buffer = std::mem::ManuallyDrop::new(buffer);
                Vec::from_raw_parts(
                    buffer.as_mut_ptr() as *mut i8,
                    buffer.len(),
                    buffer.capacity()
                )
            };
            i8_buffer.iter().map(|x| *x as f32 / graphics.window_width as f32 * 2.0).collect::<Vec<_>>()
        };

        Self {
            model: Rc::try_unwrap(model).unwrap(),
            kerns,
            screen_width: graphics.window_width as f32,
            screen_height: graphics.window_height as f32,
            letter_width: standard_width,
            letter_height: standard_height,
        }
    }

    pub fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        text: &str, mut x: f32, y: f32) {
            let mut last_char = None;
            for letter in text.chars() {
                if let Some(left) = last_char {
                    let kern = self.kerns[(left as usize - 32) * N_CHARS + letter as usize - 32];
                    x += kern;
                }
                self.render_char(pipeline_layout, command_buffer, frame_index, letter, x, y);
                x += self.letter_width;
                last_char = Some(letter);
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