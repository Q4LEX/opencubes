use std::ffi::CString;

use ash::vk::LayerProperties;

use super::{apiversion::ApiVersion, cstringstuff};

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: CString,
    pub spec_version: ApiVersion,
    pub implementation_version: u32,
    pub description: CString,
}

impl Layer {
    pub fn from_properties(properties: &LayerProperties) -> Self {
        let name = cstringstuff::i8_slice_to_cstring(&properties.layer_name);
        let description = cstringstuff::i8_slice_to_cstring(&properties.description);
        let spec_version = ApiVersion::from(properties.spec_version);

        Layer {
            name,
            spec_version,
            implementation_version: properties.implementation_version,
            description,
        }
    }

    pub fn convert_vec(properties: &Vec<LayerProperties>) -> Vec<Layer> {
        let mut result = Vec::new();
        for property in properties {
            result.push(Layer::from_properties(property));
        }
        result
    }
}
