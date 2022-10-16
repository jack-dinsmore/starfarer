pub mod vertex;
pub mod builtin;
mod primitives;

use ash::vk;
use std::ptr;
use std::ffi::CString;
use std::marker::PhantomData;
pub use vertex::Vertex;

pub use primitives::*;
use crate::Graphics;
use crate::graphics::DoubleBuffered;
use crate::input::{Input, InputLevel};

pub struct Shader<S: Signature> {
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) model_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    pub(crate) shader_descriptor_set: Option<(vk::DescriptorSetLayout, DoubleBuffered<vk::DescriptorSet>)>,
    phantom: PhantomData<S>,
}

impl<S: Signature> Shader<S> {
    pub fn new(graphics: &mut Graphics, inputs: Vec<&Input>) -> Self {
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

        let (shader_descriptor_set, model_descriptor_set_layout) = Self::create_descriptor_sets(graphics, inputs);

        let (pipeline, pipeline_layout) = graphics.create_graphics_pipeline::<S>(&shader_stages, &shader_descriptor_set, &model_descriptor_set_layout);

        unsafe {
            crate::get_device().destroy_shader_module(vert_shader_module, None);
            crate::get_device().destroy_shader_module(frag_shader_module, None);
        }

        Self {
            pipeline,
            pipeline_layout,
            model_descriptor_set_layout,
            shader_descriptor_set,
            phantom: PhantomData,
        }
    }

    pub(crate) fn reload(&mut self, graphics: &Graphics) {
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

        let (pipeline, pipeline_layout) = graphics.create_graphics_pipeline::<S>(&shader_stages, &self.shader_descriptor_set, &self.model_descriptor_set_layout);

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

/// Private static functions
impl<S: Signature> Shader<S> {
    fn create_descriptor_sets(graphics: &Graphics, inputs: Vec<&Input>
    ) -> (Option<(vk::DescriptorSetLayout, DoubleBuffered<vk::DescriptorSet>)>, Option<vk::DescriptorSetLayout>) {
        let mut shader_descriptor_set_layout_bindings = Vec::new();
        let mut model_descriptor_set_layout_bindings = Vec::new();

        for (i, input_type) in S::INPUTS.iter().enumerate() {
            match input_type.get_level() {
                InputLevel::Shader => {
                    shader_descriptor_set_layout_bindings.push(vk::DescriptorSetLayoutBinding {
                        binding: i as u32,
                        descriptor_type: input_type.get_descriptor_type(),
                        descriptor_count: 1,
                        stage_flags: input_type.get_stages(),
                        p_immutable_samplers: ptr::null(),
                    });
                },
                InputLevel::Model => {
                    model_descriptor_set_layout_bindings.push(vk::DescriptorSetLayoutBinding {
                        binding: i as u32,
                        descriptor_type: input_type.get_descriptor_type(),
                        descriptor_count: 1,
                        stage_flags: input_type.get_stages(),
                        p_immutable_samplers: ptr::null(),
                    });
                }
            }
        }

        let shader_set = if !shader_descriptor_set_layout_bindings.is_empty() {
            let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
                s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DescriptorSetLayoutCreateFlags::empty(),
                binding_count: shader_descriptor_set_layout_bindings.len() as u32,
                p_bindings: shader_descriptor_set_layout_bindings.as_ptr(),
            };
    
            let descriptor_set_layout = unsafe {
                crate::get_device()
                    .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                    .expect("Failed to create descriptor set layout!")
            };
    
            let shader_descriptor_set = graphics.allocate_descriptor_set(descriptor_set_layout);
            for (i, input_type) in S::INPUTS.iter().enumerate() {
                if let InputLevel::Shader = input_type.get_level() {
                    inputs[i].add_descriptor(&shader_descriptor_set, i as u32);
                }
            }
            Some((descriptor_set_layout, shader_descriptor_set))
        } else {
            None
        };

        let model_set = if !model_descriptor_set_layout_bindings.is_empty() {
            let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
                s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DescriptorSetLayoutCreateFlags::empty(),
                binding_count: model_descriptor_set_layout_bindings.len() as u32,
                p_bindings: model_descriptor_set_layout_bindings.as_ptr(),
            };
    
            Some(unsafe {
                crate::get_device()
                    .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                    .expect("Failed to create descriptor set layout!")
            })
        } else {
            None
        };

        (shader_set, model_set)
    }

    pub(crate) fn get_model_bind_index(&self) -> u32 {
        if self.shader_descriptor_set.is_none() {
            0
        } else {
            1
        }
    }
}

impl<S: Signature> Drop for Shader<S> {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = &crate::graphics::DEVICE {
                device.destroy_pipeline_layout(self.pipeline_layout, None);
                device.destroy_pipeline(self.pipeline, None);
                if let Some((set_layout, _)) = self.shader_descriptor_set {
                    device.destroy_descriptor_set_layout(set_layout, None);
                }
                if let Some(set_layout) = self.model_descriptor_set_layout {
                    device.destroy_descriptor_set_layout(set_layout, None);
                }
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
            crate::get_device().create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create shader module!")
        }
    }

    fn create_graphics_pipeline<S: Signature>(&self, shader_stages: &[vk::PipelineShaderStageCreateInfo],
        shader_descriptor_set: &Option<(vk::DescriptorSetLayout, DoubleBuffered<vk::DescriptorSet>)>,
        model_descriptor_set_layout: &Option<vk::DescriptorSetLayout>,
    ) -> (vk::Pipeline, vk::PipelineLayout) {

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
            blend_enable: vk::TRUE,
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
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

        // Make sure the order of the layouts is consistent with the indices defined at the top of the file.
        let mut set_layouts = Vec::new();
        if let Some((set_layout, _)) = shader_descriptor_set {
            set_layouts.push(*set_layout);
        }
        if let Some(set_layout) = model_descriptor_set_layout {
            set_layouts.push(*set_layout);
        }

        let push_constant_size = std::mem::size_of::<S::PushConstants>();
        let push_constant_ranges = [vk::PushConstantRange {
            offset: 0,
            size: push_constant_size as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        }];
        let pipeline_layout_create_info = if push_constant_size == 0 {
            vk::PipelineLayoutCreateInfo {
                s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineLayoutCreateFlags::empty(),
                set_layout_count: set_layouts.len() as u32,
                p_set_layouts: set_layouts.as_ptr(),
                push_constant_range_count: 0,
                p_push_constant_ranges: ptr::null(),
            }
        } else {
            vk::PipelineLayoutCreateInfo {
                s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineLayoutCreateFlags::empty(),
                set_layout_count: set_layouts.len() as u32,
                p_set_layouts: set_layouts.as_ptr(),
                push_constant_range_count: push_constant_ranges.len() as u32,
                p_push_constant_ranges: push_constant_ranges.as_ptr(),
            }
        };
        let pipeline_layout = unsafe {
            crate::get_device().create_pipeline_layout(&pipeline_layout_create_info, None)
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
            crate::get_device().create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphic_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline!.")
        };

        (graphics_pipelines[0], pipeline_layout)
    }
}