use ash::vk;
use std::mem::size_of;

use crate::{Graphics, shader, shader::Data};

static mut INPUT_UNIFORM_BUFFERS: InputUniformBuffers = InputUniformBuffers {
    camera: Vec::new(),
    lights: Vec::new(),
    custom: Vec::new(),
};

struct InputUniformBuffers {
    camera: Vec<vk::Buffer>,
    lights: Vec<vk::Buffer>,
    custom: Vec<Vec<vk::Buffer>>,
}

pub enum InputType {
    Camera,
    Lights,
    Custom(usize, usize, u32, shader::ShaderStages),
}

impl InputType {
    pub fn make_custom<D: Data>() -> InputType {
        let id = unsafe { INPUT_UNIFORM_BUFFERS.custom.len() };
        unsafe { INPUT_UNIFORM_BUFFERS.custom.push(Vec::new()) };
        InputType::Custom(id, size_of::<D>(), D::BINDING, D::STAGES)
    }

    pub fn new(&self, graphics: &Graphics) -> Input {
        let input = Input::new(graphics.memory_properties, graphics.swapchain_images.len(), self.get_size() as u64);
        unsafe {
            match self {
                InputType::Camera => { INPUT_UNIFORM_BUFFERS.camera = input.uniform_buffers.clone(); },
                InputType::Lights => { INPUT_UNIFORM_BUFFERS.lights = input.uniform_buffers.clone(); },
                InputType::Custom(i, _, _, _) => { INPUT_UNIFORM_BUFFERS.custom[*i] = input.uniform_buffers.clone(); },
            }
        }
        input
    }

    pub(crate) fn get_size(&self) -> u32 {
        match self {
            InputType::Camera => size_of::<shader::builtin::CameraData>() as u32,
            InputType::Lights => size_of::<shader::builtin::LightsData>() as u32,
            InputType::Custom(_, s, _, _) => *s as u32,
        }
    }

    pub(crate) fn get_binding(&self) -> u32 {
        match self {
            InputType::Camera => shader::builtin::CameraData::BINDING,
            InputType::Lights => shader::builtin::LightsData::BINDING,
            InputType::Custom(_, _, l, _) => *l,
        }
    }

    pub(crate) fn get_stages(&self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::from_raw(match self {
            InputType::Camera => shader::builtin::CameraData::STAGES.f,
            InputType::Lights => shader::builtin::LightsData::STAGES.f,
            InputType::Custom(_, _, _, s) => s.f,
        })
    }

    pub(crate) fn get_uniform_descriptor_buffer_info(&self, buffer_index: usize) -> Vec<vk::DescriptorBufferInfo> {
        
        vec![vk::DescriptorBufferInfo {
            buffer: self.get_uniform_buffers(buffer_index),
            offset: 0,
            range: self.get_size() as u64,
        }]
    }

    fn get_uniform_buffers(&self, buffer_index: usize) -> vk::Buffer {
        unsafe {
            match self {
                InputType::Camera => INPUT_UNIFORM_BUFFERS.camera[buffer_index],
                InputType::Lights => INPUT_UNIFORM_BUFFERS.lights[buffer_index],
                InputType::Custom(i, _, _, _) => INPUT_UNIFORM_BUFFERS.custom[*i][buffer_index],
            }
        }
    }
}

pub struct Input {
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    size: u64,
}

impl Input {
    fn new(memory_properties: vk::PhysicalDeviceMemoryProperties, num_images: usize, size: u64) -> Self {
        let (uniform_buffers, uniform_buffers_memory) = Graphics::create_uniform_buffers(memory_properties, num_images, size);
        Input {
            uniform_buffers,
            uniform_buffers_memory,
            size,
        }
    }

    /// Update the pattern uniform buffer
    pub fn update<D: shader::Data>(&self, uniform: D, buffer_index: usize) {
        let ubos = [uniform];

        let buffer_size = self.size;

        unsafe {
            let data_ptr =crate::get_device().map_memory(self.uniform_buffers_memory[buffer_index],
                0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("Failed to Map Memory") as *mut D;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), 1);

            crate::get_device().unmap_memory(self.uniform_buffers_memory[buffer_index]);
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = &crate::DEVICE {
                for (uniform_buffer, uniform_buffer_memory) in self.uniform_buffers.iter().zip(self.uniform_buffers_memory.iter()) {
                    device.destroy_buffer(*uniform_buffer, None);
                    device.free_memory(*uniform_buffer_memory, None);
                }
            }
        }
    }
}

impl Graphics {
    fn create_uniform_buffers(memory_properties: vk::PhysicalDeviceMemoryProperties, num_images: usize,
        buffer_size: u64) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {

        let mut uniform_buffers = Vec::new();
        let mut uniform_buffers_memory = Vec::new();
    
        for _ in 0..num_images {
            let (uniform_buffer, uniform_buffer_memory) = Graphics::create_buffer(
                &crate::get_device(),
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                memory_properties,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }
    
        (uniform_buffers, uniform_buffers_memory)
    }
}