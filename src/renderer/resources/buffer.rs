use ash::vk;

pub struct Buffer {
    pub mem: vk::DeviceMemory,
    pub buf: vk::Buffer,
}

impl Buffer {
    #[inline]
    pub fn new(
        device: &ash::Device,
        buffer_size: u64,
        buffer_usage: vk::BufferUsageFlags,
        memory_props: vk::PhysicalDeviceMemoryProperties,
        buffer_props: vk::MemoryPropertyFlags,
    ) -> Result<Self, String> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .map_err(|msg| format!("{}", msg))?
        };

        let mem_req = unsafe { device.get_buffer_memory_requirements(buffer) };

        let mem_type_index =
            match find_memory_type_index(memory_props, mem_req.memory_type_bits, buffer_props) {
                Some(mem_type_ind) => mem_type_ind,
                None => return Err(String::from("Couldn't find suitable memory type (you really should have found one, so most likely the problem is with you, or with the person who wrote the buffer creation, and in that case it is me. So basically fuck me)")),
            };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_req.size)
            .memory_type_index(mem_type_index);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .map_err(|msg| format!("{}", msg))?
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|msg| format!("{}", msg))?
        };

        Ok(Self {
            buf: buffer,
            mem: memory,
        })
    }

    #[inline]
    pub fn copy(
        &self,
        device: &ash::Device,
        dst_buffer: &Buffer,
        buffer_size: u64,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
    ) -> Result<(), String> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .map_err(|msg| format!("{}", msg))?[0]
        };

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let buffer_copy_region = vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(buffer_size)
            .build();

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .build();

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|msg| format!("{}", msg))?;

            device.cmd_copy_buffer(
                command_buffer,
                self.buf,
                dst_buffer.buf,
                &[buffer_copy_region],
            );

            device
                .end_command_buffer(command_buffer)
                .map_err(|msg| format!("{}", msg))?;

            device
                .queue_submit(queue, &[submit_info], vk::Fence::null())
                .map_err(|msg| format!("{}", msg))?;
            device
                .queue_wait_idle(queue)
                .map_err(|msg| format!("{}", msg))?;
            device.free_command_buffers(command_pool, &[command_buffer]);
        }

        Ok(())
    }

    #[inline]
    fn free(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buf, None);
            device.free_memory(self.mem, None);
        }
    }
}

fn find_memory_type_index(
    memory_props: vk::PhysicalDeviceMemoryProperties,
    allowed_types: u32,
    props: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_props
        .memory_types
        .iter()
        .enumerate()
        .find(|(i, mem_type)| {
            allowed_types & (1 << i) != 0 && mem_type.property_flags & props == props
        })
        .map(|(i, _)| i as u32)
}
