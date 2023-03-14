use ash::vk;
use winit::window::Window;

use self::{base::RenderBase, data::RenderData, utils::MAX_FRAME_DRAWS};

pub mod base;
pub mod data;
mod draw_setup;
mod resources;
pub mod utils;

pub struct Renderer {
    pub data: RenderData,
    pub base: RenderBase,

    pub current_frame_index: usize,
    pub rebuild_swapchain: bool,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self, String> {
        let mut base = RenderBase::new(window)?;
        let mut data = RenderData::new(&mut base)?;

        Ok(Self {
            base,
            data,
            current_frame_index: 0,
            rebuild_swapchain: true,
        })
    }

    #[inline]
    pub fn draw(&mut self) -> Result<(), String> {
        let image_index = match self.get_img_index()? {
            Some(index) => index,
            None => {
                self.rebuild_swapchain = true;
                return Ok(());
            }
        };

        self.wait_resource_available()?;
        self.begin_command_buffer()?;
        self.begin_render_pass();
        self.set_viewport();
        self.set_scissor();
        unsafe {
            self.base
                .device
                .cmd_end_render_pass(self.data.command_buffers[self.current_frame_index]);

            self.base
                .device
                .end_command_buffer(self.data.command_buffers[self.current_frame_index])
                .map_err(|_| String::from("failed to end command buffer"))?
        }
        self.submit()?;

        if !self.present()? {
            self.rebuild_swapchain = true;
            return Ok(());
        }
        Ok(())
    }
}
