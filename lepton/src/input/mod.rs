use ash::vk;
use std::sync::mpsc::Sender;
use std::ptr;
use std::cmp::max;

mod input_types;

use crate::graphics::{Graphics, Deletable, DoubleBuffered};
use crate::shader::Data;
pub use input_types::*;

const BLANK_WIDTH: u32 = 4;
const BLANK_HEIGHT: u32 = 4;
const ALMOST_ONE: f32 = 0.999;


/// An outward-facing UBO wrapper. One per UBO
pub enum Input {
    Buffer {
        uniform_buffers: DoubleBuffered<vk::Buffer>,
        uniform_buffers_memory: DoubleBuffered<vk::DeviceMemory>,
        size: u64,

        delete_sender: Sender<Deletable>,
    },

    Texture {
        texture_image: vk::Image,
        texture_image_view: vk::ImageView,
        texture_image_memory: vk::DeviceMemory,
        texture_sampler: vk::Sampler,
        _mip_levels: u32,

        delete_sender: Sender<Deletable>,
    }
}

impl Input {
    pub fn new_buffer<D: Data>(graphics: &Graphics) -> Self {
        let size = std::mem::size_of::<D>() as u64;
        let (uniform_buffers, uniform_buffers_memory) = Graphics::create_uniform_buffers(
            graphics.memory_properties,
            graphics.swapchain_images.len(),
            size
        );

        Self::Buffer {
            uniform_buffers,
            uniform_buffers_memory,
            size,
            delete_sender: graphics.delete_sender.clone(),
        }
    }

    pub fn new_texture<'a>(graphics: &Graphics, texture_type: TextureType<'a>) -> Self {
        graphics.check_mipmap_support(vk::Format::R8G8B8A8_SRGB);
        let (image, format, mipmap) = match texture_type {
            TextureType::Mipmap(b) => (Some(TextureType::to_image(b)), vk::Format::R8G8B8A8_SRGB, true),
            TextureType::Transparency(b) => (Some(TextureType::to_image(b)), vk::Format::R8G8B8A8_SRGB, false),
            TextureType::Monochrome(b) => (Some(TextureType::to_image(b)), vk::Format::R8_SRGB, false),
            TextureType::Blank => (None, vk::Format::R8_SRGB, false),
        };

        let (
            texture_image,
            texture_image_memory,
            mip_levels,
            texture_image_view,
            texture_sampler
        ) = if format != vk::Format::UNDEFINED {
            let (texture_image, texture_image_memory, mip_levels) = graphics.create_texture_image(image, format, mipmap);
            let texture_image_view = graphics.create_texture_image_view(texture_image, format, mip_levels);
            let texture_sampler = graphics.create_texture_sampler(mip_levels);
            (
                texture_image,
                texture_image_memory,
                mip_levels,
                texture_image_view,
                texture_sampler
            )
        } else {
            (
                vk::Image::null(),
                vk::DeviceMemory::null(),
                0,
                vk::ImageView::null(),
                vk::Sampler::null(),
            )
        };
        
        Self::Texture {
            _mip_levels: mip_levels,
            texture_image,
            texture_image_view,
            texture_sampler,
            texture_image_memory,
            delete_sender: graphics.delete_sender.clone(),
        }

    }

    pub fn update<D: Data>(&self, uniform: D, buffer_index: usize) {
        match &self {
            Self::Buffer{uniform_buffers_memory, size, ..} => {
                let ubos = [uniform];

                unsafe {
                    let data_ptr =crate::get_device().map_memory(uniform_buffers_memory.get(buffer_index),
                        0, *size, vk::MemoryMapFlags::empty())
                            .expect("Failed to Map Memory") as *mut D;
        
                    data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), 1);
        
                    crate::get_device().unmap_memory(uniform_buffers_memory.get(buffer_index));
                }
            },
            _ => unimplemented!()
        }
    }

    pub(crate) fn add_descriptor(&self, descriptor_sets: &DoubleBuffered<vk::DescriptorSet>, binding: u32) {
        match &self {
            Self::Buffer { uniform_buffers, size, .. } => {
                for (buffer_index, descriptor_set) in descriptor_sets.iter().enumerate() {
                    let descriptor_buffer_infos = [vk::DescriptorBufferInfo {
                        buffer: uniform_buffers.get(buffer_index),
                        offset: 0,
                        range: *size,
                    }];
        
                    let descriptor_write_sets = [vk::WriteDescriptorSet {
                        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                        p_next: ptr::null(),
                        dst_set: *descriptor_set,
                        dst_binding: binding,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_image_info: ptr::null(),
                        p_buffer_info: descriptor_buffer_infos.as_ptr(),
                        p_texel_buffer_view: ptr::null(),
                    }];
                    unsafe {
                        crate::get_device().update_descriptor_sets(&descriptor_write_sets, &[]);
                    }
                }
            },

            Self::Texture { texture_image_view, texture_sampler, .. } => {
                for descriptor_set in descriptor_sets.iter() {
                    let descriptor_image_infos = [vk::DescriptorImageInfo {
                        sampler: *texture_sampler,
                        image_view: *texture_image_view,
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    }];
        
                    let descriptor_write_sets = [vk::WriteDescriptorSet {
                        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                        p_next: ptr::null(),
                        dst_set: *descriptor_set,
                        dst_binding: binding,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: descriptor_image_infos.as_ptr(),
                        p_buffer_info: ptr::null(),
                        p_texel_buffer_view: ptr::null(),
                    }];
                    unsafe {
                        crate::get_device().update_descriptor_sets(&descriptor_write_sets, &[]);
                    }
                }
            }
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        match &self {
            Self::Buffer{uniform_buffers, uniform_buffers_memory, delete_sender, ..} => {
                for (uniform_buffer, uniform_buffer_memory) in uniform_buffers.iter().zip(uniform_buffers_memory.iter()) {
                    delete_sender.send(Deletable::Buffer(*uniform_buffer, *uniform_buffer_memory)).unwrap_or(());
                }
            },

            Self::Texture{texture_sampler, texture_image_view, texture_image, texture_image_memory, delete_sender, ..} => {
                if *texture_sampler != vk::Sampler::null() {
                    delete_sender.send(Deletable::Sampler(*texture_sampler, *texture_image_view)).unwrap_or(());
                    delete_sender.send(Deletable::Image(*texture_image, *texture_image_memory)).unwrap_or(());
                }
            }
        }
    }
}

impl Graphics {
    fn create_uniform_buffers(memory_properties: vk::PhysicalDeviceMemoryProperties, num_images: usize,
        buffer_size: u64) -> (DoubleBuffered<vk::Buffer>, DoubleBuffered<vk::DeviceMemory>) {

        let mut uniform_buffers = Vec::new();
        let mut uniform_buffers_memory = Vec::new();
    
        for _ in 0..num_images {
            let (uniform_buffer, uniform_buffer_memory) = Graphics::create_buffer(
                crate::get_device(),
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                memory_properties,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }
    
        (DoubleBuffered::new(uniform_buffers), DoubleBuffered::new(uniform_buffers_memory))
    }
    
    fn check_mipmap_support(&self, format: vk::Format) {
        let format_properties = unsafe { self.instance.get_physical_device_format_properties(self.physical_device, format) };

        let is_sample_image_filter_linear_support = format_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

        if !is_sample_image_filter_linear_support {
            panic!("Texture Image format does not support linear blitting!")
        }
    }

    fn create_texture_image(&self, image_object: Option<image::DynamicImage>,
        format: vk::Format, mipmap: bool) -> (vk::Image, vk::DeviceMemory, u32) {

        let (image_width, image_height, image_data, word_width) =  match image_object {
            Some(mut io) => {
                io = io.flipv();
                let (image_width, image_height) = (io.width(), io.height());
                let (image_data, word_width) = match format {
                    vk::Format::R8G8B8A8_SRGB => (io.to_rgba8().into_raw(), 4),
                    vk::Format::R8G8B8_SRGB => (io.to_rgb8().into_raw(), 3),
                    vk::Format::R8_SRGB => (io.to_luma8().into_raw(), 1),
                    _ => panic!("Image format {:?} is not supported.", format)
                };
                (image_width, image_height, image_data, word_width)
            },
            None => {
                assert_eq!(vk::Format::R8_SRGB, format);
                (BLANK_WIDTH, BLANK_HEIGHT, vec![255; BLANK_WIDTH as usize * BLANK_HEIGHT as usize], 1)
            }
        };
        
        let image_size =
            (::std::mem::size_of::<u8>() as u32 * image_width * image_height * word_width) as vk::DeviceSize;
        let mip_levels = match mipmap {
            true => ((::std::cmp::max(image_width, image_height) as f32)
                .log2()
                .floor() as u32)
                + 1,
            false => 1,
        };

        if image_size == 0 {
            panic!("Failed to load texture image!")
        }

        let (staging_buffer, staging_buffer_memory) = Graphics::create_buffer(
            crate::get_device(),
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            self.memory_properties,
        );

        unsafe {
            let data_ptr = crate::get_device().map_memory(staging_buffer_memory, 0, image_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            crate::get_device().unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = Graphics::create_image(
            crate::get_device(),
            image_width,
            image_height,
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            self.memory_properties,
        );
        self.transition_image_layout(
            texture_image,
            format,
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
            crate::get_device().destroy_buffer(staging_buffer, None);
            crate::get_device().free_memory(staging_buffer_memory, None);
        }

        (texture_image, texture_image_memory, mip_levels)
    }

    fn create_texture_image_view(&self, texture_image: vk::Image, format: vk::Format, mip_levels: u32) -> vk::ImageView {
        Self::create_image_view(crate::get_device(), texture_image, format, vk::ImageAspectFlags::COLOR, mip_levels)
    }

    fn create_texture_sampler(&self, _mip_levels: u32) -> vk::Sampler {
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
            crate::get_device()
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
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
            crate::get_device().cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &buffer_image_regions,
            );
        }
    
        self.end_single_time_command(&submit_queue, command_buffer);
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
            crate::get_device()
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
            crate::get_device()
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }
    
        command_buffer
    }

    fn end_single_time_command(&self, submit_queue: &vk::Queue, command_buffer: vk::CommandBuffer) {
        unsafe {
            crate::get_device()
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
            crate::get_device()
                .queue_submit(*submit_queue, &sumbit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            crate::get_device()
                .queue_wait_idle(*submit_queue)
                .expect("Failed to wait Queue idle!");
            crate::get_device().free_command_buffers(self.command_pool, &buffers_to_submit);
        }
    }
    pub(crate) fn transition_image_layout(&self, image: vk::Image, _format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: u32) {
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
            crate::get_device().cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        self.end_single_time_command(&self.graphics_queue, command_buffer);
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
                crate::get_device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
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
                crate::get_device().cmd_blit_image(
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
                crate::get_device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
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
            crate::get_device().cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );
        }
    
        self.end_single_time_command(submit_queue, command_buffer);
    } 

    pub(crate) fn copy_buffer(&self, submit_queue: &vk::Queue,src_buffer: vk::Buffer, dst_buffer: vk::Buffer, size: vk::DeviceSize) {

        let command_buffer = self.begin_single_time_command();
    
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];
    
        unsafe {
            crate::get_device().cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
        }
    
        self.end_single_time_command(submit_queue, command_buffer);
    }
}