// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use super::*;

/// This is the one that is going to be recreated
/// when the swapchain goes out of date
pub struct Framebuffer {
    // @todo Make a map of framebuffers indexed by render-pass as key
    pub framebuffer: vk::Framebuffer,

    pub depth_view: ImageView,
    pub depth_image: RenderImage,

    pub color_view: ImageView,
    pub color_image: RenderImage,

    pub normal_view: ImageView,
    pub normal_image: RenderImage,

    pub swapchain_view: vk::ImageView,
    pub extent: vk::Extent3D,
    device: Arc<ash::Device>,
}

impl Framebuffer {
    pub fn new(dev: &Dev, image: &RenderImage, pass: &Pass) -> Self {
        // Image view into a swapchain images (device, image, format)
        let swapchain_view = {
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

        // Color image with the same settings as the swapchain image
        let mut color_image = RenderImage::attachment(
            &dev.allocator,
            image.extent.width,
            image.extent.height,
            image.format,
        );
        color_image.transition(
            &dev.graphics_queue,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        let color_view = ImageView::new(&color_image);

        // Depth image
        let depth_format = vk::Format::D32_SFLOAT;
        let mut depth_image = RenderImage::attachment(
            &dev.allocator,
            image.extent.width,
            image.extent.height,
            depth_format,
        );
        depth_image.transition(
            &dev.graphics_queue,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );

        let depth_view = ImageView::new(&depth_image);

        // Normal image
        let normal_format = vk::Format::A2R10G10B10_UNORM_PACK32;
        let mut normal_image = RenderImage::attachment(
            &dev.allocator,
            image.extent.width,
            image.extent.height,
            normal_format,
        );
        normal_image.transition(
            &dev.graphics_queue,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        let normal_view = ImageView::new(&normal_image);

        // Framebuffers (image_views, renderpass)
        let framebuffer = {
            let attachments = [
                swapchain_view,
                depth_view.view,
                color_view.view,
                normal_view.view,
            ];

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
            color_view,
            color_image,
            normal_view,
            normal_image,
            swapchain_view,
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
            self.device.destroy_image_view(self.swapchain_view, None);
        }
    }
}

pub struct BufferCache<K>
where
    K: std::hash::Hash + Eq,
{
    map: HashMap<K, Buffer>,
    allocator: Arc<Allocator>,
}

impl<K> BufferCache<K>
where
    K: std::hash::Hash + Eq,
{
    fn new(allocator: &Arc<Allocator>) -> Self {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelMatrixKey {
    pub model: Handle<RenderModel>,
    pub node: Handle<Node>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewMatrixKey {
    pub model: Handle<RenderModel>,
    pub node: Handle<Node>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjMatrixKey {
    pub model: Handle<RenderModel>,
    pub camera: Handle<Camera>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalMatrixKey {
    pub model: Handle<RenderModel>,
    pub node: Handle<Node>,
    pub view: Handle<Node>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialKey {
    pub model: Handle<RenderModel>,
    pub material: Handle<Material>,
}

/// The frame cache contains resources that do not need to be recreated
/// when the swapchain goes out of date
pub struct FrameCache {
    /// Uniform buffers for model matrices associated to nodes
    pub model_buffers: BufferCache<ModelMatrixKey>,

    /// Uniform buffers for camera matrices associated to nodes with cameras
    pub view_buffers: BufferCache<ViewMatrixKey>,

    // Uniform buffers for proj matrices associated to cameras
    pub proj_buffers: BufferCache<ProjMatrixKey>,

    pub material_buffers: BufferCache<MaterialKey>,

    // Buffers for normal matrices associated to mesh nodes and camera nodes
    pub normal_buffers: BufferCache<NormalMatrixKey>,

    pub descriptors: Descriptors,
    pub command_buffer: CommandBuffer,
    pub fence: Fence,

    /// The image ready semaphore is used by the acquire next image function and it will be signaled
    /// then the image is ready to be rendered onto. Indeed it is also used by the submit draw
    /// function which will wait for the image to be ready before submitting draw commands
    pub image_ready: Semaphore,

    /// Image drawn sempahore is used when submitting draw commands to a back-buffer
    /// and it will be signaled when rendering is finished. Indeed the present function
    /// is waiting on this sempahore before presenting the back-buffer to screen.
    pub image_drawn: Semaphore,

    pub device: Arc<ash::Device>,
}

impl FrameCache {
    pub fn new(dev: &Dev) -> Self {
        // Graphics command buffer (device, command pool)
        let command_buffer = CommandBuffer::new(&dev.graphics_queue.command_pool);

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

#[derive(Default, Clone, Copy)]
pub struct CameraDrawInfo {
    pub camera: Handle<Camera>,
    pub node: Handle<Node>,
    pub model: Handle<RenderModel>,
}

impl CameraDrawInfo {
    pub fn new(camera: Handle<Camera>, node: Handle<Node>, model: Handle<RenderModel>) -> Self {
        Self {
            camera,
            node,
            model,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct DrawInfo {
    pub primitive: Handle<Primitive>,
    pub node: Handle<Node>,
    pub model: Handle<RenderModel>,
}

impl DrawInfo {
    pub fn new(
        primitive: Handle<Primitive>,
        node: Handle<Node>,
        model: Handle<RenderModel>,
    ) -> Self {
        Self {
            primitive,
            node,
            model,
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

    /// Swapchain current transform
    pub current_transform: vk::SurfaceTransformFlagsKHR,

    /// Map of shaders and their associated draw info
    pub shaders_drawinfos: HashMap<u32, Vec<DrawInfo>>,

    /// A frame should be able to allocate a uniform buffer on draw
    pub dev: Arc<Dev>,
}

impl Frame {
    pub fn new(
        id: usize,
        in_flight_count: usize,
        dev: &Arc<Dev>,
        image: &RenderImage,
        pass: &Pass,
        current_transform: vk::SurfaceTransformFlagsKHR,
    ) -> Self {
        let buffer = Framebuffer::new(dev, image, pass);
        let cache = FrameCache::new(dev);

        Frame {
            id,
            in_flight_count,
            buffer,
            cache,
            current_transform,
            shaders_drawinfos: HashMap::new(),
            dev: dev.clone(),
        }
    }

    pub fn get_size(&self) -> Size2 {
        Size2::new(self.buffer.extent.width, self.buffer.extent.height)
    }

    fn update_node(
        &mut self,
        node_handle: Handle<Node>,
        trs: &Trs,
        hmodel: Handle<RenderModel>,
        scene: &RenderScene,
    ) {
        let model = scene.get_model(hmodel).unwrap();
        let node = model.get_node(node_handle).unwrap();
        let world_trs = trs * &node.trs;

        if node.mesh.is_valid() || node.camera.is_valid() {
            let model_matrix_key = ModelMatrixKey {
                model: hmodel,
                node: node_handle,
            };
            let uniform_buffer = self
                .cache
                .model_buffers
                .get_or_create::<Mat4>(model_matrix_key);
            uniform_buffer.upload(&world_trs.to_mat4());

            if let Some(camera) = model.get_camera(node.camera) {
                let view_matrix_key = ViewMatrixKey {
                    model: hmodel,
                    node: node_handle,
                };
                let view_buffer = self
                    .cache
                    .view_buffers
                    .get_or_create::<Mat4>(view_matrix_key);
                view_buffer.upload(&world_trs.get_inversed().to_mat4());

                let proj_matrix_key = ProjMatrixKey {
                    model: hmodel,
                    camera: node.camera,
                };
                let proj_buffer = self
                    .cache
                    .proj_buffers
                    .get_or_create::<Mat4>(proj_matrix_key);
                proj_buffer.upload(&camera.projection);
            }

            // Collect draw infos for this node
            if let Some(mesh) = model.get_mesh(node.mesh) {
                for primitive_handle in mesh.primitives.iter().copied() {
                    let primitive = model.get_primitive(primitive_handle).unwrap();
                    let material = model
                        .get_material(primitive.material)
                        .unwrap_or(&self.dev.fallback.white_material);

                    self.shaders_drawinfos
                        .entry(material.shader)
                        .or_default()
                        .push(DrawInfo::new(primitive_handle, node_handle, hmodel));
                }
            }
        }

        for child in node.children.iter().cloned() {
            self.update_node(child, &world_trs, hmodel, scene);
        }
    }

    fn update_nodes(&mut self, hmodel: Handle<RenderModel>, scene: &RenderScene) {
        let model = scene.get_model(hmodel).unwrap();
        let trs = model.get_root().trs.clone();
        for node in model.get_root().children.iter().cloned() {
            self.update_node(node, &trs, hmodel, scene);
        }
    }

    fn update_materials(&mut self, hmodel: Handle<RenderModel>, scene: &RenderScene) {
        let model = scene.get_model(hmodel).unwrap();
        for handle_index in 0..model.get_gltf().materials.len() {
            let material_handle = handle_index.into();
            let material = model.get_material(material_handle).unwrap();
            let material_key = MaterialKey {
                model: hmodel,
                material: material_handle,
            };
            let color_buffer = self
                .cache
                .material_buffers
                .get_or_create::<Color>(material_key);
            color_buffer.upload(&material.color);
        }
    }

    fn update(&mut self, scene: &RenderScene) {
        self.shaders_drawinfos.clear();
        for hmodel in scene.get_models().get_handles() {
            self.update_nodes(hmodel, scene);
            self.update_materials(hmodel, scene);
        }
    }

    /// Updates internal buffers and begins the command buffer
    pub fn begin(&mut self, scene: &RenderScene) {
        self.update(scene);

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

    /// - `invert_viewport` according to https://www.saschawillems.de/blog/2019/03/29/flipping-the-vulkan-viewport/
    pub fn set_viewport_and_scissor(&self, scale: f32, invert_viewport: bool) {
        let size = self.get_size();

        let y = if invert_viewport {
            size.height as f32 * scale
        } else {
            0.0
        };
        let height = if invert_viewport {
            -(size.height as f32) * scale
        } else {
            size.height as f32 * scale
        };

        let viewport = vk::Viewport::default()
            .y(y)
            .width(size.width as f32 * scale)
            .height(height)
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

    pub fn draw(&mut self, scene: &RenderScene, pipelines: &[Box<dyn RenderPipeline>]) {
        // Focus on one camera for the moment
        let camera_infos = vec![scene.get_default_camera_draw_info()];

        for (shader, draw_info) in self.shaders_drawinfos.clone() {
            let pipeline = &pipelines[shader as usize];
            pipeline.render(self, scene, &camera_infos, draw_info);
        }
    }

    pub fn end(&mut self, scene: &RenderScene, pipeline: &dyn RenderPipeline) {
        self.cache.command_buffer.next_subpass();
        pipeline.render(self, scene, &[], vec![]);
    }

    fn end_render_pass_and_command_buffer(&self) {
        self.cache.command_buffer.end_render_pass();
        self.cache.command_buffer.end();
    }

    pub fn present(
        &mut self,
        dev: &Dev,
        swapchain: &Swapchain,
        image_index: u32,
    ) -> Result<(), vk::Result> {
        self.end_render_pass_and_command_buffer();

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
            self.dev
                .device
                .device
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
    device: Arc<ash::Device>,
}

impl SwapchainFrames {
    pub fn new(ctx: &Ctx, surface: &Surface, dev: &Arc<Dev>, size: Size2, pass: &Pass) -> Self {
        let swapchain = Swapchain::new(ctx, surface, dev, size, None);

        let mut frames = Vec::new();
        let in_flight_count = swapchain.images.len();
        for (id, image) in swapchain.images.iter().enumerate() {
            let frame = Frame::new(
                id,
                in_flight_count,
                dev,
                image,
                pass,
                swapchain.current_transform,
            );
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
