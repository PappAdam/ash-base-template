use ash::vk;

use super::Renderer;

impl Renderer {
    #[inline]
    pub fn record_command_buffer() {}

    #[inline]
    pub fn begin_render_pass(&self) {
        let clear_color = vk::ClearColorValue {
            float32: [0.5f32, 0.5f32, 0.5f32, 1.0f32],
        };
        let clear_values = vec![vk::ClearValue { color: clear_color }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[self.image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.base.surface_extent,
            })
            .clear_values(&clear_values)
            .build();

        unsafe {
            self.base.device.cmd_begin_render_pass(
                self.data.command_buffers[self.current_frame_index],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    #[inline]
    pub fn begin_command_buffer(&self) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.base
                .device
                .begin_command_buffer(
                    self.data.command_buffers[self.current_frame_index],
                    &begin_info,
                )
                .map_err(|_| String::from("failed to begin command buffer"))?;
        }

        Ok(())
    }

    #[inline]
    pub fn get_img_index(&self) -> Result<Option<u32>, String> {
        let (index, is_suboptimal) = match unsafe {
            self.base.swapchain_loader.acquire_next_image(
                self.base.swapchain,
                u64::MAX,
                self.data.img_available_semaphore,
                vk::Fence::null(),
            )
        } {
            Ok((index, is_suboptimal)) => (index, is_suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok(None),
            Err(_) => return Err(String::from("failed to acquire next image")),
        };

        if is_suboptimal {
            return Ok(None);
        }

        Ok(Some(index))
    }
    #[inline]
    pub fn present(&self) -> Result<bool, String> {
        let semaphores = [self.data.render_finished_semaphore];
        let swapchains = [self.base.swapchain];
        let indices = [self.image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices)
            .build();

        unsafe {
            if let Err(err) = self
                .base
                .swapchain_loader
                .queue_present(self.base.queue, &present_info)
            {
                if err == vk::Result::SUBOPTIMAL_KHR || err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    return Ok(false);
                } else {
                    return Err(String::from("failed to present"));
                }
            }
        }

        Ok(true)
    }
    #[inline]
    pub fn set_scissor(&self) {
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.base.surface_extent.width,
                height: self.base.surface_extent.height,
            },
        };

        unsafe {
            self.base.device.cmd_set_scissor(
                self.data.command_buffers[self.current_frame_index],
                0,
                &[scissor],
            );
        }
    }
    #[inline]
    pub fn set_viewport(&self) {
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.base.surface_extent.width as f32,
            height: self.base.surface_extent.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32,
        };

        unsafe {
            self.base.device.cmd_set_viewport(
                self.data.command_buffers[self.current_frame_index],
                0,
                &[viewport],
            );
        }
    }
    #[inline]
    pub fn submit(&self) -> Result<(), String> {
        let fence = self.data.fences[self.current_frame_index as usize];

        let wait_semaphores = [self.data.img_available_semaphore];
        let masks = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmd_buffers = [self.data.command_buffers[self.current_frame_index]];
        let signal_semaphores = [self.data.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&masks)
            .command_buffers(&cmd_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            self.base
                .device
                .queue_submit(self.base.queue, &[submit_info], fence)
                .map_err(|_| String::from("failed to submit graphics command buffer"))?
        }

        Ok(())
    }
    #[inline]
    pub fn wait_resource_available(&self) -> Result<(), String> {
        let fence = self.data.fences[self.current_frame_index as usize];

        unsafe {
            self.base
                .device
                .wait_for_fences(&[fence], true, u64::MAX)
                .map_err(|_| {
                    format!(
                        "failed to wait for resource fence {}",
                        self.current_frame_index
                    )
                })?;

            self.base.device.reset_fences(&[fence]).map_err(|_| {
                format!(
                    "failed to reset resource fence {}",
                    self.current_frame_index
                )
            })?;
        }

        Ok(())
    }
}
