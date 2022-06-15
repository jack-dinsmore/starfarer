use vk_shader_macros::include_glsl;
use std::rc::Rc;
use ash::vk;

use crate::{Graphics};
use crate::model::{Model, VertexType, TextureType, primitives::Vertex2Tex};
use crate::shader::{Shader, Signature, InputType, builtin::UIShader};

pub type Color = [f64; 3];

pub mod color {
    use super::Color;

    const RED: Color = [1.0, 0.0, 0.0];
}

struct UISignature;
impl Signature for UISignature {
    type V = Vertex2Tex;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.frag", kind: frag);
    const INPUTS: &'static [InputType] = &[];
}

pub struct UserInterface {
    pub model: Rc<Model>,
    pixels: Vec<u8>,
    image_size: u64,
}

impl UserInterface {
    pub fn new(graphics: &mut Graphics, ui_shader: &Shader) -> Self {
        let vertices = vec![
            Vertex2Tex{ pos: [1.0, 1.0], tex_coord: [0.0, 0.0]}, // Lower right
            Vertex2Tex{ pos: [-1.0, 1.0], tex_coord: [1.0, 0.0]}, // Lower left
            Vertex2Tex{ pos: [1.0, -1.0], tex_coord: [0.0, 1.0]}, // Upper right
            Vertex2Tex{ pos: [-1.0, -1.0], tex_coord: [1.0, 1.0]}, // Upper left
        ];
        let indices = vec![
            0, 2, 1, 1, 2, 3];
        let model = Model::new::<UIShader>(graphics, ui_shader, VertexType::Specified2Tex(vertices, indices), TextureType::Blank(graphics.window_width, graphics.window_height))
            .expect("Could not create UI");

        let pixels = vec![0; (graphics.window_width * graphics.window_height) as usize * 4];
        let image_size = std::mem::size_of::<u8>() as u64 * pixels.len() as u64;

        Self {
            model,
            pixels,
            image_size,
        }
    }

    pub fn write(&mut self, graphics: &Graphics, color: usize) {
        let color: [u8; 4] = match color {
            0 => [255, 255, 255, 128],
            1 => [255, 0, 255, 128],
            2 => [255, 255, 0, 128],
            3 => [0, 0, 255, 128],
            4 => [255, 0, 0, 128],
            _ => panic!("Unknown buffer index")
        };
        for i in 0..(graphics.window_width * graphics.window_height) as usize {
            // Time consuming in debug mode
            self.pixels[i * 4 + 0] = color[0];
            self.pixels[i * 4 + 1] = color[1];
            self.pixels[i * 4 + 2] = color[2];
            self.pixels[i * 4 + 3] = color[3];
        }

        // Transition image to general
        graphics.transition_image_layout(
            self.model.texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            1,
        );

        // Map image, write, and unmap
        unsafe {
            let data_ptr = crate::get_device().map_memory(self.model.texture_image_memory, 0, self.image_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(self.pixels.as_ptr(), self.pixels.len());

            crate::get_device().unmap_memory(self.model.texture_image_memory);
        }

        // Re-transition
        graphics.transition_image_layout(
            self.model.texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            1,
        );
    }
}