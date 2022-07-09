use ash::vk;
use std::ffi::CString;
use std::os::raw::c_char;

pub(crate) struct DeviceExtension {
    pub(crate) names: &'static [&'static str],
}

impl DeviceExtension {
    pub fn get_extensions_raw_names(&self) -> Vec<*const c_char> {
        self.names.iter().map(|n| { 
            let cstr = CString::new(*n).unwrap();
            let cstr = std::mem::ManuallyDrop::new(cstr);
            cstr.as_ptr() as *const c_char }).collect()
    }
}

pub(crate) struct SurfaceStuff {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,

    pub screen_width: u32,
    pub screen_height: u32,
}
pub(crate) struct SwapChainStuff {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub(crate) struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub(crate) struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub(crate) struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

pub(crate) struct RenderData {
    pub(crate) wait_semaphores: [vk::Semaphore; 1],
    pub(crate) wait_stages: [vk::PipelineStageFlags; 1],
    pub(crate) signal_semaphores: [vk::Semaphore; 1],
    pub(crate) buffer_index: usize,
    pub(crate) submit_infos: Vec<vk::SubmitInfo>,
}