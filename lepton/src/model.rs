use anyhow::Result;
use ash::vk;
use std::rc::Rc;
use cgmath::Matrix4;
use std::sync::mpsc::Sender;

use crate::graphics::{Graphics, Deletable, DoubleBuffered};
use crate::shader::{Shader, Signature, vertex::Vertex};
use crate::input::{Input, InputLevel, VertexType};


pub struct Model {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    num_indices: u32,
    
    inputs: Vec<Input>,
    descriptor_set: Option<DoubleBuffered<vk::DescriptorSet>>,
    delete_sender: Option<Sender<Deletable>>,
}

pub enum DrawState {
    Standard(Rc<Model>),
    Offset(Rc<Model>, Matrix4<f32>),
}

// Constructors
impl Model {
    pub fn new<'a, V: Vertex, S: Signature>(graphics: &Graphics, shader: &Shader<S>, vertex_type: VertexType<'a, V>, inputs: Vec<Input>) -> Result<Self> {

        let ((vertex_buffer, vertex_buffer_memory), (index_buffer, index_buffer_memory), num_indices) = match vertex_type {
            VertexType::Specified(v, i) => (graphics.create_vertex_buffer(&v), graphics.create_index_buffer(&i), i.len() as u32),
            VertexType::Compiled(b, n) => {
                let (vertices, indices) = VertexType::<'a, V>::to_vectors(b, n);
                (graphics.create_vertex_buffer(&vertices), graphics.create_index_buffer(&indices), indices.len() as u32)
            },
        };

        let descriptor_set = if inputs.is_empty() {
            None
        } else {
            let descriptor_set = graphics.allocate_descriptor_set(shader.descriptor_set_layout);
            let mut input_index = 0;
            for (i, shader_input) in S::INPUTS.iter().enumerate() {
                if let InputLevel::Model = shader_input.get_level() {
                    // Check for the input in the array
                    match inputs.get(input_index) {
                        Some(local_input) => local_input.add_descriptor(&descriptor_set, i as u32),
                        None => panic!("You tried to create a new model without providing enough inputs.")
                    }
                    input_index += 1;
                }
            }
            Some(descriptor_set)
        };

        let model = Model {
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            num_indices,

            inputs,
            descriptor_set,
            delete_sender: Some(graphics.delete_sender.clone()),
        };

        Ok(model)
    }

    // pub fn null() -> Result<Self> {
    //     Ok(Model {
    //         vertex_buffer: vk::Buffer::null(),
    //         vertex_buffer_memory: vk::DeviceMemory::null(),
    //         index_buffer: vk::Buffer::null(),
    //         index_buffer_memory: vk::DeviceMemory::null(),
    //         num_indices: 0,

    //         handles: Vec::new(),
    //         delete_sender: None,
    //     })
    // }
}

impl Model {
    pub(crate) fn render(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize, push_constant_bytes: Option<&[u8]>) {
        if self.num_indices == 0 { return; }
        self.bind_all(pipeline_layout, command_buffer, frame_index, push_constant_bytes);
        unsafe {
            crate::get_device().cmd_draw_indexed(command_buffer, self.num_indices, 1, 0, 0, 0);
        }
    }

    pub(crate) fn render_some(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize,
        push_constant_bytes: Option<&[u8]>, start_index: usize, count: usize) {
        self.bind_all(pipeline_layout, command_buffer, frame_index, push_constant_bytes);
        unsafe {
            crate::get_device().cmd_draw_indexed(command_buffer, count as u32, 1, start_index as u32, 0, 0);
        }
    }

    fn bind_all(&self, pipeline_layout: vk::PipelineLayout, command_buffer: vk::CommandBuffer, frame_index: usize, push_constant_bytes: Option<&[u8]>) {
        if self.num_indices == 0 { return; }
        
        let vertex_buffers = [self.vertex_buffer];
        let offsets = [0_u64];

        unsafe {
            crate::get_device().cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            crate::get_device().cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);
            if let Some(set) = &self.descriptor_set {
                let descriptor_set_to_bind = [set.get(frame_index)];
                crate::get_device().cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout, 0, &descriptor_set_to_bind, &[]);
            }
            if let Some(pc) = push_constant_bytes {
                crate::get_device().cmd_push_constants(command_buffer, pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0, pc);
            }
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.delete_sender.as_ref().unwrap().send(Deletable::Buffer(self.vertex_buffer, self.vertex_buffer_memory)).unwrap_or(());
        self.delete_sender.as_ref().unwrap().send(Deletable::Buffer(self.index_buffer, self.index_buffer_memory)).unwrap_or(());
    }
}

impl Graphics {
    
    fn create_vertex_buffer<T>(&self, data: &[T]) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            crate::get_device(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            self.memory_properties,
        );

        unsafe {
            let data_ptr = crate::get_device().map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut T;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            crate::get_device().unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            crate::get_device(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            self.memory_properties,
        );

        self.copy_buffer(&self.graphics_queue, staging_buffer, vertex_buffer, buffer_size);

        unsafe {
            crate::get_device().free_memory(staging_buffer_memory, None);
            crate::get_device().destroy_buffer(staging_buffer, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }

    fn create_index_buffer(&self, data: &[u32]) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(crate::get_device(), buffer_size, vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, self.memory_properties);
            
        unsafe {
            let data_ptr = crate::get_device().map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            crate::get_device().unmap_memory(staging_buffer_memory);
        }
        
        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            crate::get_device(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            self.memory_properties,
        );

        self.copy_buffer(&self.graphics_queue, staging_buffer, index_buffer, buffer_size);

        unsafe {
            crate::get_device().destroy_buffer(staging_buffer, None);
            crate::get_device().free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
    }

}