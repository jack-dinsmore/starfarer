pub mod font;

use ash::vk;

use crate::{Graphics};
use crate::shader::Shader;
use font::Font;

pub type Color = [f64; 3];

pub mod color {
    use super::Color;

    const RED: Color = [1.0, 0.0, 0.0];
}

pub struct UserInterface {
    font: Font,
    time: f32,
    short_time: i32,
    frames: u32,
    fps: u32
}

impl UserInterface {
    pub fn new(graphics: &mut Graphics, shader: &Shader) -> Self {
        let font = Font::new(graphics, shader, "Roboto-Regular", 48);
        Self {
            font,
            time: 0.0,
            short_time: 0,
            frames: 0,
            fps: 0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.frames += 1;
        if self.time as i32 != self.short_time {
            // Update FPS
            self.fps = self.frames;
            self.frames = 0;
            self.short_time = self.time as i32;
        }
    }

    pub fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, buffer_index: usize) {
        let mut operation_index = 0;
        self.font.render(pipeline_layout, command_buffer, buffer_index, &format!("FPS: {}", self.fps), -1.0, -1.0, &mut operation_index);
    }
}