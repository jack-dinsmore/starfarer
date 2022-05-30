mod platforms;
mod debug;
mod tools;
mod primitives;

use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use winit::event_loop::EventLoop;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;

use crate::{Control, PatternTrait, constants::*, shader::{Shader, CameraData}};
pub use primitives::*;
//use crate::vk_core::{share, structures::DeviceExtension};
use debug::ValidationInfo;

// Constants
const WINDOW_TITLE: &'static str = "29.Multi-Sampling";

pub struct Graphics {
    pub(crate) window: winit::window::Window,

    // vulkan stuff
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) surface_loader: ash::extensions::khr::Surface,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub(crate) debug_messenger: vk::DebugUtilsMessengerEXT,

    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub(crate) device: ash::Device,

    pub(crate) queue_family: QueueFamilyIndices,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,

    pub(crate) swapchain_loader: ash::extensions::khr::Swapchain,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    pub(crate) swapchain_imageviews: Vec<vk::ImageView>,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,

    pub(crate) render_pass: vk::RenderPass,
    pub(crate) ubo_layout: vk::DescriptorSetLayout,

    pub(crate) color_image: vk::Image,
    pub(crate) color_image_view: vk::ImageView,
    pub(crate) color_image_memory: vk::DeviceMemory,
    pub(crate) depth_image: vk::Image,
    pub(crate) depth_image_view: vk::ImageView,
    pub(crate) depth_image_memory: vk::DeviceMemory,
    pub(crate) msaa_samples: vk::SampleCountFlags,
    pub(crate) descriptor_pool: vk::DescriptorPool,

    pub(crate) command_pool: vk::CommandPool,
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,
    pub(crate) in_flight_fences: Vec<vk::Fence>,
    pub(crate) current_frame: usize,

    pub(crate) is_framebuffer_resized: bool,
}

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};
pub const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_PAINT_FPS_COUNTER: bool = false;




impl DeviceExtension {
    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [
            // currently just enable the Swapchain extension.
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ]
    }
}


/// Public functions
impl Graphics {
    /// Initialize the Vulkan pipeline and open the window
    pub fn new(control: &Control) -> Self {
        let window = Graphics::init_window(&control.event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = ash::Entry::linked();
        let instance = Graphics::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = Graphics::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_utils_loader, debug_messenger) = debug::setup_debug_utils(VALIDATION.is_enable, &entry, &instance);
        let physical_device = Graphics::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let msaa_samples = Graphics::get_max_usable_sample_count(&instance, physical_device);
        let physical_device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (device, queue_family) = Graphics::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_family.present_family.unwrap(), 0) };
        let swapchain_stuff = Graphics::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &queue_family);
        let swapchain_imageviews = Graphics::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = Graphics::create_render_pass(&instance, &device, physical_device, swapchain_stuff.swapchain_format, msaa_samples);
        let command_pool = Graphics::create_command_pool(&device, &queue_family);
        let descriptor_pool = Graphics::create_descriptor_pool(&device, swapchain_stuff.swapchain_images.len());
        let (color_image, color_image_view, color_image_memory) = Graphics::create_color_resources(
                &device, swapchain_stuff.swapchain_format, swapchain_stuff.swapchain_extent, &physical_device_memory_properties, msaa_samples);
        let (depth_image, depth_image_view, depth_image_memory) = Graphics::create_depth_resources(&instance, &device, physical_device,
            command_pool, graphics_queue, swapchain_stuff.swapchain_extent, &physical_device_memory_properties, msaa_samples);
        let framebuffers = Graphics::create_framebuffers(
            &device, render_pass, &swapchain_imageviews, depth_image_view, color_image_view, swapchain_stuff.swapchain_extent);
        let ubo_layout = crate::shader::create_descriptor_set_layout(&device);
        
        
        /*let (shader, shader_stages) = Shader::new(self, CameraData::new(
            swapchain_stuff.swapchain_extent.width as f32 / swapchain_stuff.swapchain_extent.height as f32));

        let (graphics_pipeline, pipeline_layout) = Graphics::create_graphics_pipeline(&device, render_pass, swapchain_stuff.swapchain_extent, ubo_layout, msaa_samples);*/
        
        let sync_objects = Graphics::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        Graphics {
            window,

            entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_utils_loader,
            debug_messenger,

            physical_device,
            memory_properties: physical_device_memory_properties,
            device,

            queue_family,
            graphics_queue,
            present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain: swapchain_stuff.swapchain,
            swapchain_format: swapchain_stuff.swapchain_format,
            swapchain_images: swapchain_stuff.swapchain_images,
            swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            framebuffers,

            ubo_layout,
            render_pass,

            color_image,
            color_image_view,
            color_image_memory,

            depth_image,
            depth_image_view,
            depth_image_memory,

            msaa_samples,

            descriptor_pool,

            command_pool,

            image_available_semaphores: sync_objects.image_available_semaphores,
            render_finished_semaphores: sync_objects.render_finished_semaphores,
            in_flight_fences: sync_objects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    /// Close the window and disable the Vulkan pipeline
    pub(crate) fn terminate(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
    }

    /// Submit redraw queue in Vulkan pipeline
    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(crate) fn draw_frame(&mut self, pattern: &dyn PatternTrait) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        pattern.update_uniform_buffer(&self, image_index as usize);

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: pattern.get_command_buffer(image_index as usize),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }
        
        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

/// Private functions
impl Graphics {
    /// Initialize the window. Panics if the window fails to create.
    fn init_window(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    /// Create the Vulkan instance. Panics if validation layers (for debugging) are not supported or if the instance fails to create.
    fn create_instance(entry: &ash::Entry, window_title: &str, is_enable_debug: bool, required_validation_layers: &Vec<&str>) -> ash::Instance {
        if is_enable_debug && debug::check_validation_layer_support(entry, required_validation_layers) == false {
            panic!("Validation layers requested, but not available!");
        }
    
        let app_name = CString::new(window_title).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            application_version: APPLICATION_VERSION,
            p_engine_name: engine_name.as_ptr(),
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION,
        };
    
        // This create info used to debug issues in vk::createInstance and vk::destroyInstance.
        let debug_utils_create_info = debug::populate_debug_messenger_create_info();
    
        // VK_EXT debug report has been requested here.
        let extension_names = platforms::required_extension_names();
    
        let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
    
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if is_enable_debug {
                layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if is_enable_debug {
                layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };
    
        let instance: ash::Instance = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Failed to create instance!")
        };
    
        instance
    }

    fn get_max_usable_sample_count(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> vk::SampleCountFlags {
        let physical_device_properties =unsafe { instance.get_physical_device_properties(physical_device) };

        let count = std::cmp::min(
            physical_device_properties.limits.framebuffer_color_sample_counts,
            physical_device_properties.limits.framebuffer_depth_sample_counts,
        );

        if count.contains(vk::SampleCountFlags::TYPE_64) {
            return vk::SampleCountFlags::TYPE_64;
        }
        if count.contains(vk::SampleCountFlags::TYPE_32) {
            return vk::SampleCountFlags::TYPE_32;
        }
        if count.contains(vk::SampleCountFlags::TYPE_16) {
            return vk::SampleCountFlags::TYPE_16;
        }
        if count.contains(vk::SampleCountFlags::TYPE_8) {
            return vk::SampleCountFlags::TYPE_8;
        }
        if count.contains(vk::SampleCountFlags::TYPE_4) {
            return vk::SampleCountFlags::TYPE_4;
        }
        if count.contains(vk::SampleCountFlags::TYPE_2) {
            return vk::SampleCountFlags::TYPE_2;
        }

        vk::SampleCountFlags::TYPE_1
    }

    fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window, screen_width: u32, screen_height: u32) -> SurfaceStuff {
        let surface = unsafe {
            platforms::create_surface(entry, instance, window).expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
    
        SurfaceStuff {
            surface_loader,
            surface,
            screen_width,
            screen_height,
        }
    }

    fn pick_physical_device(instance: &ash::Instance, surface_stuff: &SurfaceStuff, required_device_extensions: &DeviceExtension) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance.enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };
    
        let result = physical_devices.iter().find(|physical_device| {
            let is_suitable = Graphics::is_physical_device_suitable(instance, **physical_device, surface_stuff, required_device_extensions);
    
            // if is_suitable {
            //     let device_properties = instance.get_physical_device_properties(**physical_device);
            //     let device_name = super::tools::vk_to_string(&device_properties.device_name);
            //     println!("Using GPU: {}", device_name);
            // }
    
            is_suitable
        });
    
        match result {
            Some(p_physical_device) => *p_physical_device,
            None => panic!("Failed to find a suitable GPU!"),
        }
    }

    fn is_physical_device_suitable(instance: &ash::Instance, physical_device: vk::PhysicalDevice,
        surface_stuff: &SurfaceStuff, required_device_extensions: &DeviceExtension) -> bool {
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
    
        let indices = Graphics::find_queue_family(instance, physical_device, surface_stuff);
    
        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported = Graphics::check_device_extension_support(instance, physical_device, required_device_extensions);
        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support = Graphics::query_swapchain_support(physical_device, surface_stuff);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };
        let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;
    
        return is_queue_family_supported && is_device_extension_supported && is_swapchain_supported && is_support_sampler_anisotropy;
    }

    fn find_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> QueueFamilyIndices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface_stuff
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface_stuff.surface,
                    ).expect("Failed to get physical device surface support!")
            };
            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn check_device_extension_support(instance: &ash::Instance, physical_device: vk::PhysicalDevice, device_extensions: &DeviceExtension) -> bool {
        let available_extensions = unsafe {
            instance.enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        };

        let mut available_extension_names = vec![];

        for extension in available_extensions.iter() {
            let extension_name = tools::vk_to_string(&extension.extension_name);

            available_extension_names.push(extension_name);
        }

        use std::collections::HashSet;
        let mut required_extensions = HashSet::new();
        for extension in device_extensions.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return required_extensions.is_empty();
    }

    fn query_swapchain_support(physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> SwapChainSupportDetail {
        unsafe {
            let capabilities = surface_stuff
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_stuff.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_stuff
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_stuff.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_stuff
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface_stuff.surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn create_render_pass(instance: &ash::Instance, device: &ash::Device, physcial_device: vk::PhysicalDevice,
        surface_format: vk::Format, msaa_samples: vk::SampleCountFlags) -> vk::RenderPass {

        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format,
            samples: msaa_samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: Graphics::find_depth_format(instance, physcial_device),
            samples: msaa_samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let color_attachment_resolve = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let color_attachment_resolve_ref = vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: &color_attachment_resolve_ref,
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        }];

        let render_pass_attachments =
            [color_attachment, depth_attachment, color_attachment_resolve];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: vk::RenderPassCreateFlags::empty(),
            p_next: ptr::null(),
            attachment_count: render_pass_attachments.len() as u32,
            p_attachments: render_pass_attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
        };

        unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }
    }

    fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, validation: &debug::ValidationInfo,
        device_extensions: &DeviceExtension, surface_stuff: &SurfaceStuff) -> (ash::Device, QueueFamilyIndices) {
        let indices = Graphics::find_queue_family(instance, physical_device, surface_stuff);
    
        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());
    
        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }
    
        let physical_device_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE, // enable anisotropy device feature from Chapter-24.
            ..Default::default()
        };
    
        let requred_validation_layer_raw_names: Vec<CString> = validation
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
    
        let enable_extension_names = device_extensions.get_extensions_raw_names();
    
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if validation.is_enable {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if validation.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
        };
    
        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };
    
        (device, indices)
    }

    fn create_swapchain(instance: &ash::Instance, device: &ash::Device, physical_device: vk::PhysicalDevice,window: &winit::window::Window,
        surface_stuff: &SurfaceStuff, queue_family: &QueueFamilyIndices) -> SwapChainStuff {
            
        let swapchain_support = Graphics::query_swapchain_support(physical_device, surface_stuff);
    
        let surface_format = Graphics::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Graphics::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Graphics::choose_swapchain_extent(&swapchain_support.capabilities, window);
    
        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };
    
        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };
    
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface_stuff.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };
    
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };
    
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };
    
        SwapChainStuff {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        }
    }

    fn recreate_swapchain(&mut self, ) {
        panic!("Cannot recreate the swapchain yet.")
    }

    fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {

        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(available_present_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &winit::window::Window) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            use num::clamp;

            let window_size = window
                .inner_size();
            println!(
                "\t\tInner Window Size: ({}, {})",
                window_size.width, window_size.height
            );

            vk::Extent2D {
                width: clamp(
                    window_size.width as u32,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    window_size.height as u32,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_image_views(device: &ash::Device, surface_format: vk::Format, images: &Vec<vk::Image>) -> Vec<vk::ImageView> {
        let swapchain_imageviews: Vec<vk::ImageView> = images
            .iter()
            .map(|&image| {
                Graphics::create_image_view(device, image, surface_format, vk::ImageAspectFlags::COLOR, 1)
            })
            .collect();
    
        swapchain_imageviews
    }

    fn create_command_pool(device: &ash::Device, queue_families: &QueueFamilyIndices) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: queue_families.graphics_family.unwrap(),
        };
    
        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        }
    }

    fn create_color_resources(device: &ash::Device, swapchain_format: vk::Format, swapchain_extent: vk::Extent2D, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, msaa_samples: vk::SampleCountFlags) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
        let color_format = swapchain_format;

        let (color_image, color_image_memory) = Graphics::create_image(
            device,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            msaa_samples,
            color_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_memory_properties,
        );

        let color_image_view = Graphics::create_image_view(
            device,
            color_image,
            color_format,
            vk::ImageAspectFlags::COLOR,
            1,
        );

        (color_image, color_image_view, color_image_memory)
    }

    pub(crate) fn create_image_view(device: &ash::Device, image: vk::Image, format: vk::Format,
        aspect_flags: vk::ImageAspectFlags, mip_levels: u32) -> vk::ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };
    
        unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        }
    }

    pub(crate) fn create_image(device: &ash::Device, width: u32, height: u32, mip_levels: u32, num_samples: vk::SampleCountFlags, format: vk::Format, tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties) -> (vk::Image, vk::DeviceMemory) {
        let image_create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            mip_levels,
            array_layers: 1,
            samples: num_samples,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        };
    
        let texture_image = unsafe {
            device
                .create_image(&image_create_info, None)
                .expect("Failed to create Texture Image!")
        };
    
        let image_memory_requirement = unsafe { device.get_image_memory_requirements(texture_image) };
        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: image_memory_requirement.size,
            memory_type_index: Graphics::find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
                device_memory_properties,
            ),
        };
    
        let texture_image_memory = unsafe {
            device
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };
    
        unsafe {
            device
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        }
    
        (texture_image, texture_image_memory)
    }

    fn find_memory_type(type_filter: u32, required_properties: vk::MemoryPropertyFlags, mem_properties: &vk::PhysicalDeviceMemoryProperties) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }
    
        panic!("Failed to find suitable memory type!")
    }
    
    fn create_depth_resources(instance: &ash::Instance, device: &ash::Device,  physical_device: vk::PhysicalDevice, _command_pool: vk::CommandPool, _submit_queue: vk::Queue,
        swapchain_extent: vk::Extent2D, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, msaa_samples: vk::SampleCountFlags) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
        let depth_format = Graphics::find_depth_format(instance, physical_device);
        let (depth_image, depth_image_memory) = Graphics::create_image(
            device,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            msaa_samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_memory_properties,
        );
        let depth_image_view = Graphics::create_image_view(
            device,
            depth_image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
            1,
        );
    
        (depth_image, depth_image_view, depth_image_memory)
    }

    fn find_depth_format(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> vk::Format {
        Graphics::find_supported_format(
            instance,
            physical_device,
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(instance: &ash::Instance, physical_device: vk::PhysicalDevice, candidate_formats: &[vk::Format], tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags) -> vk::Format {
        for &format in candidate_formats.iter() {
            let format_properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, format) };
            if tiling == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(features)
            {
                return format.clone();
            } else if tiling == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(features)
            {
                return format.clone();
            }
        }
    
        panic!("Failed to find supported format!")
    }

    fn create_framebuffers(device: &ash::Device, render_pass: vk::RenderPass, swapchain_image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView, color_image_view: vk::ImageView, swapchain_extent: vk::Extent2D) -> Vec<vk::Framebuffer> {
        let mut framebuffers = vec![];

        for &image_view in swapchain_image_views.iter() {
            let attachments = [color_image_view, depth_image_view, image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: swapchain_extent.width,
                height: swapchain_extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            framebuffers.push(framebuffer);
        }

        framebuffers
    }

    pub(crate) fn create_buffer(device: &ash::Device, size: vk::DeviceSize, usage: vk::BufferUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };
    
        let buffer = unsafe {
            device
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create Vertex Buffer")
        };
    
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type = Graphics::find_memory_type(
            mem_requirements.memory_type_bits,
            required_memory_properties,
            device_memory_properties,
        );
    
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: memory_type,
        };
    
        let buffer_memory = unsafe {
            device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate vertex buffer memory!")
        };
    
        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer");
        }
    
        (buffer, buffer_memory)
    }

    fn create_descriptor_pool(device: &ash::Device, swapchain_images_size: usize) -> vk::DescriptorPool {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                // transform descriptor pool
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain_images_size as u32,
            },
            vk::DescriptorPoolSize {
                // sampler descriptor pool
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swapchain_images_size as u32,
            },
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: swapchain_images_size as u32,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        }
    }

    fn create_sync_objects(device: &ash::Device, max_frame_in_flight: usize) -> SyncObjects {
        let mut sync_objects = SyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            inflight_fences: vec![],
        };
    
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };
    
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };
    
        for _ in 0..max_frame_in_flight {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");
    
                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.inflight_fences.push(inflight_fence);
            }
        }
    
        sync_objects
    }
}