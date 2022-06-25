use std::ffi::{c_void, CStr};

use ash::{
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerCreateInfoEXTBuilder, DebugUtilsMessengerEXT,
    },
    Entry,
};
use log::{error, info, trace, warn};

use super::fp;

unsafe extern "system" fn debug_callback(
    severity: DebugUtilsMessageSeverityFlagsEXT,
    msg_type: DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let type_prefix = match msg_type {
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        _ => "UNKNOWN",
    };

    let message = CStr::from_ptr((*callback_data).p_message);

    match severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            trace!("[{}] {:?}", type_prefix, message);
        }
        DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!("[{}] {:?}", type_prefix, message);
        }
        DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!("[{}] {:?}", type_prefix, message);
        }
        DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!("[{}] {:?}", type_prefix, message);
        }
        _ => {
            error!("[UNKNOWN SEVERITY] [{}] {:?}", type_prefix, message);
        }
    }
    vk::FALSE
}

pub fn get_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(debug_callback))
}

pub fn setup_debug_messenger(
    entry: &Entry,
    instance: &ash::vk::Instance,
) -> DebugUtilsMessengerEXT {
    unsafe {
        let create_info = get_debug_messenger_create_info().build();
        let debug_create_fp = fp::get_create_debug_utils_messenger_fp(entry, instance);
        let mut debug_utils_messenger: DebugUtilsMessengerEXT = std::mem::zeroed();
        if debug_create_fp(
            *instance,
            &create_info,
            std::ptr::null(),
            &mut debug_utils_messenger,
        )
        .result()
        .is_err()
        {
            panic!("Failed to set up debug messenger!");
        };
        debug_utils_messenger
    }
}
