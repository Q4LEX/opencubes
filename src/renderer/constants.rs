use ash::extensions::ext::DebugUtils;

use crate::renderer::utils::apiversion::ApiVersion;
use std::ffi::CString;

lazy_static! {
    // INSTANCE
    pub static ref INSTANCE_APPLICATION_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref INSTANCE_APPLICATION_VERSION: ApiVersion = ApiVersion::new(0, 0, 0, 0);
    pub static ref INSTANCE_ENGINE_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref INSTANCE_ENGINE_VERSION: ApiVersion = ApiVersion::new(0, 0, 0, 0);
    pub static ref INSTANCE_API_VERSION: ApiVersion = ApiVersion::new(0, 1, 2, 0);

    pub static ref INSTANCE_DEBUG_LAYER_NAMES: Vec<CString> = vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    pub static ref INSTANCE_REQUIRED_LAYER_NAMES: Vec<CString> = vec![];
    pub static ref INSTANCE_DEBUG_EXTENSION_NAMES: Vec<CString> = vec![CString::from(DebugUtils::name())];
    pub static ref INSTANCE_REQUIRED_EXTENSION_NAMES: Vec<CString> = vec![];
    pub static ref INSTANCE_OPTIONAL_EXTENSION_NAMES: Vec<CString> = vec![];

    // PHYSICAL DEVICE
    pub static ref PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES: Vec<CString> = vec![CString::new("VK_KHR_swapchain").unwrap()];
    pub static ref PHYSICAL_DEVICE_OPTIONAL_EXTENSION_NAMES: Vec<CString> = vec![];
    pub static ref PHYSICAL_DEVICE_REQUIRED_LAYER_NAMES: Vec<CString> = vec![];
    pub static ref PHYSICAL_DEVICE_OPTIONAL_LAYER_NAMES: Vec<CString> = vec![];
}
