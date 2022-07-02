use ash::vk::ShaderModuleCreateInfo;

use super::device::Device;

pub struct ShaderModule {
    pub inner: ash::vk::ShaderModule,
    device: ash::Device,
}

impl ShaderModule {
    pub fn new(device: &Device, code: &[u8]) -> Self {
        let mut create_info = ShaderModuleCreateInfo::builder();
        create_info.p_code = code.as_ptr() as *const u32;
        create_info.code_size = code.len();

        let inner = unsafe {
            device
                .inner
                .create_shader_module(&create_info, None)
                .unwrap()
        };

        ShaderModule {
            inner,
            device: device.inner.clone(),
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.inner, None);
        }
    }
}
