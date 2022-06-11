use ash::vk;
use std::ptr;
use std::rc::Rc;

use crate::constants::CLEAR_VALUES;
use crate::Graphics;
use crate::model::Model;
use crate::shader::{Shader, Signature};
use crate::RenderData;

pub struct Pattern<S: Signature> {
    command_buffers: Vec<vk::CommandBuffer>,
    swapchain_current_version: u32,
    models: Vec<Rc<Model>>,
    shader: Shader<S>,
}

pub struct UnfinishedPattern<S: Signature> {
    models: Vec<Rc<Model>>,
    pub(crate) shader: Shader<S>,
}

impl<S: Signature> Pattern<S> {
    pub fn check_reload(&mut self, graphics: &Graphics) {
        if graphics.swapchain_ideal_version != self.swapchain_current_version {
            // Unload command buffer
            unsafe { crate::get_device().free_command_buffers(graphics.command_pool, &self.command_buffers); }

            // Reload command buffer
            let command_buffers = graphics.allocate_command_buffer();
            self.shader.reload(graphics);

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

    /// Begin writing to the pattern. Panics if the primary command buffer cannot be allocated or recording cannot begin.
    pub fn begin(graphics: &mut Graphics) -> UnfinishedPattern<S> {
        let shader = Shader::<S>::new(graphics);

        UnfinishedPattern::<S> {
            shader,
            models: Vec::new(),
        }
    }

    pub fn render(&self, render_data: &mut RenderData) {
        render_data.submit_infos.push(vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: render_data.wait_semaphores.len() as u32,
            p_wait_semaphores: render_data.wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: render_data.wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[render_data.buffer_index],
            
            signal_semaphore_count: render_data.signal_semaphores.len() as u32,
            p_signal_semaphores: render_data.signal_semaphores.as_ptr(),
        });
    }
}

impl<S: Signature> UnfinishedPattern<S> {
    pub fn add(&mut self, model: Rc<Model>) {
        self.models.push(model);
    }

    /// Wrap up the unfinished pattern. Consumes self.
    pub fn end(self, graphics: &Graphics) -> Pattern<S> {
        let command_buffers = graphics.allocate_command_buffer();

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            graphics.begin_command_buffer(command_buffer, &self.shader.pipeline, i);

            for model in &self.models {
                model.render(&self.shader.pipeline_layout, &command_buffer, i);
            }

            graphics.end_command_buffer(command_buffer);
        }

        Pattern {
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