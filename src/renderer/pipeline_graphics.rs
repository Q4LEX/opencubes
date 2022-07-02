use std::ffi::CString;

use ash::vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    ColorComponentFlags, CullModeFlags, FrontFace, GraphicsPipelineCreateInfo, ImageLayout,
    Offset2D, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentState,
    PipelineColorBlendStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineStageFlags,
    PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, Rect2D, RenderPass, RenderPassCreateInfo, SampleCountFlags,
    ShaderStageFlags, SubpassDependency, SubpassDescription, Viewport,
};

use super::{device::Device, shader_module::ShaderModule, swapchain::SwapChain};

pub struct GraphicsPipeline {
    pub inner: ash::vk::Pipeline,
    pub pipeline_layout: PipelineLayout,
    pub render_pass: RenderPass,
    pub device: ash::Device,
}

impl GraphicsPipeline {
    pub fn new(device: &Device, swapchain: &SwapChain) -> Self {
        let attachment_description = AttachmentDescription::builder()
            .format(swapchain.surface_format.format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let attachment_reference = AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachment_refs = [attachment_reference.build()];
        let subpass_description = SubpassDescription::builder()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        let color_attachments = [attachment_description.build()];

        let subpass_dependency = SubpassDependency::builder()
            .src_subpass(ash::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE);

        let subpass_dependencies = [subpass_dependency.build()];
        let subpasses = [subpass_description.build()];
        let render_pass_create_info = RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            device
                .inner
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        let vert_shader_module =
            ShaderModule::new(device, include_bytes!("shaders/base_shader_vert.spv"));
        let frag_shader_module =
            ShaderModule::new(device, include_bytes!("shaders/base_shader_frag.spv"));

        // VERTEX
        let vert_p_name = CString::new("main").unwrap();
        let vert_create_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_shader_module.inner)
            .name(&vert_p_name);

        // FRAGMENT
        let frag_p_name = CString::new("main").unwrap();
        let frag_create_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.inner)
            .name(&frag_p_name);

        let vertex_input_create_info = PipelineVertexInputStateCreateInfo::builder();

        let input_assembly_create_info = PipelineInputAssemblyStateCreateInfo::builder()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = Rect2D::builder()
            .offset(Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent);

        let viewports = [viewport.build()];
        let scissors = [scissor.build()];
        let viewport_create_info = PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_create_info = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_create_info = PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1);

        let color_blend_attachment = PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                ColorComponentFlags::R
                    | ColorComponentFlags::G
                    | ColorComponentFlags::B
                    | ColorComponentFlags::A,
            )
            .blend_enable(false);

        let color_blend_attachments = [color_blend_attachment.build()];
        let color_blend_create_info = PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let pipeline_layout_create_info = PipelineLayoutCreateInfo::builder();

        let pipeline_layout = unsafe {
            device
                .inner
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };

        let shader_stage_create_infos = vec![vert_create_info.build(), frag_create_info.build()];
        let create_info = GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisample_create_info)
            .color_blend_state(&color_blend_create_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let create_infos = [create_info.build()];
        let inner = unsafe {
            device
                .inner
                .create_graphics_pipelines(PipelineCache::null(), &create_infos, None)
                .unwrap()[0]
        };

        Self {
            inner,
            pipeline_layout,
            render_pass,
            device: device.inner.clone(),
        }
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.inner, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
