use std::borrow::Cow;
use std::ffi::CStr;

use ash::vk::{self, DebugUtilsMessageSeverityFlagsEXT};

pub const MAX_FRAME_DRAWS: usize = 2;

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let message = format!(
        "{{\n\ttype: {:?}\n\tid name: {}\n\tid number: {}\n\tmessage: {}\n}}",
        message_type, message_id_name, message_id_number, message,
    );

    match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("{}", message),
        DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("{}", message),
        DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{}", message),
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::debug!("{}", message),
        _ => (),
    }

    vk::FALSE
}

#[macro_export]
macro_rules! msg {
    ($type:tt, $msg:expr) => {{
        log::$type!(
            "{{\n\tmessage: {:?}\n\tlocation: {}:{}\n}}",
            $msg,
            file!(),
            line!()
        );
    }};
}

#[macro_export]
macro_rules! create_shader {
    ($path:expr, $device:expr) => {{
        unsafe {
            use ash::{util::read_spv, vk};
            use std::io::Cursor;

            let mut spv_file = Cursor::new(&include_bytes!($path)[..]);

            let code = read_spv(&mut spv_file).expect("Failed to read shader spv file");
            let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);
            let shader_module = $device
                .create_shader_module(&shader_info, None)
                .expect("shader module error");

            shader_module
        }
    }};
}
