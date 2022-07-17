mod platforms;
mod debug;
mod primitives;

use ash::vk;
use winit::event_loop::EventLoop;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::mpsc::{Receiver, Sender};
use rustc_hash::FxHashMap;
use cgmath::{Vector3, Matrix3};

use primitives::*;
pub use crate::backend::RenderTask;
use crate::{backend::Backend, constants::*, shader, model::DrawState, physics::Object};
use debug::ValidationInfo;

pub(crate) type GraphicsData = FxHashMap<Object, GraphicsInnerData>;

pub(crate) struct GraphicsInnerData {
    pub push_constants: shader::builtin::ObjectPushConstants,
    pub pos: Vector3<f32>,
}

#[derive(Clone)]
pub(crate) enum Deletable {
    Buffer(vk::Buffer, vk::DeviceMemory),
    Sampler(vk::Sampler, vk::ImageView),
    DescriptorSets(Vec<vk::DescriptorSet>),
    Image(vk::Image, vk::DeviceMemory),
}

pub(crate) static mut DEVICE: Option<ash::Device> = None;


/// Gets the device. Can only be used after the device's creation and before Graphics is dropped.
pub(crate) fn get_device() -> &'static ash::Device { unsafe {match &DEVICE { Some(d) => d, None => panic!("Device was none")}}}

pub struct Graphics {
    pub(crate) window: winit::window::Window,

    // vulkan stuff
    pub(crate) _entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) surface_loader: ash::extensions::khr::Surface,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) _debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub(crate) _debug_messenger: vk::DebugUtilsMessengerEXT,

    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) memory_properties: vk::PhysicalDeviceMemoryProperties,

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

    pub(crate) swapchain_current_version: u32,
    pub(crate) swapchain_ideal_version: u32,
    pub(crate) window_width: u32,
    pub(crate) window_height: u32,
    pub(crate) mouse_position: (f32, f32),
    input_types: Vec<shader::InputType>,

    command_buffers: Vec<vk::CommandBuffer>,

    graphics_data_receiver: Receiver<GraphicsData>,
    delete_receiver: Receiver<Deletable>,
    delete_queue: Vec<Vec<Deletable>>,
    pub(crate) delete_sender: Sender<Deletable>,
    pub(crate) object_models: FxHashMap<Object, Vec<DrawState>>,
    last_graphics_data: FxHashMap<Object, GraphicsInnerData>,

    #[cfg(target_os = "macos")]
    pub(crate) last_delta: (f64, f64),
}


const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

#[cfg(target_os = "macos")]
pub(crate) const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: &["VK_KHR_swapchain", "VK_KHR_portability_subset"],
};

#[cfg(not(target_os = "macos"))]
pub(crate) const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: &["VK_KHR_swapchain"],
};


pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Public functions
impl Graphics {
    /// Initialize the Vulkan pipeline and open the window
    pub fn new(backend: &mut Backend, window_title: &'static str, window_width: u32, window_height: u32, center_cursor: bool,
        input_types: Vec<shader::InputType>, num_shaders: usize) -> Self {

        let (delete_sender, delete_receiver) = std::sync::mpsc::channel();

        let window = Graphics::init_window(&backend.event_loop, window_title, window_width, window_height);

        // Create basic Vulkan stuff
        let entry = ash::Entry::linked();
        let instance = Graphics::create_instance(&entry, window_title, VALIDATION.is_enable, VALIDATION.required_validation_layers.as_ref());
        let surface_stuff = Graphics::create_surface(&entry, &instance, &window, window_width, window_height);
        let (_debug_utils_loader, _debug_messenger) = debug::setup_debug_utils(VALIDATION.is_enable, &entry, &instance);
        let physical_device = Graphics::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let msaa_samples = Graphics::get_max_usable_sample_count(&instance, physical_device);
        let physical_device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (device, queue_family) = Graphics::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        unsafe { DEVICE = Some(device); }
        let graphics_queue = unsafe { get_device().get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { get_device().get_device_queue(queue_family.present_family.unwrap(), 0) };

        // Create swapchain
        let swapchain_stuff = Graphics::create_swapchain(&instance, get_device(), physical_device, &window, &surface_stuff, &queue_family);
        let swapchain_imageviews = Graphics::create_image_views(get_device(), swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = Graphics::create_render_pass(&instance, get_device(), physical_device, swapchain_stuff.swapchain_format, msaa_samples);
        let command_pool = Graphics::create_command_pool(get_device(), &queue_family);
        let descriptor_pool = Graphics::create_descriptor_pool(get_device(), swapchain_stuff.swapchain_images.len(), input_types.len(), num_shaders);
        let (color_image, color_image_view, color_image_memory) = Graphics::create_color_resources(
                get_device(), swapchain_stuff.swapchain_format, swapchain_stuff.swapchain_extent, physical_device_memory_properties, msaa_samples);
        let (depth_image, depth_image_view, depth_image_memory) = Graphics::create_depth_resources(&instance, get_device(), physical_device,
            swapchain_stuff.swapchain_extent, physical_device_memory_properties, msaa_samples);
        let framebuffers = Graphics::create_framebuffers(
            get_device(), render_pass, &swapchain_imageviews, depth_image_view, color_image_view, swapchain_stuff.swapchain_extent);
        let sync_objects = Graphics::create_sync_objects(get_device(), MAX_FRAMES_IN_FLIGHT);

        if center_cursor {
            window.set_cursor_position(winit::dpi::PhysicalPosition{ x: window_width / 2, y: window_height / 2}).expect("Could not set cursor pos");
            window.set_cursor_visible(false);
        }

        let command_buffers = Self::allocate_command_buffers(command_pool, framebuffers.len() as u32);

        let graphics_data_receiver = match backend.graphics_data_receiver.take() {
            Some(r) => r,
            None => panic!("Someone picked up the graphics data receiver")
        };

        let delete_queue = vec![Vec::new(); framebuffers.len()];

        Graphics {
            window,
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            _debug_utils_loader,
            _debug_messenger,
            physical_device,
            memory_properties: physical_device_memory_properties,
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

            swapchain_current_version: 0,
            swapchain_ideal_version: 0,
            window_width,
            window_height,
            input_types,
            mouse_position: (0.0, 0.0),
            command_buffers,

            graphics_data_receiver,
            object_models: FxHashMap::default(),
            last_graphics_data: FxHashMap::default(),
            delete_sender,
            delete_receiver,
            delete_queue,

            #[cfg(target_os = "macos")]
            last_delta: (0.0, 0.0),
        }
    }

    /// Close the window and disable the Vulkan pipeline
    pub(crate) fn terminate(&self) {
        unsafe {
            get_device()
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
    }

    /// Submit redraw queue in Vulkan pipeline
    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(crate) fn add_model(&mut self, object: Object, draw_state: DrawState) {
        match self.object_models.get_mut(&object) {
            Some(v) => { v.push(draw_state); },
            None => { self.object_models.insert(object, vec![draw_state]); },
        };
    }

    pub(crate) fn record(&mut self, buffer_index: usize, actions: Vec<RenderTask>) {
        unsafe {
            crate::get_device().reset_command_buffer(
                self.command_buffers[buffer_index],
                vk::CommandBufferResetFlags::empty(),
            ).expect("Resetting the command buffer failed");
        }

        self.delete_all(buffer_index);

        self.begin_command_buffer(self.command_buffers[buffer_index], buffer_index);
        let mut pipeline_layout = None;

        for action in actions.iter() {
            match action {
                RenderTask::LoadShader(s) => {
                    unsafe { crate::get_device().cmd_bind_pipeline(
                        self.command_buffers[buffer_index],
                        vk::PipelineBindPoint::GRAPHICS,
                         s.get_pipeline());
                    }
                    pipeline_layout = Some(s.get_pipeline_layout());
                },
                RenderTask::DrawObject(o) => {
                    if let Some(draw_states) = self.object_models.get(o) {
                        if let Some(d) = self.last_graphics_data.get(o) {
                            for state in draw_states {
                                match state {
                                    DrawState::Standard(model) => {
                                        let bytes = crate::tools::struct_as_bytes(&d.push_constants);
                                        model.render(pipeline_layout.expect("You must first load a shader"),
                                        self.command_buffers[buffer_index], buffer_index, Some(bytes))
                                    },
                                    DrawState::Offset(model, matrix) => {
                                        let new_pc = d.push_constants.model * matrix;
                                        let bytes = crate::tools::struct_as_bytes(&new_pc);
                                        model.render(pipeline_layout.expect("You must first load a shader"),
                                        self.command_buffers[buffer_index], buffer_index, Some(bytes));
                                    },
                                };
                            }
                        }
                    }
                },
                RenderTask::DrawModelWithObject(o, m) => {
                    if let Some(d) = self.last_graphics_data.get(o) {
                        let bytes = crate::tools::struct_as_bytes(&d.push_constants);
                        m.render(pipeline_layout.expect("You must first load a shader"),
                            self.command_buffers[buffer_index], buffer_index, Some(bytes));
                    }
                },
                RenderTask::DrawModel(m) => {
                    m.render(pipeline_layout.expect("You must first load a shader"),
                        self.command_buffers[buffer_index], buffer_index, None);
                },
                RenderTask::DrawUI(u) => {
                    u.render(pipeline_layout.expect("You must first load a shader"), 
                        self.command_buffers[buffer_index], buffer_index);
                },
                RenderTask::ClearDepthBuffer => {
                    unsafe { crate::get_device().cmd_clear_attachments(
                        self.command_buffers[buffer_index],
                        &[vk::ClearAttachment {
                            aspect_mask: vk::ImageAspectFlags::DEPTH,
                            color_attachment: 0,
                            clear_value: CLEAR_VALUES[1],
                        }],
                        &[vk::ClearRect{
                            rect: vk::Rect2D {
                                offset: vk::Offset2D{ x: 0, y: 0},
                                extent: vk::Extent2D{ width: self.window_width, height: self.window_height}
                            },
                            base_array_layer: 0,
                            layer_count: 1,
                        }]);
                    }
                }
            }
        }

        self.end_command_buffer(self.command_buffers[buffer_index]);
    }

    pub(crate) fn begin_frame(&mut self) -> Option<RenderData> {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            get_device()
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (buffer_index, _) = unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(buffer_index) => buffer_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return None;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        unsafe {
            get_device()
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");
        }

        Some(RenderData {
            wait_semaphores,
            wait_stages,
            signal_semaphores,
            buffer_index: buffer_index as usize,
            submit_infos: Vec::new(),
        })
    }

    pub(crate) fn render(&mut self, mut render_data: RenderData) {
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

        unsafe {
            crate::get_device()
                .queue_submit(
                    self.graphics_queue,
                    &render_data.submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: render_data.signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &(render_data.buffer_index as u32),
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        };

        let must_resize = match result {
            Ok(_) => self.swapchain_ideal_version != self.swapchain_current_version,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if must_resize {
            self.swapchain_current_version = self.swapchain_ideal_version;
            self.recreate_swapchain();
        }
        
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub(crate) fn resize_framebuffer(&mut self, new_width: u32, new_height: u32) {
        self.window_width = new_width;
        self.window_height = new_height;
        self.swapchain_ideal_version += 1;
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

    pub(crate) fn create_image(device: &ash::Device, width: u32, height: u32, mip_levels: u32, num_samples: vk::SampleCountFlags,
        format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags,required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: vk::PhysicalDeviceMemoryProperties) -> (vk::Image, vk::DeviceMemory) {
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

    pub(crate) fn create_buffer(device: &ash::Device, size: vk::DeviceSize, usage: vk::BufferUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: vk::PhysicalDeviceMemoryProperties) -> (vk::Buffer, vk::DeviceMemory) {
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

    pub(crate) fn receive(&mut self) {
        if let Some(h) = receive_hash(&self.graphics_data_receiver) {
            self.last_graphics_data = h;
        }
    }

    pub fn center_cursor(&mut self) {
        self.window.set_cursor_position(winit::dpi::PhysicalPosition{ x: self.window_width / 2, y: self.window_height / 2}).expect("Could not set cursor pos)");

        #[cfg(target_os = "macos")] 
        {
            self.last_delta = (
                self.mouse_position.0 as f64 * self.window_width as f64,
                self.mouse_position.1 as f64 * self.window_height as f64
            );
        }
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    pub fn get_pos(&self, object: &Object) -> Option<Vector3<f32>> {
        if let Some(d) = self.last_graphics_data.get(object) {
            return Some(d.pos);
        }
        None
    }

    pub fn get_pos_and_rot(&self, object: &Object) -> Option<(Vector3<f32>, Matrix3<f32>)> {
        if let Some(d) = self.last_graphics_data.get(object) {
            return Some((d.pos, Matrix3::new(
                d.push_constants.rotation.x.x, d.push_constants.rotation.x.y, d.push_constants.rotation.x.z,
                d.push_constants.rotation.y.x, d.push_constants.rotation.y.y, d.push_constants.rotation.y.z,
                d.push_constants.rotation.z.x, d.push_constants.rotation.z.y, d.push_constants.rotation.z.z)));
        }
        None
    }
}

/// Private functions
impl Graphics {
    /// Initialize the window. Panics if the window fails to create.
    fn init_window(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    /// Create the Vulkan instance. Panics if validation layers (for debugging) are not supported or if the instance fails to create.
    fn create_instance(entry: &ash::Entry, window_title: &str, is_enable_debug: bool, required_validation_layers: &[&str]) -> ash::Instance {
        if is_enable_debug && !debug::check_validation_layer_support(entry, required_validation_layers) {
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

        #[cfg(target_os = "macos")]
        let create_info_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        
        #[cfg(not(target_os = "macos"))]
        let create_info_flags = vk::InstanceCreateFlags::empty();
    
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: create_info_flags,
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
            Graphics::is_physical_device_suitable(instance, **physical_device, surface_stuff, required_device_extensions)
    
            // if is_suitable {
            //     let device_properties = instance.get_physical_device_properties(**physical_device);
            //     let device_name = super::tools::vk_to_string(&device_properties.device_name);
            //     println!("Using GPU: {}", device_name);
            // }
            //is_suitable
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
    
        is_queue_family_supported && is_device_extension_supported && is_swapchain_supported && is_support_sampler_anisotropy
    }

    fn find_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> QueueFamilyIndices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family = Some(index as u32);
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
                queue_family_indices.present_family = Some(index as u32);
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }

        queue_family_indices
    }

    fn check_device_extension_support(instance: &ash::Instance, physical_device: vk::PhysicalDevice, device_extensions: &DeviceExtension) -> bool {
        #![allow(clippy::disallowed_types)]
        let available_extensions = unsafe {
            instance.enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        };

        let mut available_extension_names = vec![];

        for extension in available_extensions.iter() {
            let extension_name = crate::tools::vk_to_string(&extension.extension_name);

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

        required_extensions.is_empty()
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

        #![allow(clippy::disallowed_types)]
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
            pp_enabled_extension_names: (&enable_extension_names[..]).as_ptr(),
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

    fn recreate_swapchain(&mut self) {
        unsafe {
            get_device().device_wait_idle().expect("Failed to wait device idle!")
        };
        self.unload_self();

        let surface_suff = SurfaceStuff {
            surface_loader: self.surface_loader.clone(),
            surface: self.surface,
            screen_width: self.window_width,
            screen_height: self.window_height,
        };

        let swapchain_stuff = Graphics::create_swapchain(&self.instance, get_device(), self.physical_device, &self.window, &surface_suff, &self.queue_family);
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = Graphics::create_image_views(get_device(), self.swapchain_format, &self.swapchain_images);
        self.render_pass = Graphics::create_render_pass(&self.instance, get_device(), self.physical_device, self.swapchain_format, self.msaa_samples);
        let color_resources = Graphics::create_color_resources(get_device(), self.swapchain_format,
            self.swapchain_extent, self.memory_properties, self.msaa_samples);
        self.color_image = color_resources.0;
        self.color_image_view = color_resources.1;
        self.color_image_memory = color_resources.2;

        let depth_resources = Graphics::create_depth_resources(&self.instance, get_device(), self.physical_device,
            self.swapchain_extent, self.memory_properties, self.msaa_samples);
        self.depth_image = depth_resources.0;
        self.depth_image_view = depth_resources.1;
        self.depth_image_memory = depth_resources.2;

        self.framebuffers = Graphics::create_framebuffers(get_device(), self.render_pass, &self.swapchain_imageviews,
            self.depth_image_view, self.color_image_view, self.swapchain_extent);
    }

    fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {

        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        return *available_formats.first().unwrap();
    }

    fn choose_swapchain_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
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

    fn create_image_views(device: &ash::Device, surface_format: vk::Format, images: &[vk::Image]) -> Vec<vk::ImageView> {
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
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_families.graphics_family.unwrap(),
        };
    
        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        }
    }

    fn create_color_resources(device: &ash::Device, swapchain_format: vk::Format, swapchain_extent: vk::Extent2D,
        device_memory_properties: vk::PhysicalDeviceMemoryProperties, msaa_samples: vk::SampleCountFlags) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
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

    fn find_memory_type(type_filter: u32, required_properties: vk::MemoryPropertyFlags, mem_properties: vk::PhysicalDeviceMemoryProperties) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }
    
        panic!("Failed to find suitable memory type!")
    }
    
    fn create_depth_resources(instance: &ash::Instance, device: &ash::Device,  physical_device: vk::PhysicalDevice, swapchain_extent: vk::Extent2D, device_memory_properties: vk::PhysicalDeviceMemoryProperties,
        msaa_samples: vk::SampleCountFlags) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {

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
            if (tiling == vk::ImageTiling::LINEAR && format_properties.linear_tiling_features.contains(features))
                || (tiling == vk::ImageTiling::OPTIMAL && format_properties.optimal_tiling_features.contains(features)) {
                return format;
            }
        }
    
        panic!("Failed to find supported format!")
    }

    fn create_framebuffers(device: &ash::Device, render_pass: vk::RenderPass, swapchain_image_views: &[vk::ImageView],
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

    fn create_descriptor_pool(device: &ash::Device, swapchain_images_size: usize, num_inputs: usize, num_shaders: usize) -> vk::DescriptorPool {
        let num_uniforms = num_inputs * swapchain_images_size * num_shaders;
        let num_samplers = swapchain_images_size * num_shaders;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                // transform descriptor pool
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: num_uniforms as u32,
            },
            vk::DescriptorPoolSize {
                // sampler descriptor pool
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: num_samplers as u32,
            },
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET, //// REMOVE THIS IF DESCRIPTOR SETS ARE LATER NOT FREED
            max_sets: (num_samplers + num_uniforms) as u32,
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

    fn unload_self(&mut self) {
        unsafe {
            get_device().destroy_image(self.depth_image, None);
            get_device().destroy_image_view(self.depth_image_view, None);
            get_device().free_memory(self.depth_image_memory, None);

            get_device().destroy_image(self.color_image, None);
            get_device().destroy_image_view(self.color_image_view, None);
            get_device().free_memory(self.color_image_memory, None);

            for &framebuffer in self.framebuffers.iter() {
                get_device().destroy_framebuffer(framebuffer, None);
            }
            get_device().destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                get_device().destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
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

    /// Allocates the primary command buffer. Panics if buffer cannot be allocated.
    fn allocate_command_buffers(command_pool: vk::CommandPool, command_buffer_count: u32) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        unsafe {
            crate::get_device().allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers!")
        }
    }

    fn delete_all(&mut self, buffer_index: usize) {
        // Delete all buffes that were indicated for deletion at this frame
        for deletable in &self.delete_queue[buffer_index] {
            unsafe {
                match deletable {
                    Deletable::Buffer(b, m) => {
                        crate::get_device().destroy_buffer(*b, None);
                        crate::get_device().free_memory(*m, None);
                    },
                    Deletable::Sampler(s, v) => {
                        crate::get_device().destroy_sampler(*s, None);
                        crate::get_device().destroy_image_view(*v, None);
                    },
                    Deletable::Image(i, m) => {
                        crate::get_device().destroy_image(*i, None);
                        crate::get_device().free_memory(*m, None);
                    },
                    Deletable::DescriptorSets(v) => {
                        crate::get_device().free_descriptor_sets(self.descriptor_pool, &v).unwrap();
                    }
                }
            }
        }
        self.delete_queue[buffer_index].clear();

        // Indicate new buffers for deletion
        for deletable in self.delete_receiver.try_iter() {
            self.delete_queue[buffer_index].push(deletable);
        }
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        self.unload_self();

        unsafe {
            DEVICE = None;
        }
    }
}

pub(crate) fn receive_hash(receiver: &Receiver<GraphicsData>) -> Option<GraphicsData> {
    receiver.try_iter().last()
}