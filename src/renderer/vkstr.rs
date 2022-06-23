use std::ffi::CString;

lazy_static! {
    // LAYERS
    pub static ref VK_LAYER_KHRONOS_VALIDATION: CString =
        CString::new("VK_LAYER_KHRONOS_validation").unwrap();

    // EXTENSIONS
    pub static ref VK_EXT_DEBUG_UTILS: CString =
        CString::new("VK_EXT_debug_utils").unwrap();

    pub static ref VK_CREATE_DEBUG_UTILS_MESSENGER: CString =
        CString::new("vkCreateDebugUtilsMessengerEXT").unwrap();
        pub static ref VK_DESTROY_DEBUG_UTILS_MESSENGER: CString =
        CString::new("vkDestroyDebugUtilsMessengerEXT").unwrap();
}
