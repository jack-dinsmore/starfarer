mod camera;
mod input;
mod lights;
mod object;
pub mod builtin;

use ash::vk;
use std::ptr;
use std::ffi::CString;
use std::marker::PhantomData;

pub use camera::*;
pub use lights::*;
pub use object::*;
pub use input::{Input, InputType};
pub(crate) use input::PushConstants;
use crate::Graphics;
use crate::model::primitives::Vertex;
use crate::shader;

pub struct ShaderStages {
    f: u32,
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

    const fn and(self, rhs: Self) -> Self {
        Self{ f: self.f | rhs.f}
    }
}



pub trait Data: Clone + Copy + Send + Sync + 'static {
    const BINDING: u32;
    const STAGES: ShaderStages;
}

// pub struct Signature {
//     pub VERTEX_CODE: &'static [u32],
//     pub FRAGMENT_CODE: &'static [u32],
//     pub inputs: &'static [InputType],
// }

pub trait Signature {
    type V: Vertex;
    const VERTEX_CODE: &'static [u32];
    const FRAGMENT_CODE: &'static [u32];
    const INPUTS: &'static [InputType];
}

pub struct Shader {
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) ubo_layout: vk::DescriptorSetLayout,
}

impl Shader {
    pub fn new<S: Signature>(graphics: &mut Graphics) -> Self {
        let vert_shader_module = graphics.create_shader_module(S::VERTEX_CODE.to_vec());
        let frag_shader_module = graphics.create_shader_module(S::FRAGMENT_CODE.to_vec());

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                // Vertex Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: vert_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo {
                // Fragment Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: frag_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let ubo_layout = Graphics::create_ubo_layout(S::INPUTS, &graphics.memory_properties, graphics.swapchain_imageviews.len());

        let (pipeline, pipeline_layout) = graphics.create_graphics_pipeline::<S>(&shader_stages, ubo_layout);

        unsafe {
            crate::get_device().destroy_shader_module(vert_shader_module, None);
            crate::get_device().destroy_shader_module(frag_shader_module, None);
        }

        Self {
            pipeline,
            pipeline_layout,
            ubo_layout,
        }
    }

    pub(crate) fn reload<S: Signature>(&mut self, graphics: &Graphics) {
        //let vert_shader_module = graphics.create_shader_module(S::VERTEX_CODE.to_vec());
        //let frag_shader_module = graphics.create_shader_module(S::FRAGMENT_CODE.to_vec());
        let vert_shader_module = graphics.create_shader_module(S::VERTEX_CODE.to_vec());
        let frag_shader_module = graphics.create_shader_module(S::FRAGMENT_CODE.to_vec());

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                // Vertex Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: vert_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo {
                // Fragment Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: frag_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let (pipeline, pipeline_layout) = graphics.create_graphics_pipeline::<S>(&shader_stages, self.ubo_layout);

        unsafe {
            crate::get_device().destroy_shader_module(frag_shader_module, None);
            crate::get_device().destroy_shader_module(vert_shader_module, None);
            crate::get_device().destroy_pipeline_layout(self.pipeline_layout, None);
            crate::get_device().destroy_pipeline(self.pipeline, None);
        }

        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = &crate::DEVICE {
                device.destroy_pipeline_layout(self.pipeline_layout, None);
                device.destroy_pipeline(self.pipeline, None);
            }
        }
    }
}

impl Graphics {
    fn create_shader_module(&self, code: Vec<u32>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len() * 4,
            p_code: code.as_ptr(),
        };
    
        unsafe {
            crate::get_device()
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create shader module!")
        }
    }

    fn create_graphics_pipeline<S: Signature>(&self, shader_stages: &[vk::PipelineShaderStageCreateInfo],
        ubo_layout: vk::DescriptorSetLayout) -> (vk::Pipeline, vk::PipelineLayout) {

        let binding_description = S::V::get_binding_descriptions();
        let attribute_description = S::V::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: attribute_description.len() as u32,
            p_vertex_attribute_descriptions: attribute_description.as_ptr(),
            vertex_binding_description_count: binding_description.len() as u32,
            p_vertex_binding_descriptions: binding_description.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next: ptr::null(),
            primitive_restart_enable: vk::FALSE,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swapchain_extent.width as f32,
            height: self.swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_extent,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next: ptr::null(),
            rasterization_samples: self.msaa_samples,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: vk::FALSE,
            alpha_to_coverage_enable: vk::FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: stencil_state,
            back: stencil_state,
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };

        let set_layouts = [ubo_layout];

        let push_constant_ranges = [vk::PushConstantRange {
            offset: 0,
            size: std::mem::size_of::<shader::PushConstants>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        }];

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
        };

        let pipeline_layout = unsafe {
            crate::get_device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_statue_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: &depth_state_create_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: ptr::null(),
            layout: pipeline_layout,
            render_pass: self.render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        }];

        let graphics_pipelines = unsafe {
            crate::get_device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphic_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline!.")
        };

        (graphics_pipelines[0], pipeline_layout)
    }


    fn create_ubo_layout(input_types: &'static [InputType], memory_properties: &vk::PhysicalDeviceMemoryProperties, num_images: usize) -> vk::DescriptorSetLayout {
        let mut ubo_layout_bindings = Vec::with_capacity(input_types.len() + 1);
        for input_type in input_types {
            ubo_layout_bindings.push(vk::DescriptorSetLayoutBinding {
                // Shader uniform
                binding: input_type.get_binding(),
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: input_type.get_stages(),
                p_immutable_samplers: ptr::null(),
            });
            input_type.make(memory_properties, num_images);
        }
        ubo_layout_bindings.push(vk::DescriptorSetLayoutBinding {
            // Texture sampler
            binding: 3,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        });

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: ubo_layout_bindings.len() as u32,
            p_bindings: (&ubo_layout_bindings[..]).as_ptr(),
        };

        unsafe {
            crate::get_device()
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create descriptor set layout!")
        }
    }
}