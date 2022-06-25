use std::mem;

use ash::{
    vk::PFN_vkCreateDebugUtilsMessengerEXT,
    vk::{Instance, PFN_vkDestroyDebugUtilsMessengerEXT},
    Entry,
};

use super::names::*;

pub unsafe fn get_create_debug_utils_messenger_fp(
    entry: &Entry,
    instance: &Instance,
) -> PFN_vkCreateDebugUtilsMessengerEXT {
    mem::transmute(
        entry
            .get_instance_proc_addr(*instance, PFN_VK_CREATE_DEBUG_UTILS_MESSENGER_EXT.as_ptr())
            .unwrap(),
    )
}

pub unsafe fn get_destroy_debug_utils_messenger_fp(
    entry: &Entry,
    instance: &Instance,
) -> PFN_vkDestroyDebugUtilsMessengerEXT {
    mem::transmute(
        entry
            .get_instance_proc_addr(*instance, PFN_VK_DESTROY_DEBUG_UTILS_MESSENGER_EXT.as_ptr())
            .unwrap(),
    )
}
