use std::ffi::CString;

use ash::vk::{PhysicalDeviceLimits, PhysicalDeviceType};

use super::apiversion::ApiVersion;

pub struct PhysicalDeviceProperties {
    pub api_version: ApiVersion,
    pub device_type: PhysicalDeviceType,
    pub name: CString,
    pub limits: PhysicalDeviceLimits,
}

impl PhysicalDeviceProperties {}

impl Into<PhysicalDeviceProperties> for ash::vk::PhysicalDeviceProperties {
    fn into(self) -> PhysicalDeviceProperties {
        let name = unsafe {
            CString::from_vec_unchecked(
                self.device_name
                    .to_vec()
                    .iter()
                    .filter_map(|x| {
                        let x = *x as u8;
                        if x == 0 {
                            None
                        } else {
                            Some(x)
                        }
                    })
                    .collect(),
            )
        };

        PhysicalDeviceProperties {
            api_version: ApiVersion::from(self.api_version),
            device_type: self.device_type,
            name,
            limits: self.limits,
        }
    }
}
