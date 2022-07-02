use std::collections::HashSet;

use ash::vk::{
    Extent2D, PhysicalDeviceFeatures, PhysicalDeviceType, PresentModeKHR, QueueFamilyProperties,
    QueueFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
use winit::window::Window;

use super::{
    constants::{
        INSTANCE_API_VERSION, PHYSICAL_DEVICE_OPTIONAL_EXTENSION_NAMES,
        PHYSICAL_DEVICE_OPTIONAL_LAYER_NAMES, PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES,
        PHYSICAL_DEVICE_REQUIRED_LAYER_NAMES,
    },
    instance::Instance,
    surface::Surface,
    utils::{extension::Extension, layer::Layer, properties::PhysicalDeviceProperties},
};

pub struct PhysicalDevice {
    pub inner: ash::vk::PhysicalDevice,
    pub layers: Vec<Layer>,
    pub extensions: Vec<Extension>,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub queue_family_properties: Vec<QueueFamilyProperties>,
    pub queue_family_indices: QueueFamiliesIndices,
    pub swap_chain_support_details: SwapChainSupportDetails,
}

impl PhysicalDevice {
    pub fn pick(instance: &Instance, surface: &Surface) -> Self {
        let available = unsafe { instance.inner.enumerate_physical_devices().unwrap() };
        let suitable: Vec<(ash::vk::PhysicalDevice, u32)> = available
            .into_iter()
            .map(|x| (x, PhysicalDevice::rate(instance, &x, surface)))
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap()))
            .collect();

        let inner = suitable
            .into_iter()
            .max_by_key(|x| x.1)
            .expect("No suitable GPU found!")
            .0;

        let extensions = Extension::convert_vec(unsafe {
            &instance
                .inner
                .enumerate_device_extension_properties(inner)
                .unwrap()
        });
        let layers = Layer::convert_vec(unsafe {
            &instance
                .inner
                .enumerate_device_layer_properties(inner)
                .unwrap()
        });
        let properties: PhysicalDeviceProperties =
            unsafe { instance.inner.get_physical_device_properties(inner).into() };

        let features = unsafe { instance.inner.get_physical_device_features(inner) };
        let queue_family_properties = unsafe {
            instance
                .inner
                .get_physical_device_queue_family_properties(inner)
        };
        let queue_family_indices =
            QueueFamiliesIndices::extract(surface, &inner, &queue_family_properties);

        let swap_chain_support_details = SwapChainSupportDetails::extract(surface, inner);

        PhysicalDevice {
            inner,
            layers,
            extensions,
            properties,
            features,
            queue_family_properties,
            queue_family_indices,
            swap_chain_support_details,
        }
    }

    fn rate(
        instance: &Instance,
        vkphysical_device: &ash::vk::PhysicalDevice,
        surface: &Surface,
    ) -> Option<u32> {
        let mut score = 0;

        unsafe {
            let mut layers = Layer::convert_vec(
                &instance
                    .inner
                    .enumerate_device_layer_properties(*vkphysical_device)
                    .unwrap(),
            );
            layers = layers
                .into_iter()
                .filter(|l| {
                    PHYSICAL_DEVICE_OPTIONAL_LAYER_NAMES.contains(&l.name)
                        || PHYSICAL_DEVICE_REQUIRED_LAYER_NAMES.contains(&l.name)
                })
                .collect();
            for required in &*PHYSICAL_DEVICE_REQUIRED_LAYER_NAMES {
                let mut is_available = false;
                for layer in &layers {
                    if &layer.name == required {
                        is_available = true;
                        continue;
                    }
                }
                if !is_available {
                    return None;
                }
            }
            let mut extensions = Extension::convert_vec(
                &instance
                    .inner
                    .enumerate_device_extension_properties(*vkphysical_device)
                    .unwrap(),
            );
            extensions = extensions
                .into_iter()
                .filter(|l| {
                    PHYSICAL_DEVICE_OPTIONAL_EXTENSION_NAMES.contains(&l.name)
                        || PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES.contains(&l.name)
                })
                .collect();
            for required in &*PHYSICAL_DEVICE_REQUIRED_EXTENSION_NAMES {
                let mut is_available = false;
                for extension in &extensions {
                    if &extension.name == required {
                        is_available = true;
                        continue;
                    }
                }
                if !is_available {
                    return None;
                }
            }

            let features = instance
                .inner
                .get_physical_device_features(*vkphysical_device);
            if features.geometry_shader == 0 {
                return None;
            }

            let queue_family_properties = instance
                .inner
                .get_physical_device_queue_family_properties(*vkphysical_device);
            let queue_family_indices =
                QueueFamiliesIndices::extract(surface, vkphysical_device, &queue_family_properties);

            if queue_family_indices.graphics_family.is_none()
                || queue_family_indices.present_family.is_none()
            {
                return None;
            }

            if !SwapChainSupportDetails::extract(surface, *vkphysical_device).is_suitable() {
                return None;
            }

            let properties = instance
                .inner
                .get_physical_device_properties(*vkphysical_device);

            if properties.api_version < INSTANCE_API_VERSION.u32_patchless() {
                return None;
            }

            match properties.device_type {
                PhysicalDeviceType::DISCRETE_GPU => score += 1000,
                PhysicalDeviceType::INTEGRATED_GPU => score += 500,
                PhysicalDeviceType::VIRTUAL_GPU => score += 500,
                PhysicalDeviceType::OTHER => score += 300,
                _ => {}
            }
        }

        Some(score)
    }
}

#[derive(Debug)]
pub struct QueueFamiliesIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamiliesIndices {
    pub fn extract(
        surface: &Surface,
        vkphysical_device: &ash::vk::PhysicalDevice,
        properties: &Vec<QueueFamilyProperties>,
    ) -> Self {
        let mut graphics_family = None;
        let mut present_family = None;

        for (index, property) in properties.iter().enumerate() {
            if property.queue_flags.contains(QueueFlags::GRAPHICS) && graphics_family.is_none() {
                graphics_family = Some(index as u32);
            }

            if present_family.is_none()
                && unsafe {
                    surface
                        .loader
                        .get_physical_device_surface_support(
                            *vkphysical_device,
                            index as u32,
                            surface.inner,
                        )
                        .unwrap()
                }
            {
                present_family = Some(index as u32);
            }
        }

        Self {
            graphics_family,
            present_family,
        }
    }

    pub fn get_unique_indices(&self) -> Vec<u32> {
        let mut result = Vec::new();
        if self.graphics_family.is_some() {
            result.push(self.graphics_family.unwrap())
        }
        if self.present_family.is_some() {
            result.push(self.present_family.unwrap())
        }
        let mut unique = HashSet::new();
        result.retain(|i| unique.insert(*i));
        result
    }
}

pub struct SwapChainSupportDetails {
    pub surface_capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn extract(surface: &Surface, vkphysical_device: ash::vk::PhysicalDevice) -> Self {
        unsafe {
            let surface_capabilities = surface
                .loader
                .get_physical_device_surface_capabilities(vkphysical_device, surface.inner)
                .unwrap();
            let formats = surface
                .loader
                .get_physical_device_surface_formats(vkphysical_device, surface.inner)
                .unwrap();
            let present_modes = surface
                .loader
                .get_physical_device_surface_present_modes(vkphysical_device, surface.inner)
                .unwrap();

            Self {
                surface_capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn is_suitable(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn choose_format(&self) -> SurfaceFormatKHR {
        for available in &self.formats {
            if available.format == ash::vk::Format::B8G8R8A8_SRGB
                && available.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available;
            }
        }
        self.formats[0]
    }

    pub fn choose_present_mode(&self) -> ash::vk::PresentModeKHR {
        for available in &self.present_modes {
            if *available == ash::vk::PresentModeKHR::MAILBOX {
                return *available;
            }
        }
        ash::vk::PresentModeKHR::FIFO
    }

    pub fn choose_swap_extent(&self, window: &Window) -> Extent2D {
        if self.surface_capabilities.current_extent.width != u32::MAX {
            return self.surface_capabilities.current_extent;
        }

        let inner_size = window.inner_size();
        Extent2D {
            width: inner_size.width.clamp(
                self.surface_capabilities.min_image_extent.width,
                self.surface_capabilities.max_image_extent.width,
            ),
            height: inner_size.height.clamp(
                self.surface_capabilities.min_image_extent.height,
                self.surface_capabilities.max_image_extent.height,
            ),
        }
    }
}
