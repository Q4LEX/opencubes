mod utils;
mod vkstr;

use std::{
    ffi::{c_void, CStr, CString},
    ptr,
};

use ash::{
    extensions::khr::Surface,
    vk::{
        self, ApplicationInfo, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerCreateInfoEXTBuilder, DebugUtilsMessengerEXT,
        PFN_vkCreateDebugUtilsMessengerEXT, SurfaceKHR,
    },
    Entry, Instance,
};
use log::{error, info, trace, warn};
use winit::window::Window;

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

pub struct Renderer {
    entry: Entry,
    instance: Instance,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    surface: SurfaceKHR,
    surface_fn: Surface,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        info!("Started creating renderer.");

        let entry = Entry::linked();

        // ------------------------------------------------------------------------------------
        // ------------------------------- INSTANCE CREATION ----------------------------------
        // ------------------------------------------------------------------------------------
        let app_name = CString::new("OpenCubes").unwrap();
        let app_version = vk::make_api_version(0, 0, 0, 0);
        let engine_name = CString::new("OpenCubes").unwrap();
        let engine_version = vk::make_api_version(0, 0, 0, 0);
        let api_version = vk::make_api_version(0, 1, 3, 0);
        let app_info = ApplicationInfo::builder()
            .api_version(api_version)
            .application_name(&app_name)
            .application_version(app_version)
            .engine_name(&engine_name)
            .engine_version(engine_version);

        // VALIDATION LAYERS
        let mut required_layers = Vec::new();
        if cfg!(debug_assertions) {
            required_layers.push(vkstr::VK_LAYER_KHRONOS_VALIDATION.as_ptr());
        }

        if !required_layers.is_empty() {
            info!("Validation Layers: Requested");
            let layer_properties = entry.enumerate_instance_layer_properties().unwrap();
            for required_layer in &required_layers {
                let mut is_available = false;
                let required_layer = unsafe { CStr::from_ptr(*required_layer) };
                for layer in &layer_properties {
                    let layer_name = unsafe { utils::vk_to_cstr(&layer.layer_name) };
                    if layer_name == required_layer {
                        is_available = true;
                    }
                }
                if !is_available {
                    warn!("Validation Layers: Unavailable");
                    required_layers.clear();
                    break;
                }
                info!("Validation Layers: Available");
            }
        } else {
            info!("Validation Layers: Not Requested");
        }

        // EXTENSIONS
        info!("Getting needed extensions");
        let mut required_extensions = Vec::new();
        required_extensions.extend(ash_window::enumerate_required_extensions(window).unwrap());
        info!("Requested required surface extensions");
        if cfg!(debug_assertions) {
            info!("Requested Extension: Debug Utils");
            required_extensions.push(vkstr::VK_EXT_DEBUG_UTILS.as_ptr());
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&required_layers)
            .enabled_extension_names(&required_extensions);

        let new_info;
        let mut _dbg_create_info = Renderer::create_debug_messenger_create_info();
        if cfg!(debug_assertions) {
            new_info = create_info.push_next(&mut _dbg_create_info);
            info!("Added DebugUtilsMessengerCreateInfoEXT to InstanceCreateInfo");
        } else {
            new_info = create_info;
        }

        let instance = unsafe { entry.create_instance(&new_info, None).unwrap() };
        info!("Instance created.");

        let mut debug_utils_messenger = None;
        if cfg!(debug_assertions) {
            debug_utils_messenger =
                Some(Renderer::setup_debug_messenger(&entry, instance.handle()));
        }

        let surface =
            unsafe { ash_window::create_surface(&entry, &instance, &window, None).unwrap() };
        let surface_fn = ash::extensions::khr::Surface::new(&entry, &instance);
        info!("Aquired surface");

        Self {
            entry,
            instance,
            debug_utils_messenger,
            surface,
            surface_fn,
        }
    }

    fn setup_debug_messenger(entry: &Entry, instance: vk::Instance) -> DebugUtilsMessengerEXT {
        info!("Setting up debug messenger");
        unsafe {
            let create_info = Renderer::create_debug_messenger_create_info();
            let create_debug_utils_messenger_ext =
                utils::get_instance_proc_addr::<PFN_vkCreateDebugUtilsMessengerEXT>(
                    entry,
                    instance,
                    &vkstr::VK_CREATE_DEBUG_UTILS_MESSENGER,
                );
            info!("Aquired debug utils messenger ext function pointer");
            if let Some(create_debug_utils_messenger_fn) = create_debug_utils_messenger_ext {
                let mut debug_utils_messenger = std::mem::zeroed::<vk::DebugUtilsMessengerEXT>();
                if create_debug_utils_messenger_fn(
                    instance,
                    &*create_info,
                    ptr::null(),
                    &mut debug_utils_messenger as *mut _,
                )
                .result()
                .is_err()
                {
                    panic!("Unable to create debug utils messenger");
                }
                info!("Debug Utils Messenger set up");
                debug_utils_messenger
            } else {
                panic!("Unable to get create_debug_utils_messenger_ext function");
            }
        }
    }

    fn create_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
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

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if cfg!(debug_assertions) {
                info!("Destroying debug utils messenger");
                let destroy_debug_utils_messenger =
                    utils::get_instance_proc_addr::<vk::PFN_vkDestroyDebugUtilsMessengerEXT>(
                        &self.entry,
                        self.instance.handle(),
                        &vkstr::VK_DESTROY_DEBUG_UTILS_MESSENGER,
                    );
                if let Some(destroy_debug_utils_messenger) = destroy_debug_utils_messenger {
                    destroy_debug_utils_messenger(
                        self.instance.handle(),
                        self.debug_utils_messenger.unwrap(),
                        ptr::null(),
                    );
                }
            }
            info!("Destroying surface");
            self.surface_fn.destroy_surface(self.surface, None);
            info!("Destroying instance");
            self.instance.destroy_instance(None);
        }
    }
}
