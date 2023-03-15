mod setup;

use ash::extensions::{ext, khr};
use ash::vk;

use crate::renderer::base::setup::{
    create_swapchain, create_swapchain_image_views, get_surface_capabilities, get_surface_extent,
    get_swapchain_images,
};

use self::setup::{
    create_allocator, create_debug_call_back, create_instance, create_logical_device,
    create_surface, get_physical_device, get_present_mode, get_queue_family,
    get_required_instance_extensions, get_surface_format,
};

pub struct RenderBase {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub surface_loader: khr::Surface,
    pub swapchain_loader: khr::Swapchain,
    #[cfg(debug_assertions)]
    pub debug_utils_loader: ext::DebugUtils,
    #[cfg(debug_assertions)]
    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub queue_family: u32,
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub allocator: gpu_allocator::vulkan::Allocator,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl RenderBase {
    pub fn new(window: &winit::window::Window) -> Result<Self, String> {
        let entry = ash::Entry::linked();
        let instance_extensions = get_required_instance_extensions(window).unwrap();
        let device_extensions = vec![ash::extensions::khr::Swapchain::name()];

        let instance = create_instance(&entry, &instance_extensions);

        #[cfg(debug_assertions)]
        let debug_utils_loader = ext::DebugUtils::new(&entry, &instance);
        #[cfg(debug_assertions)]
        let debug_call_back = create_debug_call_back(&debug_utils_loader)?;

        let surface_loader = khr::Surface::new(&entry, &instance);

        let surface = create_surface(&entry, &instance, window)?;

        let physical_device = get_physical_device(&instance, &device_extensions)?;
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let surface_format = get_surface_format(physical_device, &surface_loader, surface)?;
        let present_mode = get_present_mode(physical_device, &surface_loader, surface)?;
        let queue_family = get_queue_family(&instance, physical_device, &surface_loader, surface)?;

        let device =
            create_logical_device(&instance, physical_device, queue_family, &device_extensions)?;

        let queue = unsafe { device.get_device_queue(queue_family, 0) };

        let allocator = create_allocator(&instance, &device, physical_device)?;

        let swapchain_loader = khr::Swapchain::new(&instance, &device);

        let resize_data = resize_internal(
            window,
            &device,
            &surface_loader,
            &swapchain_loader,
            physical_device,
            vk::SwapchainKHR::null(),
            surface,
            &surface_format,
            present_mode,
            &vec![],
        )?;

        Ok(RenderBase {
            entry,
            instance,
            surface,
            surface_loader,

            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_call_back,

            physical_device,
            physical_device_properties,
            surface_format,
            present_mode,
            queue_family,
            queue,
            allocator,
            surface_capabilities: resize_data.surface_capabilities,
            surface_extent: resize_data.surface_extent,
            swapchain: resize_data.swapchain,
            swapchain_images: resize_data.swapchain_images,
            swapchain_image_views: resize_data.swapchain_image_views,
            swapchain_loader,
            device,
        })
    }

    #[inline]
    pub fn resize(&mut self, window: &winit::window::Window) -> Result<(), String> {
        let resize_data = resize_internal(
            window,
            &self.device,
            &self.surface_loader,
            &self.swapchain_loader,
            self.physical_device,
            self.swapchain,
            self.surface,
            &self.surface_format,
            self.present_mode,
            &self.swapchain_image_views,
        )?;

        self.surface_capabilities = resize_data.surface_capabilities;
        self.surface_extent = resize_data.surface_extent;
        self.swapchain = resize_data.swapchain;
        self.swapchain_images = resize_data.swapchain_images;
        self.swapchain_image_views = resize_data.swapchain_image_views;

        Ok(())
    }
}

struct ResizeResult {
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    surface_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
}

fn resize_internal(
    window: &winit::window::Window,
    device: &ash::Device,
    surface_loader: &ash::extensions::khr::Surface,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    physical_device: vk::PhysicalDevice,
    old_swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    surface_format: &vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    old_swapchain_image_views: &Vec<vk::ImageView>,
) -> Result<ResizeResult, String> {
    unsafe {
        device.device_wait_idle().unwrap();
    }

    let surface_capabilities = get_surface_capabilities(surface_loader, physical_device, surface)?;
    let surface_extent = get_surface_extent(window, &surface_capabilities);

    let swapchain = create_swapchain(
        old_swapchain,
        surface,
        &surface_capabilities,
        surface_format,
        surface_extent,
        present_mode,
        swapchain_loader,
    )?;

    let swapchain_images = get_swapchain_images(swapchain_loader, swapchain)?;

    if !old_swapchain_image_views.is_empty() {
        for &image_view in old_swapchain_image_views {
            unsafe {
                device.destroy_image_view(image_view, None);
            };
        }
    }

    let swapchain_image_views =
        create_swapchain_image_views(device, &swapchain_images, surface_format)?;

    Ok(ResizeResult {
        surface_capabilities,
        surface_extent,
        swapchain,
        swapchain_images,
        swapchain_image_views,
    })
}
