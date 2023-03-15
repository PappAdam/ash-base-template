use std::ffi::{c_char, CStr};

use ash::extensions::{ext, khr};
use ash::vk::{self};
use gpu_allocator::vulkan;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::renderer::utils::vulkan_debug_callback;

pub fn get_swapchain_images(
    swapchain_loader: &khr::Swapchain,
    swapchain: vk::SwapchainKHR,
) -> Result<Vec<vk::Image>, String> {
    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .map_err(|_| String::from("failed to get swapchain images"))?
    };

    Ok(swapchain_images)
}

pub fn get_surface_format(
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, String> {
    let formats = match unsafe {
        surface_loader.get_physical_device_surface_formats(physical_device, surface)
    } {
        Ok(formats) => formats,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface formats",
            ));
        }
    };

    for f in &formats {
        if f.format == vk::Format::B8G8R8A8_UNORM
            && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            let surface_format = vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            };

            return Ok(surface_format);
        }
    }

    Ok(formats[0])
}

pub fn get_surface_extent(
    window: &winit::window::Window,
    surface_capabilities: &vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    let window_size = window.inner_size();

    let mut surface_extent = vk::Extent2D::default();

    if surface_capabilities.current_extent.width == u32::MAX {
        surface_extent.width = std::cmp::max(
            surface_capabilities.min_image_extent.width,
            std::cmp::min(
                surface_capabilities.max_image_extent.width,
                window_size.width,
            ),
        );
        surface_extent.height = std::cmp::max(
            surface_capabilities.min_image_extent.height,
            std::cmp::min(
                surface_capabilities.max_image_extent.height,
                window_size.height,
            ),
        );
    } else {
        surface_extent = surface_capabilities.current_extent;
    }

    surface_extent
}

pub fn get_surface_capabilities(
    surface_loader: &khr::Surface,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceCapabilitiesKHR, String> {
    let surface_capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|_| String::from("failed to get physical device surface capabilities"))?
    };

    Ok(surface_capabilities)
}

pub fn get_queue_family(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<u32, String> {
    let props = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    for (ind, p) in props.iter().enumerate() {
        if p.queue_count > 0 && p.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            let present_supported = match unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    ind as u32,
                    surface,
                )
            } {
                Ok(result) => result,
                Err(_) => {
                    return Err(String::from(
                        "failed to get physical device surface_support",
                    ))
                }
            };

            if present_supported {
                return Ok(ind as u32);
            }
        }
    }

    Err(String::from(
        "failed to find graphics queue with present support",
    ))
}

pub fn get_present_mode(
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, String> {
    let modes = match unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    } {
        Ok(formats) => formats,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface present modes",
            ));
        }
    };

    if modes.is_empty() {
        return Err(String::from(
            "failed to get physical device surface present modes",
        ));
    }

    if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        let present_mode = vk::PresentModeKHR::MAILBOX;

        return Ok(present_mode);
    }

    if modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
        let present_mode = vk::PresentModeKHR::IMMEDIATE;

        return Ok(present_mode);
    }

    let present_mode = vk::PresentModeKHR::FIFO;

    Ok(present_mode)
}

fn check_required_device_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    required_extensions: &Vec<&std::ffi::CStr>,
) -> Result<(), String> {
    let supported_device_extensions =
        match unsafe { instance.enumerate_device_extension_properties(physical_device) } {
            Ok(props) => props,
            Err(_) => {
                return Err(String::from(
                    "failed to enumerate instance extension properies",
                ))
            }
        };

    let mut supported_device_extensions_set = std::collections::HashSet::new();
    for vk::ExtensionProperties { extension_name, .. } in &supported_device_extensions {
        supported_device_extensions_set
            .insert(unsafe { std::ffi::CStr::from_ptr(extension_name.as_ptr()) });
    }

    for extension_name in required_extensions {
        if !supported_device_extensions_set.contains(extension_name) {
            return Err(format!(
                "device extension {:?} is not supported",
                extension_name
            ));
        }
    }

    Ok(())
}

fn check_device_suitability(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    required_extensions: &Vec<&std::ffi::CStr>,
    properties: &vk::PhysicalDeviceProperties,
) -> Result<(), String> {
    // api version

    if vk::api_version_major(properties.api_version) < 1
        && vk::api_version_minor(properties.api_version) < 2
    {
        return Err(String::from(
            "the device does not support API version 1.2.0",
        ));
    }

    // features
    let features = unsafe { instance.get_physical_device_features(physical_device) };

    // TODO pass as parameter
    if features.tessellation_shader == 0 {
        return Err(String::from(
            "the device does not support tesselation shader",
        ));
    }

    if features.fill_mode_non_solid == 0 {
        return Err(String::from(
            "the device does not support fill mode non solid",
        ));
    }

    check_required_device_extensions(instance, physical_device, required_extensions)?;

    Ok(())
}

pub fn get_physical_device<'a>(
    instance: &ash::Instance,
    required_device_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<vk::PhysicalDevice, String> {
    let devices = match unsafe { instance.enumerate_physical_devices() } {
        Ok(devices) => devices,
        Err(_) => return Err(String::from("failed to enumerate physical devices")),
    };

    for &physical_device in &devices {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) };
    }

    for physical_device in devices {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        if let Err(msg) = check_device_suitability(
            instance,
            physical_device,
            required_device_extensions,
            &properties,
        ) {
            continue;
        }

        return Ok(physical_device);
    }

    Err(String::from("failed to find suitable device"))
}

pub fn create_swapchain(
    old_swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    surface_capabilities: &vk::SurfaceCapabilitiesKHR,
    surface_format: &vk::SurfaceFormatKHR,
    surface_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    swapchain_loader: &khr::Swapchain,
) -> Result<vk::SwapchainKHR, String> {
    let mut image_count = std::cmp::max(surface_capabilities.min_image_count, 3);

    if surface_capabilities.max_image_count != 0 {
        image_count = std::cmp::min(image_count, surface_capabilities.max_image_count);
    }

    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(surface_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(old_swapchain)
        .build();

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&create_info, None)
            .map_err(|_| String::from("failed to create swapchain"))?
    };

    if old_swapchain != vk::SwapchainKHR::null() {
        unsafe { swapchain_loader.destroy_swapchain(old_swapchain, None) };
    }

    Ok(swapchain)
}

pub fn create_swapchain_image_views(
    device: &ash::Device,
    swapchain_images: &Vec<vk::Image>,
    surface_format: &vk::SurfaceFormatKHR,
) -> Result<Vec<vk::ImageView>, String> {
    let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());

    for (i, &image) in swapchain_images.iter().enumerate() {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(surface_format.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();

        let view = unsafe {
            device.create_image_view(&create_info, None).map_err(|_| {
                clear_image_views(device, &swapchain_image_views);
                format!("failed to create image view {}", i)
            })?
        };

        swapchain_image_views.push(view);
    }

    Ok(swapchain_image_views)
}

fn clear_image_views(device: &ash::Device, image_views: &Vec<vk::ImageView>) {
    for &image_view in image_views {
        unsafe {
            device.destroy_image_view(image_view, None);
        };
    }
}

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, String> {
    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
        .map_err(|_| String::from("failed to create surface"))?
    };

    Ok(surface)
}

pub fn create_logical_device<'a>(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family: u32,
    device_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<ash::Device, String> {
    let queue_indices = [queue_family];

    let mut queue_priorities = Vec::new();
    for _ in &queue_indices {
        queue_priorities.push(vec![1.0f32]);
    }

    let mut queue_create_infos = Vec::with_capacity(queue_indices.len());

    for (ind, &family_index) in queue_indices.iter().enumerate() {
        let info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(family_index)
            .queue_priorities(&queue_priorities[ind]);

        queue_create_infos.push(info.build());
    }

    // TODO pass features as parameter
    let features = vk::PhysicalDeviceFeatures::builder()
        .tessellation_shader(true)
        .fill_mode_non_solid(true)
        .build();

    let device_extensions_raw = device_extensions
        .iter()
        .map(|&s| s.as_ptr())
        .collect::<Vec<*const std::os::raw::c_char>>();

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extensions_raw)
        .enabled_features(&features);

    let device = unsafe {
        instance
            .create_device(physical_device, &create_info, None)
            .map_err(|_| String::from("failed to create device"))?
    };

    return Ok(device);
}

pub fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> Result<vulkan::Allocator, String> {
    let debug_settings = gpu_allocator::AllocatorDebugSettings {
        log_memory_information: true,
        log_leaks_on_shutdown: true,
        store_stack_traces: false,
        log_allocations: true,
        log_frees: true,
        log_stack_traces: false,
    };

    let create_info = &vulkan::AllocatorCreateDesc {
        instance: instance.clone(),
        device: device.clone(),
        physical_device,
        debug_settings,
        buffer_device_address: false,
    };

    let allocator = vulkan::Allocator::new(&create_info)
        .map_err(|_| String::from("failed to create allocator"))?;

    Ok(allocator)
}

pub fn create_debug_call_back(
    debug_utils_loader: &ext::DebugUtils,
) -> Result<vk::DebugUtilsMessengerEXT, String> {
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_call_back = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .map_err(|err| format!("{}", err))?
    };

    Ok(debug_call_back)
}

pub fn create_instance<'a>(
    entry: &ash::Entry,
    instance_extensions: &Vec<&'a std::ffi::CStr>,
) -> ash::Instance {
    let extension_names_raw = instance_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();

    let app_info = vk::ApplicationInfo::builder()
        .api_version(vk::make_api_version(0, 1, 2, 0))
        .build();

    let layer_names = unsafe {
        [CStr::from_bytes_with_nul_unchecked(
            b"VK_LAYER_KHRONOS_validation\0",
        )]
    };
    let layers_names_raw: Vec<*const c_char> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        enabled_extension_count: extension_names_raw.len() as u32,
        pp_enabled_extension_names: extension_names_raw.as_ptr(),
        #[cfg(debug_assertions)]
        enabled_layer_count: layers_names_raw.len() as u32,
        #[cfg(debug_assertions)]
        pp_enabled_layer_names: layers_names_raw.as_ptr(),
        ..Default::default()
    };

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance")
    };

    instance
}

pub fn get_required_instance_extensions(
    window: &winit::window::Window,
) -> Result<Vec<&'static std::ffi::CStr>, String> {
    let mut instance_extensions =
        match ash_window::enumerate_required_extensions(window.raw_display_handle()) {
            Ok(extensions) => extensions
                .to_vec()
                .into_iter()
                .map(|name| unsafe { std::ffi::CStr::from_ptr(name) })
                .collect::<Vec<&'static std::ffi::CStr>>(),
            Err(_) => {
                return Err(String::from(
                    "failed to enumerate required instance extensions",
                ))
            }
        };

    instance_extensions.push(ash::extensions::ext::DebugUtils::name());

    Ok(instance_extensions)
}
