use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPoolCreateFlags,
    CommandPoolCreateInfo,
};

use super::device::Device;

pub struct CommandPool {
    pub inner: ash::vk::CommandPool,
    device: ash::Device,
}

impl CommandPool {
    pub fn new(device: &Device) -> Self {
        let create_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(
                device
                    .physical_device
                    .queue_family_indices
                    .graphics_family
                    .unwrap(),
            );

        let inner = unsafe {
            device
                .inner
                .create_command_pool(&create_info, None)
                .unwrap()
        };

        Self {
            inner,
            device: device.inner.clone(),
        }
    }

    pub fn allocate(&mut self) -> CommandBuffer {
        let alloc_info = CommandBufferAllocateInfo::builder()
            .command_pool(self.inner)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap()[0] }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.inner, None);
        }
    }
}
