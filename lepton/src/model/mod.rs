use anyhow::{Result, bail};
use ash::vk;
use std::path::Path;
use std::ptr;
use std::cmp::max;

mod primitives;
pub use primitives::{VertexV3};
use crate::{Graphics, Unload, UnfinishedPattern};
use crate::shader::{Shader, ShaderData};

pub struct Model<D: ShaderData> {
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    num_indices: u32,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer_memory: vk::DeviceMemory,

    mip_levels: u32,
    texture_image: vk::Image,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    texture_image_memory: vk::DeviceMemory,
    descriptor_sets: Vec<vk::DescriptorSet>,

    phantom: std::marker::PhantomData<D>,
}

impl<D: ShaderData> Model<D> {
    pub fn new(graphics: &Graphics, pattern: &UnfinishedPattern<D>, obj_path: &Path, texture_path: &Path) -> Result<Self> {
        let (vertices, indices) = Self::get_data(obj_path)?;

        graphics.check_mipmap_support(vk::Format::R8G8B8A8_SRGB);
        let (texture_image, texture_image_memory, mip_levels) = graphics.create_texture_image(texture_path);
        let texture_image_view = graphics.create_texture_image_view(texture_image, mip_levels);
        let texture_sampler = graphics.create_texture_sampler(mip_levels);
        let (vertex_buffer, vertex_buffer_memory) = graphics.create_vertex_buffer(&vertices);
        let (index_buffer, index_buffer_memory) = graphics.create_index_buffer(&indices);

        let descriptor_sets = graphics.create_descriptor_sets(&pattern.shader, texture_image_view, texture_sampler);
            
        let model = Model::<D> {
            vertex_buffer,
            index_buffer,
            vertex_buffer_memory,
            index_buffer_memory,
            num_indices: indices.len() as u32,

            mip_levels,
            texture_image,
            texture_image_view,
            texture_sampler,
            texture_image_memory,

            descriptor_sets,
            phantom: std::marker::PhantomData,
        };

        pattern.render(graphics, &model);

        Ok(model)
    }

    pub(crate) fn render(&self, graphics: &Graphics, pipeline_layout: &vk::PipelineLayout, command_buffer: &vk::CommandBuffer, frame_index: usize) {
        let vertex_buffers = [self.vertex_buffer];
        let offsets = [0_u64];
        let descriptor_sets_to_bind = [self.descriptor_sets[frame_index]];

        unsafe {
            graphics.device.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
            graphics.device.cmd_bind_index_buffer(*command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);
            graphics.device.cmd_bind_descriptor_sets(*command_buffer, vk::PipelineBindPoint::GRAPHICS,
                *pipeline_layout, 0, &descriptor_sets_to_bind, &[]);

            graphics.device.cmd_draw_indexed(*command_buffer, self.num_indices, 1, 0, 0, 0);
        }
    }

    fn get_data(path: &Path) -> Result<(Vec<VertexV3>,Vec<u32>)> {
        let model_obj = match tobj::load_obj(path, &tobj::LoadOptions{single_index: true, ..Default::default()}) {
            Ok(m) => m,
            Err(_) => bail!("Failed to load model object {}", path.display())
        };

        let mut vertices = vec![];
        let mut indices = vec![];
    
        let (models, _) = model_obj;
        for m in models.iter() {
            let mesh = &m.mesh;
    
            if mesh.texcoords.len() == 0 {
                bail!("Missing texture coordinates for model {}", path.display());
            }
    
            let total_vertices_count = mesh.positions.len() / 3;
            for i in 0..total_vertices_count {
                let vertex = VertexV3 {
                    pos: [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                        1.0,
                    ],
                    color: [1.0, 1.0, 1.0, 1.0],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }
    
            indices = mesh.indices.clone();
        }
    
        Ok((vertices, indices))
    }
}

impl<D: ShaderData> Unload for Model<D> {
    fn unload(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.index_buffer, None);
            device.free_memory(self.index_buffer_memory, None);

            device.destroy_buffer(self.vertex_buffer, None);
            device.free_memory(self.vertex_buffer_memory, None);

            device.destroy_sampler(self.texture_sampler, None);
            device.destroy_image_view(self.texture_image_view, None);
        
            device.destroy_image(self.texture_image, None);
            device.free_memory(self.texture_image_memory, None);
        }
    }
}

impl Graphics {
    fn check_mipmap_support(&self, format: vk::Format) {
        let format_properties = unsafe { self.instance.get_physical_device_format_properties(self.physical_device, format) };

        let is_sample_image_filter_linear_support = format_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

        if is_sample_image_filter_linear_support == false {
            panic!("Texture Image format does not support linear blitting!")
        }
    }

    fn create_texture_image(&self, texture_path: &Path) -> (vk::Image, vk::DeviceMemory, u32) {
        let mut image_object = image::open(texture_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_data = image_object.to_rgba8().into_raw(); // Altered from the tutorial. May be wrong for different image formats
        let image_size =
            (::std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let mip_levels = ((::std::cmp::max(image_width, image_height) as f32)
            .log2()
            .floor() as u32)
            + 1;

        if image_size <= 0 {
            panic!("Failed to load texture image!")
        }

        let (staging_buffer, staging_buffer_memory) = Graphics::create_buffer(
            &self.device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &self.memory_properties,
        );

        unsafe {
            let data_ptr = self.device.map_memory(staging_buffer_memory, 0, image_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            self.device.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = Graphics::create_image(
            &self.device,
            image_width,
            image_height,
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &self.memory_properties,
        );

        self.transition_image_layout(
            &self.graphics_queue,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
        );

        self.copy_buffer_to_image(
            self.graphics_queue,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
        );

        self.generate_mipmaps(
            &self.graphics_queue,
            texture_image,
            image_width,
            image_height,
            mip_levels,
        );

        unsafe {
            self.device.destroy_buffer(staging_buffer, None);
            self.device.free_memory(staging_buffer_memory, None);
        }

        (texture_image, texture_image_memory, mip_levels)
    }

    fn create_texture_image_view(&self, texture_image: vk::Image, mip_levels: u32) -> vk::ImageView {
        Self::create_image_view(&self.device, texture_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR, mip_levels)
    }

    fn create_texture_sampler(&self, mip_levels: u32) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            min_lod: 0.0,
            max_lod: 0.0,
            mip_lod_bias: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            anisotropy_enable: vk::TRUE,
            unnormalized_coordinates: vk::FALSE,
        };
    
        unsafe {
            self.device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }

    fn create_vertex_buffer<T>(&self, data: &[T]) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            &self.device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &self.memory_properties,
        );

        unsafe {
            let data_ptr = self.device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut T;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            self.device.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            &self.device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &self.memory_properties,
        );

        self.copy_buffer(&self.graphics_queue, staging_buffer, vertex_buffer, buffer_size);

        unsafe {
            self.device.free_memory(staging_buffer_memory, None);
            self.device.destroy_buffer(staging_buffer, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }

    fn create_index_buffer(&self, data: &[u32]) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(&self.device, buffer_size, vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, &self.memory_properties);
            
        unsafe {
            let data_ptr = self.device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            self.device.unmap_memory(staging_buffer_memory);
        }
        
        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            &self.device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &self.memory_properties,
        );

        self.copy_buffer(&self.graphics_queue, staging_buffer, index_buffer, buffer_size);

        unsafe {
            self.device.destroy_buffer(staging_buffer, None);
            self.device.free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
    }

    fn create_descriptor_sets<D: ShaderData>(&self, shader: &Shader<D>, texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler) -> Vec<vk::DescriptorSet> {

        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..self.swapchain_images.len() {
            layouts.push(self.ubo_layout);
        }
    
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: self.descriptor_pool,
            descriptor_set_count: self.swapchain_images.len() as u32,
            p_set_layouts: layouts.as_ptr(),
        };
    
        let descriptor_sets = unsafe {
            self.device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };
    
        let descriptor_image_infos = [vk::DescriptorImageInfo {
            sampler: texture_sampler,
            image_view: texture_image_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }];
        let descriptor_buffer_infos = (0..descriptor_sets.len()).map(|i| shader.get_uniform_descriptor_buffer_info(i))
            .collect::<Vec<Vec<vk::DescriptorBufferInfo>>>();
        for (i, &descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_write_sets = [
                vk::WriteDescriptorSet {
                    // transform uniform
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descritptor_set,
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info: ptr::null(),
                    p_buffer_info: descriptor_buffer_infos[i].as_ptr(),
                    p_texel_buffer_view: ptr::null(),
                },
                vk::WriteDescriptorSet {
                    // sampler uniform
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descritptor_set,
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    p_image_info: descriptor_image_infos.as_ptr(),
                    p_buffer_info: ptr::null(),
                    p_texel_buffer_view: ptr::null(),
                },
            ];
    
            unsafe {
                self.device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }
    
        descriptor_sets
    }

    fn copy_buffer_to_image(&self, submit_queue: vk::Queue, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32) {
        let command_buffer = self.begin_single_time_command();
    
        let buffer_image_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            buffer_offset: 0,
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];
    
        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &buffer_image_regions,
            );
        }
    
        self.end_single_time_command(&submit_queue, command_buffer);
    }

    fn transition_image_layout(&self, submit_queue: &vk::Queue, image: vk::Image, _format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: u32) {
        let command_buffer = self.begin_single_time_command();

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask =
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        self.end_single_time_command(submit_queue, command_buffer);
    }

    fn generate_mipmaps(&self, submit_queue: &vk::Queue, image: vk::Image, tex_width: u32, tex_height: u32, mip_levels: u32) {
        let command_buffer = self.begin_single_time_command();
    
        let mut image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };
    
        let mut mip_width = tex_width as i32;
        let mut mip_height = tex_height as i32;
    
        for i in 1..mip_levels {
            image_barrier.subresource_range.base_mip_level = i - 1;
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;
    
            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }
    
            let blits = [vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1,
                    },
                ],
            }];
    
            unsafe {
                self.device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                );
            }
    
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
    
            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }
    
            mip_width = max(mip_width / 2, 1);
            mip_height = max(mip_height / 2, 1);
        }
    
        image_barrier.subresource_range.base_mip_level = mip_levels - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
    
        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }
    
        self.end_single_time_command(submit_queue, command_buffer);
    } 

    fn begin_single_time_command(&self) -> vk::CommandBuffer{
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };
    
        let command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }[0];
    
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };
    
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }
    
        command_buffer
    }

    fn end_single_time_command(&self, submit_queue: &vk::Queue, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    
        let buffers_to_submit = [command_buffer];
    
        let sumbit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: buffers_to_submit.as_ptr(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }];
    
        unsafe {
            self.device
                .queue_submit(*submit_queue, &sumbit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            self.device
                .queue_wait_idle(*submit_queue)
                .expect("Failed to wait Queue idle!");
            self.device.free_command_buffers(self.command_pool, &buffers_to_submit);
        }
    }

    pub(crate) fn copy_buffer(&self, submit_queue: &vk::Queue,src_buffer: vk::Buffer, dst_buffer: vk::Buffer, size: vk::DeviceSize) {

        let command_buffer = self.begin_single_time_command();
    
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];
    
        unsafe {
            self.device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
        }
    
        self.end_single_time_command(submit_queue, command_buffer);
    }
}