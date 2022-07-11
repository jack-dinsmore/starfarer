use ash::vk;

use crate::Graphics;
use crate::model::{Model, VertexType, TextureType, vertex::Vertex2Tex};
use crate::shader::{Shader, builtin};
use crate::ui::Color;

const N_COLS: usize = 12;// COMMON! DO NOT CHANGE WITHOUT ADJUSTING STARFARER_MACROS
const N_ROWS: usize = 8;// COMMON! DO NOT CHANGE WITHOUT ADJUSTING STARFARER_MACROS
const N_CHARS: usize = N_COLS * N_ROWS;// COMMON! DO NOT CHANGE WITHOUT ADJUSTING STARFARER_MACROS

pub struct Font {
    pub(crate) model: Model,
    kerns: Vec<f32>,
    screen_width: f32,
    screen_height: f32,
    letter_width: f32,
    letter_height: f32,
}

impl Font {
    pub fn new(graphics: &Graphics, shader: &Shader<builtin::UISignature>, bytes: (&[u8], &[u8]), size: usize, spacing: i8) -> Font {
        let standard_width = size as f32 / graphics.window_width as f32 * 2.0;
        let standard_height = size as f32 / graphics.window_height as f32 * 2.0;

        let mut vertices = Vec::with_capacity((N_COLS + 1) * (N_ROWS + 1));
        let mut indices = Vec::with_capacity(6 * N_COLS * N_ROWS);
        let mut vertex_num = 0u32;
        for row in 0..N_ROWS {
            for col in 0..N_COLS {
                vertices.push(Vertex2Tex {
                    pos: [0.0, 0.0],
                    coord: [col as f32 / N_COLS as f32, row as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [0.0, standard_height],
                    coord: [col as f32 / N_COLS as f32, (row + 1) as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [standard_width, 0.0],
                    coord: [(col + 1) as f32 / N_COLS as f32, row as f32 / N_ROWS as f32],
                });
                vertices.push(Vertex2Tex {
                    pos: [standard_width, standard_height],
                    coord: [(col + 1) as f32 / N_COLS as f32, (row + 1) as f32 / N_ROWS as f32],
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

        let model = Model::new(graphics, shader, VertexType::Specified(vertices, indices),
            TextureType::Monochrome(bytes.0)).expect("Could not load font");

        let kerns = {
            let i8_buffer = unsafe {
                // Leak buffer, but I'm supposed to only do this for bytes.1. Small memory leak
                let bytes = std::mem::ManuallyDrop::new(bytes);
                std::slice::from_raw_parts(
                    bytes.1.as_ptr() as *mut i8,
                    bytes.1.len()
                )
            };
            i8_buffer.iter().map(|x| (*x + spacing) as f32 / graphics.window_width as f32 * 2.0).collect::<Vec<_>>()
        };

        Self {
            model,
            kerns,
            screen_width: graphics.window_width as f32,
            screen_height: graphics.window_height as f32,
            letter_width: standard_width,
            letter_height: standard_height,
        }
    }

    pub(crate) fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        text: &str, mut x: f32, y: f32, color: Color, operation_index: &mut u32) {

        let mut last_char = None;
        for letter in text.chars() {
            if let Some(left) = last_char {
                let kern = self.kerns[(left as usize - 32) * N_CHARS + letter as usize - 32];
                x += kern;
            }
            self.render_char(pipeline_layout, command_buffer, frame_index, letter, x, y, color, *operation_index);
            x += self.letter_width;
            last_char = Some(letter);
            *operation_index += 1
        }
    }

    pub fn length(&self, text: &str) -> f32{
        let mut x = 0.0;
        let mut last_char = None;
        for letter in text.chars() {
            if let Some(left) = last_char {
                let kern = self.kerns[(left as usize - 32) * N_CHARS + letter as usize - 32];
                x += kern;
            }
            x += self.letter_width;
            last_char = Some(letter);
        }

        x
    }

    pub fn height(&self) -> f32 {
        self.letter_height
    }

    fn render_char(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        letter: char, x: f32, y: f32, color: Color, operation_index: u32) {
        if (letter as usize) < 32 || (letter as usize) >= 128 {
            return;
        }
        let index = 6 * (letter as usize - 32);

        let push_constants = builtin::UIPushConstants {
            x,
            y,
            stretch_x: 1.0,
            stretch_y: 1.0,
            color: [color[0], color[1], color[2], 1.0],
            depth: 0.5 - operation_index as f32 / super::NUM_OPERATIONS / 2.0,
        };
        let push_constant_bytes = crate::tools::struct_as_bytes(&push_constants);
        self.model.render_some(pipeline_layout, command_buffer, frame_index, Some(push_constant_bytes), index, 6);
    }
}