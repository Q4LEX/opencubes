use std::ffi::{c_void, CStr};

use ash::{
    extensions::ext::DebugUtils,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerCreateInfoEXTBuilder, DebugUtilsMessengerEXT,
    },
    Entry, Instance,
};
use log::{error, info, trace, warn};

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

pub struct DebugMessenger {
    pub loader: DebugUtils,
    pub messenger: DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(entry: &Entry, instance: &Instance) -> Self {
        let loader = DebugUtils::new(entry, instance);
        let create_info = DebugMessenger::get_create_info();
        let messenger = unsafe {
            loader
                .create_debug_utils_messenger(&create_info, None)
                .unwrap()
        };

        Self {
            loader: DebugUtils::new(entry, instance),
            messenger,
        }
    }

    pub fn get_create_info() -> DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
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
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}
