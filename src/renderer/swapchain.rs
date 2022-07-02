use ash::vk::{
    ComponentMapping, ComponentSwizzle, Extent2D, Framebuffer, FramebufferCreateInfo,
    ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType,
    PresentModeKHR, SurfaceFormatKHR, SwapchainCreateInfoKHR,
};
use winit::window::Window;

use super::{
    device::Device, instance::Instance, pipeline_graphics::GraphicsPipeline, surface::Surface,
};

pub struct SwapChain {
    pub inner: ash::vk::SwapchainKHR,
    pub loader: ash::extensions::khr::Swapchain,
    pub images: Vec<ash::vk::Image>,
    pub image_views: Vec<ImageView>,
    pub surface_format: SurfaceFormatKHR,
    pub extent: Extent2D,
    pub present_mode: PresentModeKHR,
    pub framebuffers: Vec<Framebuffer>,
    device: ash::Device,
}

impl SwapChain {
    pub fn new(instance: &Instance, window: &Window, surface: &Surface, device: &Device) -> Self {
        let physical_device = &device.physical_device;
        let surface_format = physical_device.swap_chain_support_details.choose_format();
        let present_mode = physical_device
            .swap_chain_support_details
            .choose_present_mode();
        let extent = physical_device
            .swap_chain_support_details
            .choose_swap_extent(window);

        let mut image_count = physical_device
            .swap_chain_support_details
            .surface_capabilities
            .min_image_count
            + 1;
        if physical_device
            .swap_chain_support_details
            .surface_capabilities
            .max_image_count
            > 0
            && image_count
                > physical_device
                    .swap_chain_support_details
                    .surface_capabilities
                    .max_image_count
        {
            image_count = physical_device
                .swap_chain_support_details
                .surface_capabilities
                .max_image_count;
        }

        let mut create_info = SwapchainCreateInfoKHR::builder()
            .surface(surface.inner)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(
                physical_device
                    .swap_chain_support_details
                    .surface_capabilities
                    .current_transform,
            )
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_indices;
        if physical_device
            .queue_family_indices
            .graphics_family
            .unwrap()
            != physical_device.queue_family_indices.present_family.unwrap()
        {
            queue_family_indices = [
                physical_device
                    .queue_family_indices
                    .graphics_family
                    .unwrap(),
                physical_device.queue_family_indices.present_family.unwrap(),
            ];
            create_info = create_info
                .image_sharing_mode(ash::vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE);
        }

        let loader = ash::extensions::khr::Swapchain::new(&instance.inner, &device.inner);
        let inner = unsafe { loader.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { loader.get_swapchain_images(inner).unwrap() };
        let mut image_views = Vec::new();

        for image in &images {
            let components = ComponentMapping::builder()
                .a(ComponentSwizzle::IDENTITY)
                .r(ComponentSwizzle::IDENTITY)
                .g(ComponentSwizzle::IDENTITY)
                .b(ComponentSwizzle::IDENTITY);

            let subresource_range = ImageSubresourceRange::builder()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let image_view_create_info = ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(*components)
                .subresource_range(*subresource_range);

            let image_view = unsafe {
                device
                    .inner
                    .create_image_view(&image_view_create_info, None)
                    .unwrap()
            };

            image_views.push(image_view);
        }

        Self {
            inner,
            loader,
            images,
            image_views,
            surface_format,
            present_mode,
            framebuffers: Vec::new(),
            extent,
            device: device.inner.clone(),
        }
    }

    pub fn create_framebuffers(&mut self, device: &Device, graphics_pipeline: &GraphicsPipeline) {
        self.framebuffers.clear();
        for i in 0..self.image_views.len() {
            let attachments = [self.image_views[i]];
            let create_info = FramebufferCreateInfo::builder()
                .render_pass(graphics_pipeline.render_pass)
                .attachments(&attachments)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);

            let framebuffer =
                unsafe { device.inner.create_framebuffer(&create_info, None).unwrap() };
            self.framebuffers.push(framebuffer);
        }
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.loader.destroy_swapchain(self.inner, None);
        }
    }
}
