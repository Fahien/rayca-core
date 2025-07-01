// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::rc::Rc;

use super::*;

/// This is the one that is going to be recreated
/// when the swapchain goes out of date
pub struct Framebuffer {
    pub area: vk::Rect2D,
    // @todo Make a map of framebuffers indexed by render-pass as key
    pub framebuffer: vk::Framebuffer,
    pub image_view: vk::ImageView,
    device: Rc<ash::Device>,
}

impl Framebuffer {
    pub fn new(dev: &mut Dev, image: &Image, pass: &Pass) -> Self {
        // Image view into a swapchain images (device, image, format)
        let image_view = {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(image.format)
                .components(
                    vk::ComponentMapping::default()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );
            unsafe { dev.device.create_image_view(&create_info, None) }
                .expect("Failed to create Vulkan image view")
        };

        // Framebuffers (image_view, renderpass)
        let framebuffer = {
            let attachments = [image_view];

            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(pass.render)
                .attachments(&attachments)
                .width(image.width)
                .height(image.height)
                .layers(1);

            unsafe { dev.device.create_framebuffer(&create_info, None) }
                .expect("Failed to create Vulkan framebuffer")
        };

        // Needed by cmd_begin_render_pass
        let area = vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(
                vk::Extent2D::default()
                    .width(image.width)
                    .height(image.height),
            );

        Self {
            area,
            framebuffer,
            image_view,
            device: dev.device.device.clone(),
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device");
            self.device.destroy_framebuffer(self.framebuffer, None);
            self.device.destroy_image_view(self.image_view, None);
        }
    }
}

/// Frame resources that do not need to be recreated
/// when the swapchain goes out of date
pub struct FrameCache {
    pub command_buffer: vk::CommandBuffer,
    pub fence: vk::Fence,
    pub can_wait: bool,
    pub image_ready: vk::Semaphore,
    pub image_drawn: vk::Semaphore,
    device: Rc<ash::Device>,
}

impl FrameCache {
    pub fn new(dev: &mut Dev) -> Self {
        // Graphics command buffer (device, command pool)
        let command_buffer = {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(dev.graphics_command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let buffers = unsafe { dev.device.allocate_command_buffers(&alloc_info) }
                .expect("Failed to allocate command buffer");
            buffers[0]
        };

        // Fence (device)
        let fence = {
            let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            unsafe { dev.device.create_fence(&create_info, None) }
        }
        .expect("Failed to create Vulkan fence");

        // Semaphores (device)
        let (image_ready, image_drawn) = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe {
                (
                    dev.device
                        .create_semaphore(&create_info, None)
                        .expect("Failed to create Vulkan semaphore"),
                    dev.device
                        .create_semaphore(&create_info, None)
                        .expect("Failed to create Vulkan semaphore"),
                )
            }
        };

        Self {
            command_buffer,
            fence,
            can_wait: true,
            image_ready,
            image_drawn,
            device: dev.device.device.clone(),
        }
    }

    pub fn wait(&mut self) {
        if !self.can_wait {
            return;
        }

        unsafe {
            self.device
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .expect("Failed to wait for Vulkan frame fence");
            self.device
                .reset_fences(&[self.fence])
                .expect("Failed to reset Vulkan frame fence");
        }
        self.can_wait = false;
    }
}

impl Drop for FrameCache {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_semaphore(self.image_drawn, None);
            self.device.destroy_semaphore(self.image_ready, None);
            self.device.destroy_fence(self.fence, None)
        }
    }
}

pub struct Frame {
    pub buffer: Framebuffer,
    pub cache: FrameCache,
    pub device: Rc<ash::Device>,
}

impl Frame {
    pub fn new(dev: &mut Dev, image: &Image, pass: &Pass) -> Self {
        let buffer = Framebuffer::new(dev, image, pass);
        let cache = FrameCache::new(dev);

        Frame {
            buffer,
            cache,
            device: dev.device.device.clone(),
        }
    }

    pub fn begin(&self, pass: &Pass) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device
                .begin_command_buffer(self.cache.command_buffer, &begin_info)
        }
        .expect("Failed to begin Vulkan command buffer");

        let mut clear = vk::ClearValue::default();
        clear.color.float32 = [0.025, 0.025, 0.025, 1.0];
        let clear_values = [clear];
        let create_info = vk::RenderPassBeginInfo::default()
            .framebuffer(self.buffer.framebuffer)
            .render_pass(pass.render)
            .render_area(self.buffer.area)
            .clear_values(&clear_values);
        // Record it in the main command buffer
        let contents = vk::SubpassContents::INLINE;
        unsafe {
            self.device
                .cmd_begin_render_pass(self.cache.command_buffer, &create_info, contents)
        };
    }

    pub fn end(&self) {
        unsafe {
            self.device.cmd_end_render_pass(self.cache.command_buffer);
            self.device
                .end_command_buffer(self.cache.command_buffer)
                .expect("Failed to end command buffer");
        }
    }

    pub fn present(
        &mut self,
        dev: &Dev,
        swapchain: &Swapchain,
        image_index: u32,
    ) -> Result<(), vk::Result> {
        // Wait for the image to be available ..
        let wait_semaphores = [self.cache.image_ready];
        // .. at color attachment output stage
        let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.cache.command_buffer];
        let signal_semaphores = [self.cache.image_drawn];
        let submits = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];
        unsafe {
            self.device
                .queue_submit(dev.graphics_queue, &submits, self.cache.fence)
        }
        .expect("Failed to submit to Vulkan queue");

        self.cache.can_wait = true;

        // Present result
        let pres_image_indices = [image_index];
        let pres_swapchains = [swapchain.swapchain];
        let pres_semaphores = [self.cache.image_drawn];
        let present_info = vk::PresentInfoKHR::default()
            .image_indices(&pres_image_indices)
            .swapchains(&pres_swapchains)
            .wait_semaphores(&pres_semaphores);

        match unsafe {
            swapchain
                .ext
                .queue_present(dev.graphics_queue, &present_info)
        } {
            Ok(_subotimal) => Ok(()),
            Err(result) => Err(result),
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device");
        }
    }
}

pub trait Frames {
    fn next_frame(&mut self) -> Result<&mut Frame, vk::Result>;
    fn present(&mut self, dev: &Dev) -> Result<(), vk::Result>;
}

/// Offscreen frames work on user allocated images
struct _OffscreenFrames {
    _frames: Vec<Frame>,
    _images: Vec<vk::Image>,
}

impl Frames for _OffscreenFrames {
    fn next_frame(&mut self) -> Result<&mut Frame, vk::Result> {
        // Unimplemented
        Err(vk::Result::ERROR_UNKNOWN)
    }

    fn present(&mut self, _dev: &Dev) -> Result<(), vk::Result> {
        // Unimplemented
        Err(vk::Result::ERROR_UNKNOWN)
    }
}

/// Swapchain frames work on swapchain images
pub struct SwapchainFrames {
    pub current: usize,
    image_index: u32,
    pub frames: Vec<Frame>,
    pub swapchain: Swapchain,
}

impl SwapchainFrames {
    pub fn new(
        ctx: &Ctx,
        surface: &Surface,
        dev: &mut Dev,
        width: u32,
        height: u32,
        pass: &Pass,
    ) -> Self {
        let swapchain = Swapchain::new(ctx, surface, dev, width, height);

        let mut frames = Vec::new();
        for image in &swapchain.images {
            let frame = Frame::new(dev, image, pass);
            frames.push(frame);
        }

        Self {
            current: 0,
            image_index: 0,
            frames,
            swapchain,
        }
    }
}

impl Frames for SwapchainFrames {
    fn next_frame(&mut self) -> Result<&mut Frame, vk::Result> {
        // Wait for this frame to be ready
        let frame = &mut self.frames[self.current];
        frame.cache.wait();

        let acquire_res = unsafe {
            self.swapchain.ext.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                frame.cache.image_ready,
                vk::Fence::null(),
            )
        };

        match acquire_res {
            Ok((image_index, _)) => {
                self.image_index = image_index;
                Ok(frame)
            }
            Err(result) => {
                self.current = 0;
                Err(result)
            }
        }
    }

    fn present(&mut self, dev: &Dev) -> Result<(), vk::Result> {
        match self.frames[self.current].present(dev, &self.swapchain, self.image_index) {
            Ok(()) => {
                self.current = (self.current + 1) % self.frames.len();
                Ok(())
            }
            Err(result) => {
                self.current = 0;
                Err(result)
            }
        }
    }
}
