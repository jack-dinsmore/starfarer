use ash::vk;
use std::ptr;

use cgmath::{Matrix4, Vector3, Point3, Deg};

use crate::constants::CLEAR_VALUES;
use crate::Graphics;
use crate::model::Model;
use crate::shader::{Shader, ShaderStarter, ShaderData};

pub trait PatternTrait {
    fn update_uniform_buffer(&self, graphics: &Graphics, image_index: usize);
    fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer;
}

pub struct Pattern<D: ShaderData> {
    shader: Shader<D>,
    command_buffers: Vec<vk::CommandBuffer>,
}

pub struct UnfinishedPattern<D: ShaderData> {
    pub(crate) shader: Shader<D>,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl<D: ShaderData> PatternTrait for Pattern<D> {
    fn update_uniform_buffer(&self, graphics: &Graphics, image_index: usize) {
        self.shader.update_uniform_buffer(graphics, image_index);
    }
    fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer {
        &self.command_buffers[image_index]
    }
}

impl<D: ShaderData> Pattern<D> {
    /// Begin writing to the pattern. Panics if the primary command buffer cannot be allocated or recording cannot begin.
    pub fn begin(graphics: &Graphics, shader_starter: ShaderStarter<D>) -> UnfinishedPattern<D> {
        let command_buffers = graphics.allocate_command_buffer();
        let shader = Shader::new(graphics, shader_starter);

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            graphics.begin_command_buffer(command_buffer, &shader.pipeline, i);
        }

        UnfinishedPattern::<D> { shader, command_buffers }
    }

    pub fn update_uniform(&mut self, delta_time: f32) {
        self.shader.update_uniform(delta_time);
    }
}

impl<D: ShaderData> UnfinishedPattern<D> {
    /// Wrap up the unfinished pattern. Consumes self.
    pub fn end(self, graphics: &Graphics) -> Pattern<D> {
        for command_buffer in self.command_buffers.iter() {
            graphics.end_command_buffer(*command_buffer);
        }

        Pattern::<D> {
            shader: self.shader,
            command_buffers: self.command_buffers,
        }
    }

    /// Inside the pattern, render a model.
    pub(crate) fn render(&self, graphics: &Graphics, model: &Model<D>) {
        for (i, &command_buffer) in self.command_buffers.iter().enumerate() {
            model.render(graphics, &self.shader.get_pipeline_layout(), &command_buffer, i);
        }
    }
}

impl Graphics {
    /// Allocates the primary command buffer. Panics if buffer cannot be allocated.
    fn allocate_command_buffer(&self) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: self.framebuffers.len() as u32,
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        unsafe {
            self.device.allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers!")
        }
    }

    /// Begin the ommand buffer. Panics if buffer cannot be started.
    fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer, pipeline: &vk::Pipeline, buffer_index: usize) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            self.device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording command buffer at beginning!");
        }

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[buffer_index],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            },
            clear_value_count: CLEAR_VALUES.len() as u32,
            p_clear_values: CLEAR_VALUES.as_ptr(),
        };

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *pipeline,
            );
        }
    }

    /// End the command buffer. Panics if buffer cannot be ended.
    fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)
                .expect("Failed to record command buffer at ending!");
        }
    }
}