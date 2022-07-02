use ash::{
    vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDeviceFeatures, Queue},
    Instance,
};

use super::{
    constants::{
        PHYSICAL_DEVICE_OPTIONAL_EXTENSION_NAMES, PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES,
    },
    physical_device::PhysicalDevice,
    utils::extension::Extension,
};

pub struct Device {
    pub inner: ash::Device,
    pub physical_device: PhysicalDevice,
    pub enabled_extensions: Vec<Extension>,
    pub enabled_features: PhysicalDeviceFeatures,
    pub graphics_queue: Queue,
    pub present_queue: Queue,
}

impl Device {
    pub fn new(instance: &Instance, physical_device: PhysicalDevice) -> Self {
        let mut queue_create_infos: Vec<DeviceQueueCreateInfo> = Vec::new();
        let unique_queue_families = physical_device.queue_family_indices.get_unique_indices();
        let queue_priorities = [1.0];
        for unique in unique_queue_families {
            let queue_create_info = DeviceQueueCreateInfo::builder()
                .queue_family_index(unique)
                .queue_priorities(&queue_priorities);
            queue_create_infos.push(queue_create_info.build());
        }

        let enabled_extensions: Vec<Extension> = physical_device
            .extensions
            .iter()
            .filter(|x| {
                PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES.contains(&x.name)
                    || PHYSICAL_DEVICE_OPTIONAL_EXTENSION_NAMES.contains(&x.name)
            })
            .cloned()
            .collect();

        let enabled_extensions_names_raw: Vec<*const i8> =
            enabled_extensions.iter().map(|x| x.name.as_ptr()).collect();

        let enabled_features = PhysicalDeviceFeatures::builder().build();

        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&enabled_extensions_names_raw)
            .enabled_features(&enabled_features);

        let inner = unsafe {
            instance
                .create_device(physical_device.inner, &device_create_info, None)
                .unwrap()
        };

        let graphics_queue = unsafe {
            inner.get_device_queue(
                physical_device
                    .queue_family_indices
                    .graphics_family
                    .unwrap(),
                0,
            )
        };

        let present_queue = unsafe {
            inner.get_device_queue(
                physical_device.queue_family_indices.present_family.unwrap(),
                0,
            )
        };

        Self {
            inner,
            physical_device,
            enabled_features,
            enabled_extensions,
            graphics_queue,
            present_queue,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.inner.destroy_device(None) };
    }
}
