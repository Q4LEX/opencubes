use std::ffi::CString;

use ash::vk::ExtensionProperties;

use crate::renderer::utils::cstringstuff;

#[derive(Clone, Debug)]
pub struct Extension {
    pub name: CString,
    pub spec_version: Option<u32>,
}

impl Extension {
    pub fn from_properties(properties: &ExtensionProperties) -> Self {
        let name = cstringstuff::i8_slice_to_cstring(&properties.extension_name);

        Extension {
            name,
            spec_version: Some(properties.spec_version),
        }
    }

    pub fn convert_vec(properties: &Vec<ExtensionProperties>) -> Vec<Extension> {
        let mut result = Vec::new();
        for property in properties {
            result.push(Extension::from_properties(property));
        }
        result
    }
}
