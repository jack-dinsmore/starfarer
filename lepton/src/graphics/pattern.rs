use ash::vk;
use std::ptr;
use std::rc::Rc;

use crate::constants::CLEAR_VALUES;
use crate::Graphics;
use crate::shader::ShaderTrait;
use crate::physics::Object;
use crate::RenderData;
use crate::model::Model;
use crate::ui::UserInterfaceTrait;

pub struct Pattern {
    command_buffers: Vec<vk::CommandBuffer>,
}

pub enum Action<'a> {
    DrawObject(&'a mut Object),
    LoadShader(&'a dyn ShaderTrait),
    DrawModel(&'a Rc<Model>),
    DrawUI(&'a dyn UserInterfaceTrait),
}

impl Pattern {
    pub fn new(graphics: &Graphics) -> Self {
        let command_buffers = graphics.allocate_command_buffer();
        Self {
            command_buffers,
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

    pub fn record(&mut self, graphics: &Graphics, buffer_index: usize, actions: &mut Vec<Action>) {
        unsafe {
            crate::get_device().reset_command_buffer(
                self.command_buffers[buffer_index],
                vk::CommandBufferResetFlags::empty(),
            ).expect("Resetting the command buffer failed");
        }

        graphics.begin_command_buffer(self.command_buffers[buffer_index], buffer_index);
        let mut pipeline_layout = None;

        for action in actions.iter_mut() {
            match action {
                Action::LoadShader(s) => {
                    unsafe { crate::get_device().cmd_bind_pipeline(self.command_buffers[buffer_index],
                        vk::PipelineBindPoint::GRAPHICS, s.get_pipeline()); }
                        pipeline_layout = Some(s.get_pipeline_layout());
                },
                Action::DrawObject(o) => {
                    o.make_push_constants();
                    if let Some(ref m) = o.model {
                        m.render(pipeline_layout.expect("You must first load a shader"),
                            self.command_buffers[buffer_index], buffer_index, Some(o.get_push_constant_bytes()));
                    }
                },
                Action::DrawModel(m) => {
                    m.render(pipeline_layout.expect("You must first load a shader"),
                        self.command_buffers[buffer_index], buffer_index, None);
                },
                Action::DrawUI(u) => {
                    u.render(pipeline_layout.expect("You must first load a shader"), 
                        self.command_buffers[buffer_index], buffer_index);
                }
            }
        }

        graphics.end_command_buffer(self.command_buffers[buffer_index]);
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
    fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer, buffer_index: usize) {
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