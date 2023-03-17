pub mod buffer;

use ash::vk;

use super::utils::MAX_FRAME_DRAWS;

pub fn create_render_pass(
    device: &ash::Device,
    surface_format: vk::Format,
) -> Result<vk::RenderPass, String> {
    let mut attachment_descriptions = Vec::new();

    attachment_descriptions.push(
        vk::AttachmentDescription::builder()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build(),
    );

    let col_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let references = [col_attachment_ref];

    let mut subpass_descriptions = Vec::new();

    subpass_descriptions.push(
        vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&references)
            .build(),
    );

    let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachment_descriptions)
        .subpasses(&subpass_descriptions);

    let render_pass = unsafe {
        device
            .create_render_pass(&create_info, None)
            .map_err(|_| String::from("failed to create render pass"))?
    };

    Ok(render_pass)
}

pub fn create_pipelines(
    device: &ash::Device,
    vertex_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> Result<vk::Pipeline, String> {
    let shader_entry_name = std::ffi::CString::new("main").unwrap();

    let vs_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(&shader_entry_name)
        .build();

    let fs_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragment_shader_module)
        .name(&shader_entry_name)
        .build();

    let ia_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .build();

    let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0f32)
        .build();

    let col_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .build();

    let attachments = [col_blend_attachment_state];
    let col_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&attachments)
        .build();

    // let states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let viewports = [vk::Viewport {
        ..Default::default()
    }];
    let scissors = [vk::Rect2D {
        ..Default::default()
    }];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewports)
        .scissors(&scissors)
        .build();

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let stages = [vs_state, fs_state];

    // let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&states);

    let vert_inp_state = vk::PipelineVertexInputStateCreateInfo::builder().build();

    let solid_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .flags(vk::PipelineCreateFlags::ALLOW_DERIVATIVES)
        .stages(&stages)
        .input_assembly_state(&ia_state)
        .rasterization_state(&raster_state)
        .color_blend_state(&col_blend_state)
        .viewport_state(&viewport_state)
        .layout(pipeline_layout)
        // .dynamic_state(&dynamic_state_info)
        .render_pass(render_pass)
        .subpass(0)
        .multisample_state(&multisample_state)
        .vertex_input_state(&vert_inp_state)
        .build();

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[solid_pipeline_create_info],
                None,
            )
            .map_err(|_| String::from("failed to create pipelines"))?
    };

    let pipeline = pipelines[0];

    Ok(pipeline)
}

pub fn create_pipeline_layout(
    device: &ash::Device,
    // descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, String> {
    // let layouts = [descriptor_set_layout];
    let create_info = vk::PipelineLayoutCreateInfo::builder()
        // .set_layouts(&layouts)
        .build();

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&create_info, None)
            .map_err(|_| String::from("failed to create pipeline layout"))?
    };

    Ok(pipeline_layout)
}

pub fn create_framebuffers(
    device: &ash::Device,
    swapchain_image_views: &Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    framebuffer_extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>, String> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for (i, &view) in swapchain_image_views.iter().enumerate() {
        let attachments = [view];

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(framebuffer_extent.width)
            .height(framebuffer_extent.height)
            .layers(1)
            .build();

        let framebuffer = unsafe {
            device.create_framebuffer(&create_info, None).map_err(|_| {
                for &fb in &framebuffers {
                    device.destroy_framebuffer(fb, None);
                }
                format!("failed to create framebuffer {}", i)
            })?
        };

        framebuffers.push(framebuffer);
    }

    Ok(framebuffers)
}

pub fn create_semaphore(
    device: &ash::Device,
    object_name: &str,
) -> Result<Vec<vk::Semaphore>, String> {
    let mut semaphores = Vec::<vk::Semaphore>::with_capacity(MAX_FRAME_DRAWS);

    for _ in 0..MAX_FRAME_DRAWS {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            device
                .create_semaphore(&create_info, None)
                .map_err(|_| format!("failed to create {}", object_name))?
        };

        semaphores.push(semaphore);
    }

    Ok(semaphores)
}

pub fn create_fences(device: &ash::Device) -> Result<Vec<vk::Fence>, String> {
    let create_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED)
        .build();

    let mut fences = Vec::with_capacity(MAX_FRAME_DRAWS as usize);

    for i in 0..MAX_FRAME_DRAWS {
        let fence = unsafe {
            device.create_fence(&create_info, None).map_err(|_| {
                for &f in &fences {
                    device.destroy_fence(f, None);
                }

                format!("failed to create fence {}", i)
            })?
        };

        fences.push(fence);
    }

    Ok(fences)
}

pub fn create_command_pool(
    device: &ash::Device,
    queue_family: u32,
) -> Result<vk::CommandPool, String> {
    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family);

    let command_pool = unsafe {
        device
            .create_command_pool(&create_info, None)
            .expect("Failed to create command pool")
    };

    Ok(command_pool)
}
