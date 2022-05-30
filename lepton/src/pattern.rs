use ash::vk;
use std::ptr;

use crate::constants::CLEAR_VALUES;
use crate::Graphics;
use crate::model::Model;

//use cgmath::{Matrix4, Vector3, Point3, Deg};
//use crate::graphics::{UniformBufferObject};

pub struct Pattern {
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    //uniform_transform: UniformBufferObject,
    //uniform_buffers: Vec<vk::Buffer>,
    //uniform_buffers_memory: Vec<vk::DeviceMemory>,
}

pub struct UnfinishedPattern {
    command_buffers: Vec<vk::CommandBuffer>,
}

impl Pattern {
    /// Begin writing to the pattern. Panics if the primary command buffer cannot be allocated or recording cannot begin.
    pub fn begin(graphics: &Graphics) -> UnfinishedPattern {
        let command_buffers = graphics.allocate_command_buffer();

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            graphics.begin_command_buffer(command_buffer, i);
        }

        UnfinishedPattern { command_buffers }
    }

    /*/// Update the pattern uniform buffer
    pub(crate) fn update_uniform_buffer(&self, graphics: &Graphics, image_index: usize, delta_time: f32) {
        self.uniform_transform.model =
            Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time)
                * self.uniform_transform.model;

        let ubos = [self.uniform_transform.clone()];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =graphics.device.map_memory(self.uniform_buffers_memory[image_index],
                0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            graphics.device.unmap_memory(self.uniform_buffers_memory[image_index]);
        }
    }*/
}

impl UnfinishedPattern {
    /// Wrap up the unfinished pattern. Consumes self.
    pub fn end(self, graphics: &Graphics) -> Pattern {
        for command_buffer in self.command_buffers.iter() {
            graphics.end_command_buffer(*command_buffer);
        }

        /*let uniform_transform =  UniformBufferObject {
            model: Matrix4::from_angle_z(Deg(90.0)),
            view: Matrix4::look_at_rh(
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let mut proj = cgmath::perspective(
                    Deg(45.0),
                    graphics.swapchain_extent.width as f32
                        / graphics.swapchain_extent.height as f32,
                    0.1,
                    10.0,
                );
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
        };*/

        Pattern { 
            command_buffers: self.command_buffers,
            //uniform_transform,
            //uniform_buffers,
            //uniform_buffers_memory,
        }
    }

    /// Inside the pattern, render a model.
    pub fn render(&self, graphics: &Graphics, model: &Model) {
        for (i, &command_buffer) in self.command_buffers.iter().enumerate() {
            model.render(graphics, &command_buffer, i);
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
    fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer, buffer_index: usize) {
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
                self.graphics_pipeline,
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