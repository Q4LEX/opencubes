use cstr::cstr;
use std::ffi::CStr;
// --- INSTANCE
// LAYERS
pub const VK_LAYER_KHRONOS_VALIDATION: &CStr = cstr!("VK_LAYER_KHRONOS_validation");

// EXTENSIONS
// Debug
pub const VK_EXT_DEBUG_UTILS_EXTENSION: &CStr = cstr!("VK_EXT_debug_utils");

// --- PHYSICAL DEVICE
pub const VK_KHR_SWAPCHAIN: &CStr = cstr!("VK_KHR_swapchain");

// --- FP
pub const PFN_VK_CREATE_DEBUG_UTILS_MESSENGER_EXT: &CStr = cstr!("vkCreateDebugUtilsMessengerEXT");
pub const PFN_VK_DESTROY_DEBUG_UTILS_MESSENGER_EXT: &CStr =
    cstr!("vkDestroyDebugUtilsMessengerEXT");
