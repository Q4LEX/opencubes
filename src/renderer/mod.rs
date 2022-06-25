use std::ffi::{CStr, CString};

use ash::{
    extensions::khr::{Surface, Swapchain},
    vk::{
        make_api_version, ApplicationInfo, AttachmentDescription, AttachmentLoadOp,
        AttachmentReference, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags,
        CompositeAlphaFlagsKHR, CullModeFlags, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerEXT, DeviceCreateInfo, DeviceQueueCreateInfo, DynamicState, Extent2D,
        Format, FrontFace, GraphicsPipelineCreateInfo, ImageLayout, ImageUsageFlags, ImageView,
        InstanceCreateInfo, LogicOp, PhysicalDevice, Pipeline, PipelineBindPoint, PipelineCache,
        PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
        PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
        PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
        PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
        PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
        PrimitiveTopology, Rect2D, RenderPass, RenderPassCreateInfo, SampleCountFlags,
        ShaderStageFlags, SharingMode, SubpassDescription, SurfaceKHR, SwapchainCreateInfoKHR,
        SwapchainKHR, Viewport,
    },
    Device, Entry, Instance,
};
use log::info;
use winit::window::Window;

use crate::renderer::utils::api_version_tuple;

use self::{
    names::*,
    utils::{cstring_slice_to_raw, QueueFamilyIndices, Queues, SwapChainSupportDetails},
};

mod debug;
mod fp;
mod names;
mod utils;

// CONSTANTS
#[rustfmt::skip]
lazy_static! {
    pub static ref APPLICATION_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref APPLICATION_VERSION: u32 = make_api_version(0, 0, 0, 0);
    pub static ref ENGINE_NAME: CString = CString::new("OpenCubes").unwrap();
    pub static ref ENGINE_VERSION: u32 = make_api_version(0, 0, 0, 0);

    pub static ref API_VERSION_VARIANT: u32 = 0;
    pub static ref API_VERSION_MAJOR: u32 = 1;
    pub static ref API_VERSION_MINOR: u32 = 2;
    pub static ref API_VERSION_PATCH: u32 = 218;

    pub static ref API_VERSION: u32 = make_api_version(
        *API_VERSION_VARIANT,
        *API_VERSION_MAJOR,
        *API_VERSION_MINOR,
        *API_VERSION_PATCH
    );
    pub static ref API_VERSION_PATCHLESS: u32 = make_api_version(
        *API_VERSION_VARIANT,
        *API_VERSION_MAJOR,
        *API_VERSION_MINOR,
        0
    );
}

// INSTANCE LAYERS/EXTENSIONS
// DEBUG (OPTIONAL)
const VALIDATION_LAYERS: &[&CStr; 1] = &[VK_LAYER_KHRONOS_VALIDATION];
const DEBUG_EXTENSIONS: &[&CStr; 1] = &[VK_EXT_DEBUG_UTILS_EXTENSION];

// PHYSICAL DEVICE EXTENSIONS
// REQUIRED
const REQUIRED_DEVICE_EXTENSIONS: &[&CStr; 1] = &[VK_KHR_SWAPCHAIN];
// OPTIONAL
const OPTIONAL_DEVICE_EXTENSIONS: &[&CStr; 0] = &[];
const OPTIONAL_DEVICE_EXTENSIONS_RATING: &[u32; 0] = &[];

pub struct Renderer {
    entry: Entry,
    instance: Instance,

    instance_layers: Vec<CString>,
    instance_extensions: Vec<CString>,

    surface: SurfaceKHR,
    surface_loader: Surface,

    physical_device: PhysicalDevice,
    device: Device,
    device_extensions: Vec<CString>,

    queue_family_indices: QueueFamilyIndices,
    queues: Queues,

    swap_chain: SwapchainKHR,
    swap_chain_loader: Swapchain,
    swap_chain_format: ash::vk::Format,
    swap_chain_extent: Extent2D,

    swap_chain_images: Vec<ash::vk::Image>,
    image_views: Vec<ImageView>,

    render_pass: RenderPass,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Pipeline,

    debug_messenger: Option<DebugUtilsMessengerEXT>,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let entry = Entry::linked();
        let (instance, instance_layers, instance_extensions) =
            Renderer::create_instance(&entry, window);

        let mut debug_messenger = None;
        if cfg!(debug_assertions)
            && instance_extensions
                .contains(&CString::new(VK_EXT_DEBUG_UTILS_EXTENSION.to_bytes().clone()).unwrap())
        {
            debug_messenger = Some(debug::setup_debug_messenger(&entry, &instance.handle()));
        }

        let surface: SurfaceKHR =
            unsafe { ash_window::create_surface(&entry, &instance, window, None).unwrap() };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        let physical_device = unsafe {
            utils::pick_most_suitable_physical_device(
                &instance,
                &surface_loader,
                &surface,
                REQUIRED_DEVICE_EXTENSIONS,
                OPTIONAL_DEVICE_EXTENSIONS,
                OPTIONAL_DEVICE_EXTENSIONS_RATING,
            )
        };

        let mut device_extensions = Vec::new();
        unsafe {
            device_extensions.extend(utils::filter_supported_physical_device_extensions(
                &instance,
                &physical_device,
                OPTIONAL_DEVICE_EXTENSIONS,
            ));
            device_extensions.extend(utils::filter_supported_physical_device_extensions(
                &instance,
                &physical_device,
                REQUIRED_DEVICE_EXTENSIONS,
            ));
        }

        info!("Device Extensions used: {:?}", device_extensions);
        let queue_family_indices =
            utils::find_queue_families(&instance, &physical_device, &surface_loader, &surface);

        let device = unsafe {
            Renderer::create_logical_device(
                &instance,
                &surface_loader,
                &surface,
                &physical_device,
                &device_extensions,
            )
        };

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present_family.unwrap(), 0) };

        let queues = Queues {
            graphics_queue,
            present_queue,
        };

        let (
            swap_chain_loader,
            swap_chain,
            swap_chain_images,
            swap_chain_format,
            swap_chain_extent,
        ) = Renderer::create_swap_chain(
            &instance,
            &physical_device,
            &device,
            &surface_loader,
            &surface,
            window,
        );

        let image_views = utils::create_image_views(&device, &swap_chain_images, swap_chain_format);

        let render_pass = Renderer::create_render_pass(&device, swap_chain_format);
        let (pipeline_layout, graphics_pipeline) =
            Renderer::create_graphics_pipeline(&device, render_pass, swap_chain_extent);

        Renderer {
            entry,
            instance,
            instance_layers,
            instance_extensions,
            surface,
            surface_loader,
            physical_device,
            device,
            device_extensions,
            swap_chain,
            swap_chain_loader,
            swap_chain_images,
            image_views,
            swap_chain_format,
            swap_chain_extent,
            queue_family_indices,
            queues,
            debug_messenger,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> (Instance, Vec<CString>, Vec<CString>) {
        let version = match entry.try_enumerate_instance_version().unwrap() {
            Some(version) => version,
            None => make_api_version(0, 1, 0, 0),
        };

        if version < *API_VERSION_PATCHLESS {
            panic!(
                "Vulkan Instance version too low: {:?} vs {:?}",
                api_version_tuple(version),
                api_version_tuple(*API_VERSION_PATCHLESS)
            );
        }

        let app_info = ApplicationInfo::builder()
            .application_name(&APPLICATION_NAME)
            .application_version(*APPLICATION_VERSION)
            .engine_name(&ENGINE_NAME)
            .engine_version(*ENGINE_VERSION)
            .api_version(*API_VERSION);

        let mut create_info = InstanceCreateInfo::builder().application_info(&app_info);

        // Add Surface Extensions
        let _extensions_required_for_surface =
            ash_window::enumerate_required_extensions(window).unwrap();
        let mut extensions: Vec<CString> =
            unsafe { utils::raw_to_cstring(_extensions_required_for_surface) };

        // Add Validation Layers and Debug Extensions
        let mut layers: Vec<CString> = Vec::new();
        if cfg!(debug_assertions) {
            unsafe {
                layers = utils::filter_supported_layers(entry, VALIDATION_LAYERS);
                extensions.extend(utils::filter_supported_instance_extensions(
                    entry,
                    &utils::cstring_slice_to_cstr(&layers),
                    DEBUG_EXTENSIONS,
                ));
            }
        }

        // Add Instance Creation/Destruction Debugging
        let mut dbg_msg_create_info: DebugUtilsMessengerCreateInfoEXT;
        if cfg!(debug_assertions) {
            dbg_msg_create_info = *debug::get_debug_messenger_create_info();
            create_info = create_info.push_next(&mut dbg_msg_create_info);
        }

        let layers_raw = unsafe { utils::cstring_slice_to_raw(&layers) };
        create_info = create_info.enabled_layer_names(&layers_raw);

        let extensions_raw = unsafe { utils::cstring_slice_to_raw(&extensions) };
        create_info = create_info.enabled_extension_names(&extensions_raw);

        info!("INSTANCE LAYERS USED: {:?}", layers);
        info!("INSTANCE EXTENSIONS USED: {:?}", extensions);

        (
            unsafe { entry.create_instance(&create_info, None).unwrap() },
            layers,
            extensions,
        )
    }

    unsafe fn create_logical_device(
        instance: &Instance,
        surface_loader: &Surface,
        surface: &SurfaceKHR,
        physical_device: &PhysicalDevice,
        extensions: &[CString],
    ) -> Device {
        let queue_family_indices =
            utils::find_queue_families(instance, physical_device, &surface_loader, &surface);
        let unique_indices = queue_family_indices.get_unique_indices();
        let mut create_infos = Vec::new();
        for unique in unique_indices {
            let mut create_info = DeviceQueueCreateInfo::builder()
                .queue_family_index(unique)
                .queue_priorities(&[1.0]);
            create_info.queue_count = 1;
            create_infos.push(*create_info);
        }

        let slice = cstring_slice_to_raw(extensions);
        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&create_infos)
            .enabled_extension_names(&slice);

        instance
            .create_device(*physical_device, &device_create_info, None)
            .unwrap()
    }

    fn create_swap_chain(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        surface_loader: &Surface,
        surface: &SurfaceKHR,
        window: &Window,
    ) -> (
        Swapchain,
        SwapchainKHR,
        Vec<ash::vk::Image>,
        Format,
        Extent2D,
    ) {
        let swap_chain_support_details =
            SwapChainSupportDetails::query_from(instance, physical_device, surface_loader, surface);

        let format = swap_chain_support_details.choose_format();
        let present_mode = swap_chain_support_details.choose_presentation_mode();
        let extent = swap_chain_support_details.choose_swap_extend(window);
        let image_count = swap_chain_support_details.choose_image_count();

        let mut create_info = SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(swap_chain_support_details.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let indices =
            utils::find_queue_families(instance, physical_device, surface_loader, surface);
        let indices_arr = [
            indices.graphics_family.unwrap(),
            indices.present_family.unwrap(),
        ];

        if indices_arr[0] != indices_arr[1] {
            create_info = create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&indices_arr);
        } else {
            create_info = create_info.image_sharing_mode(SharingMode::EXCLUSIVE);
        }

        let swap_chain_loader = Swapchain::new(instance, device);
        let swap_chain = unsafe {
            swap_chain_loader
                .create_swapchain(&create_info, None)
                .unwrap()
        };

        let images = unsafe { swap_chain_loader.get_swapchain_images(swap_chain).unwrap() };

        (swap_chain_loader, swap_chain, images, format.format, extent)
    }

    fn create_render_pass(device: &Device, format: Format) -> RenderPass {
        let color_attachment = AttachmentDescription::builder()
            .format(format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachment_refs = [*color_attachment_ref];
        let subpass = SubpassDescription::builder()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        let color_attachments = [*color_attachment];
        let subpasses = [*subpass];
        let render_pass_create_info = RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses);

        unsafe {
            device
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        }
    }

    fn create_graphics_pipeline(
        device: &Device,
        render_pass: RenderPass,
        extent: Extent2D,
    ) -> (PipelineLayout, Pipeline) {
        let vert_shader_code = include_bytes!("./shaders/base_shader_vert.spv");
        let frag_shader_code = include_bytes!("./shaders/base_shader_frag.spv");

        let vert_shader = utils::create_shader_module(device, vert_shader_code);
        let frag_shader = utils::create_shader_module(device, frag_shader_code);

        let create_info_vert_shader_stage_name = CString::new("main").unwrap();
        let mut create_info_vertex_shader_stage = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_shader)
            .name(&create_info_vert_shader_stage_name);

        let create_info_frag_shader_stage_name = CString::new("main").unwrap();
        let mut create_info_fragment_shader_stage = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_shader)
            .name(&create_info_frag_shader_stage_name);

        let shader_stages = [
            *create_info_vertex_shader_stage,
            *create_info_fragment_shader_stage,
        ];

        let vertex_input_info = PipelineVertexInputStateCreateInfo::builder();

        let input_assembly = PipelineInputAssemblyStateCreateInfo::builder()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let view_port = Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = Rect2D::builder()
            .offset(ash::vk::Offset2D { x: 0, y: 0 })
            .extent(extent.clone());

        let _viewports = [*view_port];
        let _scissors = [*scissor];
        let viewport_create_info = PipelineViewportStateCreateInfo::builder()
            .viewports(&_viewports)
            .scissors(&_scissors);

        let rasterizer_create_info = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisample_create_info = PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachment = PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                ColorComponentFlags::R
                    | ColorComponentFlags::G
                    | ColorComponentFlags::B
                    | ColorComponentFlags::A,
            )
            .blend_enable(false);

        let _color_bend_attachments = [*color_blend_attachment];
        let color_blend_create_info = PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(&_color_bend_attachments);

        let dynamic_states = vec![DynamicState::VIEWPORT, DynamicState::LINE_WIDTH];
        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let pipeline_layout_create_info = PipelineLayoutCreateInfo::builder();

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };

        let pipeline_create_info = GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&&input_assembly)
            .viewport_state(&viewport_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisample_create_info)
            .color_blend_state(&color_blend_create_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let create_infos = [*pipeline_create_info];
        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(PipelineCache::null(), &create_infos, None)
                .unwrap()
        };

        unsafe {
            device.destroy_shader_module(vert_shader, None);
            device.destroy_shader_module(frag_shader, None);
        }

        (pipeline_layout, graphics_pipelines[0])
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(dbg_msg) = self.debug_messenger {
                let destroy_fn =
                    fp::get_destroy_debug_utils_messenger_fp(&self.entry, &self.instance.handle());
                destroy_fn(self.instance.handle(), dbg_msg, std::ptr::null());
            }
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.swap_chain_loader
                .destroy_swapchain(self.swap_chain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
