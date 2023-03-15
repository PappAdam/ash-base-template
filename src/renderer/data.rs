use crate::create_shader;
use ash::vk;

use super::{
    base::RenderBase,
    resources::{self, create_framebuffers},
    utils::MAX_FRAME_DRAWS,
};

pub struct RenderData {
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    // pub index_buffer: Buffer,
    // pub index_count: u32,
    // pub vertex_buffer: Buffer,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub img_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub fences: Vec<vk::Fence>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl RenderData {
    pub fn new(base: &mut RenderBase) -> Result<Self, String> {
        let vertex_shader_module = create_shader!("../shaders/vert.spv", base.device);
        let fragment_shader_module = create_shader!("../shaders/frag.spv", base.device);

        let pipeline_layout = resources::create_pipeline_layout(&base.device)?;

        let render_pass = resources::create_render_pass(&base.device, base.surface_format.format)?;

        let pipeline = resources::create_pipelines(
            &base.device,
            vertex_shader_module,
            fragment_shader_module,
            pipeline_layout,
            render_pass,
        )?;

        let framebuffers = resources::create_framebuffers(
            &base.device,
            &base.swapchain_image_views,
            render_pass,
            base.surface_extent,
        )?;

        let img_available_semaphores =
            resources::create_semaphore(&base.device, "img available semaphore")?;

        let render_finished_semaphores =
            resources::create_semaphore(&base.device, "rendering finished semaphore")?;

        let fences = resources::create_fences(&base.device)?;
        let command_pool = resources::create_command_pool(&base.device, base.queue_family)?;

        let command_buffers = {
            unsafe {
                let cb_info = vk::CommandBufferAllocateInfo::builder()
                    .command_pool(command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(MAX_FRAME_DRAWS as u32)
                    .build();
                base.device
                    .allocate_command_buffers(&cb_info)
                    .map_err(|err| format!("{}", err))?
            }
        };

        Ok(Self {
            vertex_shader_module,
            fragment_shader_module,
            // index_buffer: todo!(),
            // index_count: todo!(),
            // vertex_buffer: todo!(),
            pipeline_layout,
            render_pass,
            pipeline,
            framebuffers,
            img_available_semaphores,
            render_finished_semaphores,
            fences,
            command_pool,
            command_buffers,
        })
    }

    #[inline]
    pub fn resize(&mut self, vulkan_base: &RenderBase) -> Result<(), String> {
        unsafe {
            for &framebuffer in &self.framebuffers {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }
        }

        self.framebuffers = create_framebuffers(
            &vulkan_base.device,
            &vulkan_base.swapchain_image_views,
            self.render_pass,
            vulkan_base.surface_extent,
        )?;

        Ok(())
    }
}
