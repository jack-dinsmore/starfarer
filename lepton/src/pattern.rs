use ash::vk;
use std::ptr;
use std::sync::Arc;

use crate::constants::CLEAR_VALUES;
use crate::Graphics;
use crate::model::Model;
use crate::shader::{Shader, ShaderData};

pub trait PatternTrait {
    fn update_uniform_buffer(&self, image_index: usize);
    fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer;
    fn check_swapchain_version(&mut self, graphics: &Graphics);
}

pub struct Pattern<D: ShaderData> {
    shader: Shader<D>,
    command_buffers: Vec<vk::CommandBuffer>,
    swapchain_current_version: u32,
    models: Vec<Arc<Model<D>>>,
}

pub struct UnfinishedPattern<D: ShaderData> {
    pub(crate) shader: Shader<D>,
    models: Vec<Arc<Model<D>>>,
}

impl<D: ShaderData> PatternTrait for Pattern<D> {
    fn update_uniform_buffer(&self, image_index: usize) {
        self.shader.update_uniform_buffer(image_index);
    }
    fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer {
        &self.command_buffers[image_index]
    }
    fn check_swapchain_version(&mut self, graphics: &Graphics) {
        if graphics.swapchain_ideal_version != self.swapchain_current_version {
            // Unload command buffer
            unsafe { crate::get_device().free_command_buffers(graphics.command_pool, &self.command_buffers); }

            self.shader.recreate_swapchain(graphics);

            // Reload command buffer
            let command_buffers = graphics.allocate_command_buffer();

            for (i, &command_buffer) in command_buffers.iter().enumerate() {
                graphics.begin_command_buffer(command_buffer, &self.shader.pipeline, i);

                for model in &self.models {
                    model.render(&self.shader.pipeline_layout, &command_buffer, i);
                }

                graphics.end_command_buffer(command_buffer);
            }

            self.command_buffers = command_buffers;
            self.swapchain_current_version = graphics.swapchain_ideal_version;
        }
    }
}

impl<D: ShaderData> Pattern<D> {
    /// Begin writing to the pattern. Panics if the primary command buffer cannot be allocated or recording cannot begin.
    pub fn begin(graphics: &Graphics) -> UnfinishedPattern<D> {
        let shader = Shader::new(graphics);

        UnfinishedPattern::<D> {
            shader,
            models: Vec::new()
        }
    }

    pub fn uniform(&mut self) -> &mut D {
        &mut self.shader.uniform
    }
}

impl<D: ShaderData> UnfinishedPattern<D> {
    pub fn add(&mut self, model: Arc<Model<D>>) {
        self.models.push(model);
    }

    /// Wrap up the unfinished pattern. Consumes self.
    pub fn end(self, graphics: &Graphics) -> Pattern<D> {
        let command_buffers = graphics.allocate_command_buffer();

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            graphics.begin_command_buffer(command_buffer, &self.shader.pipeline, i);

            for model in &self.models {
                model.render(&self.shader.pipeline_layout, &command_buffer, i);
            }

            graphics.end_command_buffer(command_buffer);
        }

        Pattern::<D> {
            shader: self.shader,
            models: self.models,
            command_buffers,
            swapchain_current_version: graphics.swapchain_current_version,
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
            crate::get_device().allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers!")
        }
    }

    /// Begin the command buffer. Panics if buffer cannot be started.
    fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer, pipeline: &vk::Pipeline, buffer_index: usize) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            crate::get_device().begin_command_buffer(command_buffer, &command_buffer_begin_info)
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
            crate::get_device().cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            crate::get_device().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *pipeline,
            );
        }
    }

    /// End the command buffer. Panics if buffer cannot be ended.
    fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            crate::get_device().cmd_end_render_pass(command_buffer);
            crate::get_device().end_command_buffer(command_buffer)
                .expect("Failed to record command buffer at ending!");
        }
    }
}