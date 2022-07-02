use std::ffi::CStr;

use ash::{
    extensions::ext::DebugUtils,
    vk::{ApplicationInfo, InstanceCreateInfo},
    Entry,
};
use winit::window::Window;

use crate::renderer::{
    constants::{
        INSTANCE_DEBUG_EXTENSION_NAMES, INSTANCE_DEBUG_LAYER_NAMES,
        INSTANCE_OPTIONAL_EXTENSION_NAMES, INSTANCE_REQUIRED_EXTENSION_NAMES,
        INSTANCE_REQUIRED_LAYER_NAMES,
    },
    utils::extension::Extension,
};

use super::{
    constants::{
        INSTANCE_API_VERSION, INSTANCE_APPLICATION_NAME, INSTANCE_APPLICATION_VERSION,
        INSTANCE_ENGINE_NAME, INSTANCE_ENGINE_VERSION,
    },
    utils::{apiversion::ApiVersion, debug::DebugMessenger, layer::Layer},
};

pub struct Instance {
    pub inner: ash::Instance,
    pub version: ApiVersion,
    pub layers: Vec<Layer>,
    pub extensions: Vec<Extension>,
}

impl Instance {
    pub fn new(entry: &Entry, window: &Window) -> Self {
        let version = match entry.try_enumerate_instance_version().unwrap() {
            Some(version) => ApiVersion::from(version),
            None => ApiVersion::new(0, 1, 0, 0),
        };

        if version < *INSTANCE_API_VERSION {
            panic!(
                "Vulkan API Version is too low! Actual: {:?}, Required: {:?}",
                version, *INSTANCE_API_VERSION
            );
        }

        let mut layers: Vec<Layer> =
            Layer::convert_vec(&entry.enumerate_instance_layer_properties().unwrap());
        layers = layers
            .into_iter()
            .filter(|l| {
                let mut is_debug = false;
                if cfg!(debug_assertions) {
                    is_debug = INSTANCE_DEBUG_LAYER_NAMES.contains(&l.name);
                }
                is_debug || INSTANCE_REQUIRED_LAYER_NAMES.contains(&l.name)
            })
            .collect();

        let mut extensions: Vec<Extension> =
            Extension::convert_vec(&entry.enumerate_instance_extension_properties(None).unwrap());
        for layer in &layers {
            extensions.extend(Extension::convert_vec(
                &entry
                    .enumerate_instance_extension_properties(Some(layer.name.as_c_str()))
                    .unwrap(),
            ));
        }

        extensions = extensions
            .into_iter()
            .filter(|e| {
                let mut is_debug = false;
                if cfg!(debug_assertions) {
                    is_debug = INSTANCE_DEBUG_EXTENSION_NAMES.contains(&e.name);
                }
                INSTANCE_REQUIRED_EXTENSION_NAMES.contains(&e.name)
                    || INSTANCE_OPTIONAL_EXTENSION_NAMES.contains(&e.name)
                    || is_debug
            })
            .collect();

        for required in &*INSTANCE_REQUIRED_LAYER_NAMES {
            let mut is_supported = false;
            let last_checked_name = required.clone();
            for layer in &layers {
                if *required == layer.name {
                    is_supported = true;
                    break;
                }
            }
            if !is_supported {
                panic!("REQUIRED LAYER NOT SUPPORTED: {:?}", last_checked_name);
            }
        }

        for required in &*INSTANCE_REQUIRED_EXTENSION_NAMES {
            let mut is_supported = false;
            let last_checked_name = required.clone();
            for extension in &extensions {
                if *required == extension.name {
                    is_supported = true;
                    break;
                }
            }
            if !is_supported {
                panic!("REQUIRED EXTENSION NOT SUPPORTED: {:?}", last_checked_name);
            }
        }

        let application_info = ApplicationInfo::builder()
            .application_name(&INSTANCE_APPLICATION_NAME)
            .application_version(INSTANCE_APPLICATION_VERSION.u32())
            .engine_name(&INSTANCE_ENGINE_NAME)
            .engine_version(INSTANCE_ENGINE_VERSION.u32())
            .api_version(INSTANCE_API_VERSION.u32_patchless());

        let layer_names_raw: Vec<*const i8> =
            layers.iter().map(|l| l.name.as_c_str().as_ptr()).collect();

        let mut extension_names_raw: Vec<*const i8> = extensions
            .iter()
            .map(|l| l.name.as_c_str().as_ptr())
            .collect();
        extension_names_raw.extend(ash_window::enumerate_required_extensions(window).unwrap());

        let mut create_info = InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw);

        let mut instance_debug_create_info;
        if cfg!(debug_assertions)
            && extensions
                .iter()
                .any(|x| (x.name).as_c_str() == DebugUtils::name())
        {
            instance_debug_create_info = DebugMessenger::get_create_info();
            create_info = create_info.push_next(&mut instance_debug_create_info);
        }

        let inner = unsafe { entry.create_instance(&create_info, None).unwrap() };

        Instance {
            inner,
            version,
            layers,
            extensions,
        }
    }

    pub fn has_extension_debug_utils(&self) -> bool {
        self.has_extension(DebugUtils::name())
    }

    pub fn has_extension(&self, name: &CStr) -> bool {
        self.extensions.iter().any(|x| (x.name).as_c_str() == name)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy_instance(None);
        }
    }
}
