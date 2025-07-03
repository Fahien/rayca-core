// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::{
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use super::*;

/// This is the one that is going to be recreated
/// when the swapchain goes out of date
pub struct Framebuffer {
    // @todo Make a map of framebuffers indexed by render-pass as key
    pub framebuffer: vk::Framebuffer,
    pub image_view: vk::ImageView,
    device: Rc<ash::Device>,
}

impl Framebuffer {
    pub fn new(device: &Rc<ash::Device>, image: &RenderImage, pass: &Pass) -> Self {
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
            unsafe { device.create_image_view(&create_info, None) }
                .expect("Failed to create Vulkan image view")
        };

        // Framebuffers (image_view, renderpass)
        let framebuffer = {
            let attachments = [image_view];

            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(pass.render)
                .attachments(&attachments)
                .width(image.extent.width)
                .height(image.extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&create_info, None) }
                .expect("Failed to create Vulkan framebuffer")
        };

        Self {
            framebuffer,
            image_view,
            device: device.clone(),
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
    /// Uniform buffers for model matrix are associated to node indices
    pub uniforms: HashMap<Handle<Node>, Buffer>,
    pub descriptors: Descriptors,
    pub command_buffer: vk::CommandBuffer,
    pub fence: Fence,
    pub image_ready: Semaphore,
    pub image_drawn: Semaphore,
    pub device: Rc<ash::Device>,
}

impl FrameCache {
    pub fn new(dev: &Dev) -> Self {
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

        Self {
            uniforms: HashMap::new(),
            descriptors: Descriptors::new(&dev.device),
            command_buffer,
            fence: Fence::signaled(&dev.device.device),
            image_ready: Semaphore::new(&dev.device.device),
            image_drawn: Semaphore::new(&dev.device.device),
            device: dev.device.device.clone(),
        }
    }

    pub fn wait(&mut self) {
        if self.fence.can_wait {
            self.fence.wait();
            self.fence.reset();
        }
    }
}

pub struct Frame {
    pub buffer: Framebuffer,
    pub cache: FrameCache,

    /// A frame should be able to allocate a uniform buffer on draw
    allocator: Rc<vk_mem::Allocator>,
    pub device: Rc<ash::Device>,
}

impl Frame {
    pub fn new(dev: &Dev, image: &RenderImage, pass: &Pass) -> Self {
        let buffer = Framebuffer::new(&dev.device.device, image, pass);
        let cache = FrameCache::new(dev);

        Frame {
            buffer,
            cache,
            allocator: dev.allocator.clone(),
            device: dev.device.device.clone(),
        }
    }

    pub fn update(&mut self, model: &RenderModel) {
        for node_handle in model.gltf.scene.iter().cloned() {
            let node = model.gltf.nodes.get(node_handle).unwrap();
            if !node.mesh.is_valid() {
                // Skip nodes without a mesh
                continue;
            }

            let uniform_buffer = match self.cache.uniforms.entry(node_handle) {
                Entry::Vacant(e) => {
                    let uniform_buffer =
                        Buffer::new::<Mat4>(&self.allocator, vk::BufferUsageFlags::UNIFORM_BUFFER);
                    e.insert(uniform_buffer)
                }
                Entry::Occupied(e) => e.into_mut(),
            };
            uniform_buffer.upload(&node.trs.to_mat4());
        }
    }

    pub fn begin(&self, pass: &Pass, area: Size2) {
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
            .render_area(vk::Rect2D::default().extent(vk::Extent2D {
                width: area.width,
                height: area.height,
            }))
            .clear_values(&clear_values);
        // Record it in the main command buffer
        let contents = vk::SubpassContents::INLINE;
        unsafe {
            self.device
                .cmd_begin_render_pass(self.cache.command_buffer, &create_info, contents)
        };

        let viewports = [vk::Viewport::default()
            .width(area.width as f32)
            .height(area.height as f32)];
        unsafe {
            self.device
                .cmd_set_viewport(self.cache.command_buffer, 0, &viewports)
        };

        let scissors = [vk::Rect2D::default().extent(
            vk::Extent2D::default()
                .width(area.width)
                .height(area.height),
        )];
        unsafe {
            self.device
                .cmd_set_scissor(self.cache.command_buffer, 0, &scissors)
        }
    }

    pub fn draw(&mut self, model: &RenderModel, pipelines: &[Box<dyn RenderPipeline>]) {
        for node_handle in model.gltf.scene.iter().cloned() {
            let node = model.gltf.nodes.get(node_handle).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let primitive = model.gltf.primitives.get(mesh.primitive).unwrap();
            let material = model.gltf.materials.get(primitive.material).unwrap();
            let pipeline = &pipelines[material.shader as usize];
            pipeline.render(self, model, &[node_handle]);
        }
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
        dev.graphics_queue.submit_draw(
            &self.cache.command_buffer,
            &self.cache.image_ready,
            &self.cache.image_drawn,
            Some(&mut self.cache.fence),
        );

        dev.graphics_queue
            .present(image_index, swapchain, &self.cache.image_drawn)
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
                frame.cache.image_ready.semaphore,
                vk::Fence::null(),
            )
        };

        match acquire_res {
            Ok((image_index, false)) => {
                self.image_index = image_index;
                Ok(frame)
            }
            // Suboptimal
            Ok((_, true)) => {
                self.current = 0;
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
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
