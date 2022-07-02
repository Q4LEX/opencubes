use ash::{
    vk::{
        ClearValue, CommandBuffer, CommandBufferBeginInfo, CommandBufferResetFlags, Fence,
        FenceCreateFlags, FenceCreateInfo, PipelineBindPoint, PipelineStageFlags, PresentInfoKHR,
        RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, SubmitInfo, SubpassContents,
    },
    Entry,
};
use winit::window::Window;

use self::{
    command_pool::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice,
    pipeline_graphics::GraphicsPipeline, surface::Surface, swapchain::SwapChain,
    utils::debug::DebugMessenger,
};

mod command_pool;
mod constants;
mod device;
mod instance;
mod physical_device;
mod pipeline_graphics;
mod shader_module;
mod surface;
mod swapchain;
mod utils;

pub struct Renderer {
    // SYNC
    image_available_smph: Semaphore,
    render_finished_smph: Semaphore,
    in_flight_fence: Fence,
    command_buffer: CommandBuffer,
    command_pool: CommandPool,
    graphics_pipeline: GraphicsPipeline,
    swap_chain: SwapChain,
    device: Device,
    surface: Surface,
    debug_messenger: Option<DebugMessenger>,
    instance: Instance,
    entry: Entry,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let entry = Entry::linked();
        let instance = Instance::new(&entry, window);

        let mut debug_messenger = None;
        if cfg!(debug_assertions) && instance.has_extension_debug_utils() {
            debug_messenger = Some(DebugMessenger::new(&entry, &instance.inner));
        }

        let surface = Surface::new(&entry, &instance, window);
        let physical_device = PhysicalDevice::pick(&instance, &surface);
        let device = Device::new(&instance.inner, physical_device);
        let mut swap_chain = SwapChain::new(&instance, window, &surface, &device);
        let graphics_pipeline = GraphicsPipeline::new(&device, &swap_chain);
        swap_chain.create_framebuffers(&device, &graphics_pipeline);
        let mut command_pool = CommandPool::new(&device);
        let command_buffer = command_pool.allocate();

        let smph_info = SemaphoreCreateInfo::builder();
        let fence_info = FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED);

        let (image_available_smph, render_finished_smph, in_flight_fence) = unsafe {
            (
                device.inner.create_semaphore(&smph_info, None).unwrap(),
                device.inner.create_semaphore(&smph_info, None).unwrap(),
                device.inner.create_fence(&fence_info, None).unwrap(),
            )
        };

        Renderer {
            entry,
            instance,
            debug_messenger,
            surface,
            device,
            swap_chain,
            graphics_pipeline,
            command_pool,
            command_buffer,
            image_available_smph,
            render_finished_smph,
            in_flight_fence,
        }
    }

    pub fn draw_frame(&mut self) {
        unsafe {
            self.device
                .inner
                .wait_for_fences(&[self.in_flight_fence], true, u64::MAX)
                .unwrap();
            self.device
                .inner
                .reset_fences(&[self.in_flight_fence])
                .unwrap();
            let index = self
                .swap_chain
                .loader
                .acquire_next_image(
                    self.swap_chain.inner,
                    u64::MAX,
                    self.image_available_smph,
                    Fence::null(),
                )
                .unwrap()
                .0;
            self.device
                .inner
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())
                .unwrap();
            self.record_commandbuffer(index as usize);

            let wait_semaphores = [self.image_available_smph];
            let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished_smph];
            let submit_info = SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            let submit_infos = [submit_info.build()];
            self.device
                .inner
                .queue_submit(
                    self.device.graphics_queue,
                    &submit_infos,
                    self.in_flight_fence,
                )
                .unwrap();

            let indices = [index];
            let swapchains = [self.swap_chain.inner];
            let present_info = PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);

            self.swap_chain
                .loader
                .queue_present(self.device.present_queue, &present_info)
                .unwrap();
        }
    }

    pub fn record_commandbuffer(&mut self, image_index: usize) {
        let begin_info = CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .inner
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap();
        }

        let clear_color = ClearValue::default();
        let clear_colors = [clear_color];
        let render_pass_begin_info = RenderPassBeginInfo::builder()
            .render_pass(self.graphics_pipeline.render_pass)
            .framebuffer(self.swap_chain.framebuffers[image_index])
            .render_area(ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: self.swap_chain.extent,
            })
            .clear_values(&clear_colors);

        unsafe {
            self.device.inner.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            );
            self.device.inner.cmd_bind_pipeline(
                self.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.inner,
            );
            self.device.inner.cmd_draw(self.command_buffer, 3, 1, 0, 0);
            self.device.inner.cmd_end_render_pass(self.command_buffer);
            self.device
                .inner
                .end_command_buffer(self.command_buffer)
                .unwrap();
        }
    }

    pub fn shutdown(&mut self) {
        unsafe {
            self.device.inner.device_wait_idle().unwrap();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .destroy_semaphore(self.image_available_smph, None);
            self.device
                .inner
                .destroy_semaphore(self.render_finished_smph, None);
            self.device.inner.destroy_fence(self.in_flight_fence, None);
        }
    }
}
