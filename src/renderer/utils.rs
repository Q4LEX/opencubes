use ash::{
    extensions::khr::Surface,
    vk::{ExtensionProperties, PhysicalDevice, PhysicalDeviceType, Queue, QueueFlags, SurfaceKHR},
    Entry, Instance,
};
use log::warn;
use std::{
    collections::HashSet,
    ffi::{CStr, CString},
};

pub fn api_version_tuple(x: u32) -> (u32, u32, u32, u32) {
    (
        ash::vk::api_version_variant(x),
        ash::vk::api_version_major(x),
        ash::vk::api_version_minor(x),
        ash::vk::api_version_patch(x),
    )
}

pub unsafe fn cstr_ref_slice_to_raw(slice: &[&CStr]) -> Vec<*const i8> {
    slice.iter().map(|x| x.as_ptr()).collect()
}

pub unsafe fn cstr_ref_to_cstring(slice: &[&CStr]) -> Vec<CString> {
    slice.iter().map(|x| (*x).to_owned()).collect()
}

pub unsafe fn raw_to_cstr_ref(slice: &[*const i8]) -> Vec<&CStr> {
    slice.iter().map(|x| CStr::from_ptr(*x)).collect()
}
pub unsafe fn raw_to_cstring(slice: &[*const i8]) -> Vec<CString> {
    slice
        .iter()
        .map(|x| {
            let cstr = CStr::from_ptr(*x);
            cstr.to_owned()
        })
        .collect()
}

pub unsafe fn cstring_slice_to_cstr(slice: &[CString]) -> Vec<&CStr> {
    slice.iter().map(|x| x.as_c_str()).collect()
}

pub unsafe fn cstring_slice_to_raw(slice: &[CString]) -> Vec<*const i8> {
    slice.iter().map(|x| x.as_ptr()).collect()
}

// INSTANCE STUFF
pub unsafe fn filter_supported_layers(entry: &Entry, layers: &[&CStr]) -> Vec<CString> {
    let mut available_layers = Vec::new();
    let properties = entry.enumerate_instance_layer_properties().unwrap();
    for layer in layers {
        let mut is_available = false;
        for property in &properties {
            let name = CStr::from_ptr(property.layer_name.as_ptr());
            if name == *layer {
                available_layers.push(CString::new((*layer).clone().to_bytes()).unwrap());
                is_available = true;
                break;
            }
        }
        if !is_available {
            warn!("Requested layer: {:?} not available.", layer);
        }
    }
    available_layers
}

pub unsafe fn filter_supported_instance_extensions(
    entry: &Entry,
    supported_layers: &[&CStr],
    requested_extensions: &[&CStr],
) -> Vec<CString> {
    let mut available_extensions = Vec::new();
    let extensions = get_supported_instance_extensions(entry, supported_layers);
    for requested_extension in requested_extensions {
        for extension in &extensions {
            if CStr::from_ptr(extension.extension_name.as_ptr()) == *requested_extension {
                available_extensions
                    .push(CString::new(requested_extension.clone().to_bytes()).unwrap());
                break;
            }
        }
    }
    available_extensions
}

unsafe fn get_supported_instance_extensions(
    entry: &Entry,
    supported_layers: &[&CStr],
) -> Vec<ExtensionProperties> {
    let mut result = entry.enumerate_instance_extension_properties(None).unwrap();
    for layer in supported_layers {
        result.extend_from_slice(
            &entry
                .enumerate_instance_extension_properties(Some(layer))
                .unwrap(),
        );
    }
    result
}

// PHYSICAL DEVICE STUFF
pub unsafe fn pick_most_suitable_physical_device(
    instance: &Instance,
    surface_loader: &Surface,
    surface: &SurfaceKHR,
    required_extensions: &[&CStr],
    optional_extensions: &[&CStr],
    ratings: &[u32],
) -> PhysicalDevice {
    if ratings.len() != optional_extensions.len() {
        panic!("Length of optional device extensions and ratings is off!");
    }

    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("No Vulkan GPU available!");
    let mut best: (usize, u32) = (0, 0);
    'outer: for (p_index, physical_device) in physical_devices.iter().enumerate() {
        let mut score: u32 = 0;
        let available_extensions = instance
            .enumerate_device_extension_properties(*physical_device)
            .unwrap();

        // 1. MAKE SURE DEVICE IS SUITABLE
        if !find_queue_families(instance, physical_device, surface_loader, surface).has_required() {
            continue 'outer;
        }
        for extension in required_extensions {
            let mut is_available = false;
            for available_extension in &available_extensions {
                let available_name = CStr::from_ptr(available_extension.extension_name.as_ptr());
                if *extension == available_name {
                    is_available = true;
                    break;
                }
            }
            if !is_available {
                continue 'outer;
            }
        }

        // 2. RATE DEVICE BY EXTENSIONS
        for (index, extension) in optional_extensions.iter().enumerate() {
            let mut is_available = false;
            for available_extension in &available_extensions {
                let available_name = CStr::from_ptr(available_extension.extension_name.as_ptr());
                if *extension == available_name {
                    is_available = true;
                    break;
                }
            }
            if is_available {
                score += ratings[index];
            }
        }

        // 3. RATE DEVICE BY ETC
        let device_properties = instance.get_physical_device_properties(*physical_device);
        if device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        if score > best.1 {
            best = (p_index, score);
        }
    }

    if best.1 == 0 {
        panic!("No Suitable Physical Device found");
    }

    physical_devices[best.0]
}

pub unsafe fn filter_supported_physical_device_extensions(
    instance: &Instance,
    physical_device: &PhysicalDevice,
    extensions: &[&CStr],
) -> Vec<CString> {
    let mut result = Vec::new();
    let available_extension_properties = instance
        .enumerate_device_extension_properties(*physical_device)
        .unwrap();
    for extension in extensions {
        for available_extension in &available_extension_properties {
            if *extension == CStr::from_ptr(available_extension.extension_name.as_ptr()) {
                result.push(CString::new(extension.clone().to_bytes()).unwrap());
            }
        }
    }
    result
}

pub struct Queues {
    pub graphics_queue: Queue,
    pub present_queue: Queue,
}

#[derive(Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn has_required(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }

    pub fn get_unique_indices(&self) -> HashSet<u32> {
        let mut result = HashSet::new();
        if self.graphics_family.is_some() {
            result.insert(self.graphics_family.unwrap());
        }
        if self.present_family.is_some() {
            result.insert(self.present_family.unwrap());
        }

        result
    }
}

pub fn find_queue_families(
    instance: &Instance,
    device: &PhysicalDevice,
    surface_loader: &Surface,
    surface: &SurfaceKHR,
) -> QueueFamilyIndices {
    let mut graphics_family = None;
    let mut present_family = None;
    unsafe {
        let properties = instance.get_physical_device_queue_family_properties(*device);
        for (index, property) in properties.iter().enumerate() {
            if property.queue_flags.contains(QueueFlags::GRAPHICS) {
                if graphics_family.is_none() {
                    graphics_family = Some(index as u32);
                }
            }
            if present_family.is_none() {
                if surface_loader
                    .get_physical_device_surface_support(*device, index as u32, *surface)
                    .unwrap()
                {
                    present_family = Some(index as u32);
                }
            }
            // TODO CHECK FOR PRESENT
        }
    }

    QueueFamilyIndices {
        graphics_family,
        present_family,
    }
}
