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
    pub depth_view: ImageView,
    pub depth_image: RenderImage,
    pub image_view: vk::ImageView,
    pub extent: vk::Extent3D,
    device: Rc<ash::Device>,
}

impl Framebuffer {
    pub fn new(dev: &Dev, image: &RenderImage, pass: &Pass) -> Self {
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

        let depth_format = vk::Format::D32_SFLOAT;
        let mut depth_image = RenderImage::attachment(
            &dev.allocator,
            image.extent.width,
            image.extent.height,
            depth_format,
        );
        depth_image.transition(dev, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let depth_view = ImageView::new(&dev.device.device, &depth_image);

        // Framebuffers (image_view, renderpass)
        let framebuffer = {
            let attachments = [image_view, depth_view.view];

            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(pass.render)
                .attachments(&attachments)
                .width(image.extent.width)
                .height(image.extent.height)
                .layers(1);

            unsafe { dev.device.create_framebuffer(&create_info, None) }
                .expect("Failed to create Vulkan framebuffer")
        };

        Self {
            framebuffer,
            depth_view,
            depth_image,
            image_view,
            extent: image.extent,
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

pub struct BufferCache<K>
where
    K: std::hash::Hash + Eq,
{
    map: HashMap<K, Buffer>,
    allocator: Rc<vk_mem::Allocator>,
}

impl<K> BufferCache<K>
where
    K: std::hash::Hash + Eq,
{
    fn new(allocator: &Rc<vk_mem::Allocator>) -> Self {
        Self {
            map: Default::default(),
            allocator: allocator.clone(),
        }
    }

    pub fn get_or_create<T>(&mut self, key: K) -> &mut Buffer {
        match self.map.entry(key) {
            Entry::Vacant(e) => {
                let uniform_buffer =
                    Buffer::new::<T>(&self.allocator, vk::BufferUsageFlags::UNIFORM_BUFFER);
                e.insert(uniform_buffer)
            }
            Entry::Occupied(e) => e.into_mut(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&Buffer> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut Buffer> {
        self.map.get_mut(key)
    }
}

/// Container of fallback resources for a frame such as
/// A white 1x1 pixel texture (image, view, and sampler)
pub struct Fallback {
    _white_image: RenderImage,
    _white_view: ImageView,
    _white_sampler: RenderSampler,
    pub white_texture: RenderTexture,
    pub white_buffer: Buffer,
    pub white_material: Material,
}

impl Fallback {
    fn new(dev: &Dev) -> Self {
        let white = [255, 255, 255, 255];
        let white_image = RenderImage::from_data(&dev, &white, 1, 1, vk::Format::R8G8B8A8_SRGB);
        let white_view = ImageView::new(&dev.device.device, &white_image);
        let white_sampler = RenderSampler::new(&dev.device.device);
        let white_texture = RenderTexture::new(&white_view, &white_sampler);
        let mut white_buffer =
            Buffer::new::<Color>(&dev.allocator, vk::BufferUsageFlags::UNIFORM_BUFFER);
        white_buffer.upload(&Color::WHITE);
        let white_material = Material::default();

        Self {
            _white_image: white_image,
            _white_view: white_view,
            _white_sampler: white_sampler,
            white_texture,
            white_buffer,
            white_material,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalMatrixKey {
    pub node: Handle<Node>,
    pub view: Handle<Node>,
}

/// The frame cache contains resources that do not need to be recreated
/// when the swapchain goes out of date
pub struct FrameCache {
    /// Uniform buffers for model matrices associated to nodes
    pub model_buffers: BufferCache<Handle<Node>>,

    /// Uniform buffers for camera matrices associated to nodes with cameras
    pub view_buffers: BufferCache<Handle<Node>>,

    // Uniform buffers for proj matrices associated to cameras
    pub proj_buffers: BufferCache<Handle<Camera>>,

    pub material_buffers: BufferCache<Handle<Material>>,

    // Buffers for normal matrices associated to mesh nodes and camera nodes
    pub normal_buffers: BufferCache<NormalMatrixKey>,

    pub descriptors: Descriptors,
    pub command_buffer: CommandBuffer,
    pub fence: Fence,
    pub image_ready: Semaphore,
    pub image_drawn: Semaphore,

    pub fallback: Fallback,

    pub device: Rc<ash::Device>,
}

impl FrameCache {
    pub fn new(dev: &Dev) -> Self {
        // Graphics command buffer (device, command pool)
        let command_buffer = CommandBuffer::new(&dev.graphics_command_pool);

        Self {
            model_buffers: BufferCache::new(&dev.allocator),
            view_buffers: BufferCache::new(&dev.allocator),
            proj_buffers: BufferCache::new(&dev.allocator),
            material_buffers: BufferCache::new(&dev.allocator),
            normal_buffers: BufferCache::new(&dev.allocator),
            descriptors: Descriptors::new(&dev.device),
            command_buffer,
            fence: Fence::signaled(&dev.device.device),
            image_ready: Semaphore::new(&dev.device.device),
            image_drawn: Semaphore::new(&dev.device.device),
            fallback: Fallback::new(dev),
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
    /// The number of this frame
    pub id: usize,

    /// The number of in-flight frames
    pub in_flight_count: usize,

    pub buffer: Framebuffer,
    pub cache: FrameCache,

    /// A frame should be able to allocate a uniform buffer on draw
    pub device: Rc<ash::Device>,
}

impl Frame {
    pub fn new(
        id: usize,
        in_flight_count: usize,
        dev: &Dev,
        image: &RenderImage,
        pass: &Pass,
    ) -> Self {
        let buffer = Framebuffer::new(dev, image, pass);
        let cache = FrameCache::new(dev);

        Frame {
            id,
            in_flight_count,
            buffer,
            cache,
            device: dev.device.device.clone(),
        }
    }

    pub fn get_size(&self) -> Size2 {
        Size2::new(self.buffer.extent.width, self.buffer.extent.height)
    }

    fn update_node(&mut self, node_handle: Handle<Node>, model: &RenderModel) {
        let node = model.get_node(node_handle).unwrap();
        if node.mesh.is_valid() || node.camera.is_valid() {
            let uniform_buffer = self.cache.model_buffers.get_or_create::<Mat4>(node_handle);
            uniform_buffer.upload(&node.trs.to_mat4());

            if let Some(camera) = model.get_camera(node.camera) {
                let view_buffer = self.cache.view_buffers.get_or_create::<Mat4>(node_handle);
                view_buffer.upload(&node.trs.to_view_mat4());

                let proj_buffer = self.cache.proj_buffers.get_or_create::<Mat4>(node.camera);
                proj_buffer.upload(&camera.projection);
            }

            if let Some(mesh) = model.get_mesh(node.mesh) {
                for primitive_handle in mesh.primitives.iter().copied() {
                    let primitive = model.get_primitive(primitive_handle).unwrap();
                    if let Some(material) = model.get_material(primitive.material) {
                        let color_buffer = self
                            .cache
                            .material_buffers
                            .get_or_create::<Color>(primitive.material);
                        color_buffer.upload(&material.color);
                    }
                }
            }
        }
        for child in node.children.iter().cloned() {
            self.update_node(child, model);
        }
    }

    fn update(&mut self, model: &RenderModel) {
        for node in model.get_scene().children.iter().cloned() {
            self.update_node(node, model);
        }
    }

    /// Updates internal buffers and begins the command buffer
    pub fn begin(&mut self, model: &RenderModel) {
        self.update(model);

        self.cache
            .command_buffer
            .begin(vk::CommandBufferUsageFlags::default());
    }

    pub fn begin_render(&self, pass: &Pass) {
        let size = self.get_size();
        self.cache
            .command_buffer
            .begin_render_pass(pass, &self.buffer, size);
    }

    pub fn set_viewport_and_scissor(&self, scale: f32) {
        let size = self.get_size();

        let viewport = vk::Viewport::default()
            .width(size.width as f32 * scale)
            .height(size.height as f32 * scale)
            .min_depth(1.0)
            .max_depth(0.0);
        self.cache.command_buffer.set_viewport(viewport);

        let scissor = vk::Rect2D::default().extent(
            vk::Extent2D::default()
                .width(size.width)
                .height(size.height),
        );
        self.cache.command_buffer.set_scissor(scissor);
    }

    pub fn draw_node(
        &mut self,
        cameras: &[Handle<Node>],
        node_handle: Handle<Node>,
        model: &RenderModel,
        pipelines: &[Box<dyn RenderPipeline>],
    ) {
        let node = model.get_node(node_handle).unwrap();
        if let Some(mesh) = model.get_mesh(node.mesh) {
            for primitive_handle in mesh.primitives.iter().copied() {
                let primitive = model.get_primitive(primitive_handle).unwrap();
                let material = match model.get_material(primitive.material) {
                    Some(material) => material,
                    None => &self.cache.fallback.white_material,
                };
                let pipeline = &pipelines[material.shader as usize];
                // Rendering a node multiple times for each primitive?
                pipeline.render(self, model, cameras, &[node_handle]);
            }
        }
        for child in node.children.iter().cloned() {
            self.draw_node(cameras, child, model, pipelines);
        }
    }

    pub fn draw(&mut self, model: &RenderModel, pipelines: &[Box<dyn RenderPipeline>]) {
        // Collect camera handles
        let mut cameras = Vec::default();
        for node_handle in model.get_scene().children.iter().cloned() {
            let node = model.get_node(node_handle).unwrap();
            if node.camera.is_valid() {
                cameras.push(node_handle);
            }
        }

        for node_handle in model.get_scene().children.iter().cloned() {
            self.draw_node(&cameras, node_handle, model, pipelines);
        }
    }

    pub fn end(&self) {
        self.cache.command_buffer.end_render_pass();
        self.cache.command_buffer.end();
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
    fn next_frame(&mut self) -> Result<Frame, vk::Result>;
    fn present(&mut self, dev: &Dev, frame: Frame) -> Result<(), vk::Result>;
}

/// Offscreen frames work on user allocated images
struct _OffscreenFrames {
    _frames: Vec<Frame>,
    _images: Vec<vk::Image>,
}

impl Frames for _OffscreenFrames {
    fn next_frame(&mut self) -> Result<Frame, vk::Result> {
        // Unimplemented
        Err(vk::Result::ERROR_UNKNOWN)
    }

    fn present(&mut self, _dev: &Dev, _frame: Frame) -> Result<(), vk::Result> {
        // Unimplemented
        Err(vk::Result::ERROR_UNKNOWN)
    }
}

/// Swapchain frames work on swapchain images
pub struct SwapchainFrames {
    pub frames: Vec<Option<Frame>>,
    pub swapchain: Swapchain,
    device: Rc<ash::Device>,
}

impl SwapchainFrames {
    pub fn new(ctx: &Ctx, surface: &Surface, dev: &Dev, size: Size2, pass: &Pass) -> Self {
        let swapchain = Swapchain::new(ctx, surface, dev, size, None);

        let mut frames = Vec::new();
        let in_flight_count = swapchain.images.len();
        for (id, image) in swapchain.images.iter().enumerate() {
            let frame = Frame::new(id, in_flight_count, dev, image, pass);
            frames.push(Some(frame));
        }

        Self {
            frames,
            swapchain,
            device: dev.device.device.clone(),
        }
    }
}

impl Frames for SwapchainFrames {
    fn next_frame(&mut self) -> Result<Frame, vk::Result> {
        // Create a new semaphore for the next image
        let image_ready = Semaphore::new(&self.device);

        let acquire_res = unsafe {
            self.swapchain.ext.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                image_ready.semaphore,
                vk::Fence::null(),
            )
        };

        match acquire_res {
            Ok((image_index, _)) => {
                // Take frame at image index
                let mut frame = self.frames[image_index as usize].take().unwrap();
                assert_eq!(frame.id, image_index as usize);
                // Wait for this frame's command buffer to be ready
                frame.cache.wait();
                // Save created semaphore in this frame
                frame.cache.image_ready = image_ready;
                Ok(frame)
            }
            // Suboptimal
            //Ok((_, true)) => Err(vk::Result::ERROR_OUT_OF_DATE_KHR),
            Err(result) => Err(result),
        }
    }

    fn present(&mut self, dev: &Dev, frame: Frame) -> Result<(), vk::Result> {
        let image_index = frame.id;
        self.frames[image_index].replace(frame);

        let frame = self.frames[image_index].as_mut().unwrap();
        match frame.present(dev, &self.swapchain, image_index as u32) {
            Ok(()) => Ok(()),
            Err(result) => Err(result),
        }
    }
}
