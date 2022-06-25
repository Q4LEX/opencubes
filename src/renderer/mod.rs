use std::ffi::{CStr, CString};

use ash::{
    extensions::khr::Surface,
    vk::{
        make_api_version, ApplicationInfo, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerEXT, DeviceCreateInfo, DeviceQueueCreateInfo, InstanceCreateInfo,
        PhysicalDevice, SurfaceKHR,
    },
    Device, Entry, Instance,
};
use log::info;
use winit::window::Window;

use crate::renderer::utils::api_version_tuple;

use self::{
    names::*,
    utils::{cstring_slice_to_raw, QueueFamilyIndices, Queues},
};

mod debug;
mod fp;
mod names;
mod utils;

// CONSTANTS
#[rustfmt::skip]
lazy_static! {
    pub static ref APPLICATION_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref APPLICATION_VERSION: u32 = make_api_version(0, 0, 0, 0);
    pub static ref ENGINE_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref ENGINE_VERSION: u32 = make_api_version(0, 0, 0, 0);

    pub static ref API_VERSION_VARIANT: u32 = 0;
    pub static ref API_VERSION_MAJOR: u32 = 1;
    pub static ref API_VERSION_MINOR: u32 = 2;
    pub static ref API_VERSION_PATCH: u32 = 218;

    pub static ref API_VERSION: u32 = make_api_version(
        *API_VERSION_VARIANT,
        *API_VERSION_MAJOR,
        *API_VERSION_MINOR,
        *API_VERSION_PATCH
    );
    pub static ref API_VERSION_PATCHLESS: u32 = make_api_version(
        *API_VERSION_VARIANT,
        *API_VERSION_MAJOR,
        *API_VERSION_MINOR,
        0
    );
}

// INSTANCE LAYERS/EXTENSIONS
// DEBUG (OPTIONAL)
const VALIDATION_LAYERS: &[&CStr; 1] = &[VK_LAYER_KHRONOS_VALIDATION];
const DEBUG_EXTENSIONS: &[&CStr; 1] = &[VK_EXT_DEBUG_UTILS_EXTENSION];

// PHYSICAL DEVICE EXTENSIONS
// REQUIRED
const REQUIRED_DEVICE_EXTENSIONS: &[&CStr; 0] = &[];
// OPTIONAL
const OPTIONAL_DEVICE_EXTENSIONS: &[&CStr; 0] = &[];
const OPTIONAL_DEVICE_EXTENSIONS_RATING: &[u32; 0] = &[];

pub struct Renderer {
    entry: Entry,
    instance: Instance,

    instance_layers: Vec<CString>,
    instance_extensions: Vec<CString>,

    surface: SurfaceKHR,
    surface_loader: Surface,

    physical_device: PhysicalDevice,
    device: Device,
    device_extensions: Vec<CString>,

    queue_family_indices: QueueFamilyIndices,
    queues: Queues,

    debug_messenger: Option<DebugUtilsMessengerEXT>,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let entry = Entry::linked();
        let (instance, instance_layers, instance_extensions) =
            Renderer::create_instance(&entry, window);

        let mut debug_messenger = None;
        if cfg!(debug_assertions)
            && instance_extensions
                .contains(&CString::new(VK_EXT_DEBUG_UTILS_EXTENSION.to_bytes().clone()).unwrap())
        {
            debug_messenger = Some(debug::setup_debug_messenger(&entry, &instance.handle()));
        }

        let surface: SurfaceKHR =
            unsafe { ash_window::create_surface(&entry, &instance, window, None).unwrap() };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        let physical_device = unsafe {
            utils::pick_most_suitable_physical_device(
                &instance,
                &surface_loader,
                &surface,
                REQUIRED_DEVICE_EXTENSIONS,
                OPTIONAL_DEVICE_EXTENSIONS,
                OPTIONAL_DEVICE_EXTENSIONS_RATING,
            )
        };

        let mut device_extensions = Vec::new();
        unsafe {
            device_extensions.extend(utils::filter_supported_physical_device_extensions(
                &instance,
                &physical_device,
                OPTIONAL_DEVICE_EXTENSIONS,
            ));
            device_extensions.extend(utils::filter_supported_physical_device_extensions(
                &instance,
                &physical_device,
                REQUIRED_DEVICE_EXTENSIONS,
            ));
        }

        info!("Device Extensions used: {:?}", device_extensions);
        let queue_family_indices =
            utils::find_queue_families(&instance, &physical_device, &surface_loader, &surface);

        let device = unsafe {
            Renderer::create_logical_device(
                &instance,
                &surface_loader,
                &surface,
                &physical_device,
                &device_extensions,
            )
        };

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(queue_family_indices.present_family.unwrap(), 0) };

        let queues = Queues { graphics_queue, present_queue };

        Renderer {
            entry,
            instance,
            instance_layers,
            instance_extensions,
            surface,
            surface_loader,
            physical_device,
            device,
            device_extensions,
            queue_family_indices,
            queues,
            debug_messenger,
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> (Instance, Vec<CString>, Vec<CString>) {
        let version = match entry.try_enumerate_instance_version().unwrap() {
            Some(version) => version,
            None => make_api_version(0, 1, 0, 0),
        };

        if version < *API_VERSION_PATCHLESS {
            panic!(
                "Vulkan Instance version too low: {:?} vs {:?}",
                api_version_tuple(version),
                api_version_tuple(*API_VERSION_PATCHLESS)
            );
        }

        let app_info = ApplicationInfo::builder()
            .application_name(&APPLICATION_NAME)
            .application_version(*APPLICATION_VERSION)
            .engine_name(&ENGINE_NAME)
            .engine_version(*ENGINE_VERSION)
            .api_version(*API_VERSION);

        let mut create_info = InstanceCreateInfo::builder().application_info(&app_info);

        // Add Surface Extensions
        let _extensions_required_for_surface =
            ash_window::enumerate_required_extensions(window).unwrap();
        let mut extensions: Vec<CString> =
            unsafe { utils::raw_to_cstring(_extensions_required_for_surface) };

        // Add Validation Layers and Debug Extensions
        let mut layers: Vec<CString> = Vec::new();
        if cfg!(debug_assertions) {
            unsafe {
                layers = utils::filter_supported_layers(entry, VALIDATION_LAYERS);
                extensions.extend(utils::filter_supported_instance_extensions(
                    entry,
                    &utils::cstring_slice_to_cstr(&layers),
                    DEBUG_EXTENSIONS,
                ));
            }
        }

        // Add Instance Creation/Destruction Debugging
        let mut dbg_msg_create_info: DebugUtilsMessengerCreateInfoEXT;
        if cfg!(debug_assertions) {
            dbg_msg_create_info = *debug::get_debug_messenger_create_info();
            create_info = create_info.push_next(&mut dbg_msg_create_info);
        }

        let layers_raw = unsafe { utils::cstring_slice_to_raw(&layers) };
        create_info = create_info.enabled_layer_names(&layers_raw);

        let extensions_raw = unsafe { utils::cstring_slice_to_raw(&extensions) };
        create_info = create_info.enabled_extension_names(&extensions_raw);

        info!("INSTANCE LAYERS USED: {:?}", layers);
        info!("INSTANCE EXTENSIONS USED: {:?}", extensions);

        (
            unsafe { entry.create_instance(&create_info, None).unwrap() },
            layers,
            extensions,
        )
    }

    unsafe fn create_logical_device(
        instance: &Instance,
        surface_loader: &Surface,
        surface: &SurfaceKHR,
        physical_device: &PhysicalDevice,
        extensions: &[CString],
    ) -> Device {
        let queue_family_indices =
            utils::find_queue_families(instance, physical_device, &surface_loader, &surface);

        let mut family_queue_create_info = DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_family.unwrap())
            .queue_priorities(&[1.0]);
        family_queue_create_info.queue_count = 1;

        let slice = cstring_slice_to_raw(extensions);
        let infos = [*family_queue_create_info];
        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&infos)
            .enabled_extension_names(&slice);

        instance
            .create_device(*physical_device, &device_create_info, None)
            .unwrap()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(dbg_msg) = self.debug_messenger {
                let destroy_fn =
                    fp::get_destroy_debug_utils_messenger_fp(&self.entry, &self.instance.handle());
                destroy_fn(self.instance.handle(), dbg_msg, std::ptr::null());
            }
            self.surface_loader.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
