use ash::vk;

use super::vertex::Vertex;
use crate::input::{InputType, InputLevel};
use crate::shader::{Shader};

pub struct ShaderStages {
    pub(crate) f: u32,
}

impl ShaderStages {
    // Must agree with vk::ShaderStageFlags
    pub const VERTEX: Self = Self{ f: 0b1 };
    pub const FRAGMENT: Self = Self{ f: 0b1_0000 };
    // pub const TESSELLATION_CONTROL: u32 = Self{ f: 0b10 };
    // pub const TESSELLATION_EVALUATION: u32 = Self{ f: 0b100 };
    // pub const GEOMETRY: u32 = Self{ f: 0b1000 };
    // pub const COMPUTE: u32 = Self{ f: 0b10_0000 };
    // pub const ALL_GRAPHICS: u32 = Self{ f: 0x0000_001F };
    // pub const ALL: u32 = Self{ f: 0x7FFF_FFFF };

    pub const fn and(self, rhs: Self) -> Self {
        Self{ f: self.f | rhs.f}
    }
}



pub trait Data: Clone + Copy + Send + Sync + 'static {
    const STAGES: ShaderStages;
    const LEVEL: InputLevel;
}

pub trait Signature {
    type V: Vertex;
    type PushConstants;
    const VERTEX_CODE: &'static [u32];
    const FRAGMENT_CODE: &'static [u32];
    const INPUTS: &'static [InputType];
    
}

pub trait ShaderTrait {
    fn get_pipeline(&self) -> vk::Pipeline;
    fn get_pipeline_layout(&self) -> vk::PipelineLayout;
    fn get_descriptor_set(&self, frame_index: usize) -> Option<vk::DescriptorSet>;
    fn get_bind_index(&self) -> u32;
    fn load(&self, command_buffer: vk::CommandBuffer, frame_index: usize) ->  vk::PipelineLayout{
        unsafe { crate::get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
                self.get_pipeline());
        }
        let pipeline_layout = self.get_pipeline_layout();

        if let Some(set) = &self.get_descriptor_set(frame_index) {
            let descriptor_set_to_bind = [*set];
            unsafe {
                crate::get_device().cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout, self.get_bind_index(), &descriptor_set_to_bind, &[]);
            }
        }

        pipeline_layout
    }
}

impl<S: Signature> ShaderTrait for Shader<S> {
    fn get_pipeline(&self) -> vk::Pipeline { self.pipeline }
    fn get_pipeline_layout(&self) -> vk::PipelineLayout { self.pipeline_layout }
    fn get_descriptor_set(&self, frame_index: usize) -> Option<vk::DescriptorSet> {
        if let Some((_, sets)) = &self.shader_descriptor_set {
            Some(sets.get(frame_index))
        } else {
            None
        }
    }
    fn get_bind_index(&self) -> u32 { 
        0
    }
}
