// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::*;

pub struct CommandBuffer {
    pub command_buffer: vk::CommandBuffer,
    pool: vk::CommandPool,
    pub device: Arc<ash::Device>,
}

impl CommandBuffer {
    pub fn new(pool: &CommandPool) -> Self {
        let create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let mut command_buffers = unsafe { pool.device.allocate_command_buffers(&create_info) }
            .expect("Failed to create Vulkan command buffer");
        let command_buffer = command_buffers.pop().unwrap();

        Self {
            command_buffer,
            pool: pool.pool,
            device: pool.device.clone(),
        }
    }

    pub fn begin(&self, flags: vk::CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);
        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)
        }
        .expect("Failed to begin Vulkan command buffer");
    }

    pub fn begin_render_pass(&self, pass: &Pass, framebuffer: &Framebuffer, area: Size2) {
        let area = vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(
                vk::Extent2D::default()
                    .width(area.width)
                    .height(area.height),
            );

        let mut present_clear = vk::ClearValue::default();
        present_clear.color.float32 = [0.0, 10.0 / 255.0, 28.0 / 255.0, 1.0];

        let mut depth_clear = vk::ClearValue::default();
        depth_clear.depth_stencil.depth = 0.0;
        depth_clear.depth_stencil.stencil = 0;

        let mut color_clear = vk::ClearValue::default();
        color_clear.color.float32 = [0.2, 0.0, 0.1, 1.0];

        let mut normal_clear = vk::ClearValue::default();
        normal_clear.color.float32 = [0.0, 0.0, 0.0, 1.0];

        let clear_values = [present_clear, depth_clear, color_clear, normal_clear];
        let create_info = vk::RenderPassBeginInfo::default()
            .framebuffer(framebuffer.framebuffer)
            .render_pass(pass.render)
            .render_area(area)
            .clear_values(&clear_values);
        // Record it in the main command buffer
        let contents = vk::SubpassContents::INLINE;
        unsafe {
            self.device
                .cmd_begin_render_pass(self.command_buffer, &create_info, contents)
        };
    }

    pub fn next_subpass(&self) {
        unsafe {
            self.device
                .cmd_next_subpass(self.command_buffer, vk::SubpassContents::INLINE)
        };
    }

    pub fn set_viewport(&self, viewport: vk::Viewport) {
        unsafe {
            self.device
                .cmd_set_viewport(self.command_buffer, 0, &[viewport])
        };
    }

    pub fn set_scissor(&self, scissor: vk::Rect2D) {
        unsafe {
            self.device
                .cmd_set_scissor(self.command_buffer, 0, &[scissor])
        }
    }

    pub fn bind_pipeline(&self, pipeline: vk::Pipeline) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;
        unsafe {
            self.device
                .cmd_bind_pipeline(self.command_buffer, graphics_bind_point, pipeline);
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        layout: vk::PipelineLayout,
        sets: &[vk::DescriptorSet],
        set_index: u32,
    ) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                graphics_bind_point,
                layout,
                set_index,
                sets,
                &[],
            )
        };
    }

    pub fn bind_vertex_buffer(&self, buffer: &RenderBuffer) {
        let first_binding = 0;
        let buffers = [buffer.buffer];
        let offsets = [vk::DeviceSize::default()];
        unsafe {
            self.device.cmd_bind_vertex_buffers(
                self.command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
        }
    }

    pub fn bind_index_buffer(&self, buffer: &RenderBuffer, index_type: vk::IndexType) {
        unsafe {
            self.device
                .cmd_bind_index_buffer(self.command_buffer, buffer.buffer, 0, index_type);
        }
    }

    pub fn push_constants(
        &self,
        pipeline: &impl Pipeline,
        stages: vk::ShaderStageFlags,
        offset: u32,
        constants: &[u8],
    ) {
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                pipeline.get_layout(),
                stages,
                offset,
                constants,
            )
        }
    }

    pub fn draw_indexed(&self, index_count: u32, index_offset: u32, vertex_offset: i32) {
        unsafe {
            self.device.cmd_draw_indexed(
                self.command_buffer,
                index_count,
                1,
                index_offset,
                vertex_offset,
                0,
            );
        }
    }

    pub fn draw(&self, vertex_count: u32) {
        unsafe {
            self.device
                .cmd_draw(self.command_buffer, vertex_count, 1, 0, 0);
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device.cmd_end_render_pass(self.command_buffer);
        }
    }

    pub fn end(&self) {
        unsafe { self.device.end_command_buffer(self.command_buffer) }
            .expect("Failed to end command buffer");
    }

    pub fn pipeline_barriers(
        &self,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                &[],
                &[],
                image_memory_barriers,
            );
        }
    }

    pub fn copy_buffer_to_image(
        &self,
        buffer: &RenderBuffer,
        image: &RenderImage,
        region: &vk::BufferImageCopy,
    ) {
        unsafe {
            self.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                buffer.buffer,
                image.image,
                image.layout,
                &[*region],
            );
        }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .free_command_buffers(self.pool, &[self.command_buffer])
        }
    }
}

pub struct CommandPool {
    pool: vk::CommandPool,
    pub device: Arc<ash::Device>,
}

impl CommandPool {
    pub fn new(device: &Device) -> Self {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.graphics_queue_index);

        let pool = {
            unsafe {
                device
                    .device
                    .create_command_pool(&create_info, None)
                    .expect("Failed to create Vulkan command pool")
            }
        };

        Self {
            pool,
            device: device.device.clone(),
        }
    }

    pub fn destroy(&mut self) {
        if self.pool != vk::CommandPool::null() {
            unsafe {
                self.device.destroy_command_pool(self.pool, None);
            }
            self.pool = vk::CommandPool::null();
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        self.destroy();
    }
}
