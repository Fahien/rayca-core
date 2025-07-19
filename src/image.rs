// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::*;

pub struct Png<R: std::io::Read> {
    pub reader: png::Reader<R>,
}

impl<R: std::io::Read> Png<R> {
    /// Creates a png decoder without loading data yet
    pub fn new(read: R) -> Self {
        let mut decoder = png::Decoder::new(read);
        decoder.set_transformations(png::Transformations::STRIP_16 | png::Transformations::ALPHA);
        let reader = decoder.read_info().unwrap();
        Self { reader }
    }
}

pub struct RenderImage {
    /// Whether this image is manages and should be freed, or not (like swapchain images)
    managed: bool,
    pub image: vk::Image,
    pub layout: vk::ImageLayout,
    pub extent: vk::Extent3D,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    allocation: Option<vk_mem::Allocation>,
    allocator: Option<Arc<Allocator>>,
    device: Arc<Device>,
}

impl RenderImage {
    pub fn is_depth_format(format: vk::Format) -> bool {
        format == vk::Format::D16_UNORM
            || format == vk::Format::D16_UNORM_S8_UINT
            || format == vk::Format::D24_UNORM_S8_UINT
            || format == vk::Format::D32_SFLOAT
            || format == vk::Format::D32_SFLOAT_S8_UINT
    }

    pub fn get_aspect_from_format(format: vk::Format) -> vk::ImageAspectFlags {
        if Self::is_depth_format(format) {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        }
    }

    pub fn unmanaged(
        device: &Arc<Device>,
        image: vk::Image,
        size: Size2,
        format: vk::Format,
        color_space: vk::ColorSpaceKHR,
    ) -> Self {
        // Minimum size is 1x1
        let extent = vk::Extent3D::default()
            .width(size.width.max(1))
            .height(size.height.max(1))
            .depth(1);

        Self {
            managed: true,
            image,
            layout: vk::ImageLayout::UNDEFINED,
            extent,
            format,
            color_space,
            allocation: None,
            allocator: None,
            device: device.clone(),
        }
    }

    /// Creates a new empty image
    pub fn new(
        allocator: &Arc<Allocator>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Self {
        let allocator = allocator.clone();

        // Minimum size is 1x1
        let extent = vk::Extent3D::default()
            .width(width.max(1))
            .height(height.max(1))
            .depth(1);

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .format(format)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        use vk_mem::Alloc;
        let (image, allocation) = unsafe { allocator.create_image(&image_info, &alloc_info) }
            .expect("Failed to create Vulkan image");

        let device = allocator.device.clone();

        Self {
            managed: true,
            image,
            layout: vk::ImageLayout::UNDEFINED,
            extent,
            format,
            color_space: vk::ColorSpaceKHR::default(),
            allocation: Some(allocation),
            allocator: Some(allocator),
            device,
        }
    }

    /// Create an image that can be used as an input or output attachment
    pub fn attachment(
        allocator: &Arc<Allocator>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        let usage = if Self::is_depth_format(format) {
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT
        } else {
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT
        };
        Self::new(allocator, width, height, format, usage)
    }

    /// Create an image that can be used to upload data from disk and sampled from a fragment shader
    pub fn sampled(
        allocator: &Arc<Allocator>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        Self::new(
            allocator,
            width,
            height,
            format,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        )
    }

    /// Creates a new image from raw data uploading it into a sampled image
    pub fn from_data(
        allocator: &Arc<Allocator>,
        graphics_queue: &GraphicsQueue,
        data: &[u8],
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        let mut image = Self::sampled(allocator, width, height, format);
        let usage = vk::BufferUsageFlags::TRANSFER_SRC;
        let staging = RenderBuffer::from_data(allocator, data, usage);
        image.simple_copy_from(&staging, graphics_queue);
        image
    }

    /// Loads a PNG image from file and uploads it into a sampled image
    pub fn load(allocator: &Arc<Allocator>, graphics_queue: &GraphicsQueue, asset: Asset) -> Self {
        let image_reader = ::image::ImageReader::new(std::io::Cursor::new(asset.data))
            .with_guessed_format()
            .expect("Failed to guess image format")
            .decode()
            .expect("Failed to decode image");
        let rgba8_image = image_reader.into_rgba8();
        let dim = rgba8_image.dimensions();
        let staging = RenderBuffer::load(allocator, rgba8_image);

        let format = vk::Format::R8G8B8A8_SRGB;
        let mut image = Self::sampled(allocator, dim.0, dim.1, format);
        image.simple_copy_from(&staging, graphics_queue);
        image
    }

    pub fn transition(&mut self, graphics_queue: &GraphicsQueue, new_layout: vk::ImageLayout) {
        // @todo Use TRANSFER pool and transfer queue?
        let command_buffer = CommandBuffer::new(&graphics_queue.command_pool);
        command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        // Old layout -> New layout
        let src_stage_mask = vk::PipelineStageFlags::TOP_OF_PIPE;
        let dst_stage_mask = vk::PipelineStageFlags::TRANSFER;
        let dependency_flags = vk::DependencyFlags::default();
        let image_memory_barriers = vec![
            vk::ImageMemoryBarrier::default()
                .old_layout(self.layout)
                .new_layout(new_layout)
                .image(self.image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(Self::get_aspect_from_format(self.format))
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
        ];
        command_buffer.pipeline_barriers(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            &image_memory_barriers,
        );

        self.layout = new_layout;

        command_buffer.end();

        let mut fence = Fence::unsignaled(&graphics_queue.command_pool.device);

        let commands = [command_buffer.command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        graphics_queue.submit(&submits, Some(&mut fence));

        fence.wait();
    }

    pub fn simple_copy_from(&mut self, staging: &RenderBuffer, graphics_queue: &GraphicsQueue) {
        // @todo Use TRANSFER pool and transfer queue
        let command_buffer = CommandBuffer::new(&graphics_queue.command_pool);
        command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.copy_from(staging, &command_buffer);

        command_buffer.end();

        let mut fence = Fence::unsignaled(&graphics_queue.command_pool.device);

        let commands = [command_buffer.command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        graphics_queue.submit(&submits, Some(&mut fence));

        fence.wait();
    }

    pub fn copy_from(&mut self, staging: &RenderBuffer, command_buffer: &CommandBuffer) {
        // Undefined -> Transfer dst optimal
        let new_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;

        let src_stage_mask = vk::PipelineStageFlags::TOP_OF_PIPE;
        let dst_stage_mask = vk::PipelineStageFlags::TRANSFER;
        let dependency_flags = vk::DependencyFlags::default();
        let image_memory_barriers = vec![
            vk::ImageMemoryBarrier::default()
                .old_layout(self.layout)
                .new_layout(new_layout)
                .image(self.image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
        ];
        command_buffer.pipeline_barriers(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            &image_memory_barriers,
        );

        self.layout = new_layout;

        // Copy
        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_extent(self.extent);
        command_buffer.copy_buffer_to_image(staging, self, &region);

        // Transfer dst optimal -> Shader read only optimal
        let new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

        let src_stage_mask = vk::PipelineStageFlags::TRANSFER;
        let dst_stage_mask = vk::PipelineStageFlags::FRAGMENT_SHADER;
        let dependency_flags = vk::DependencyFlags::default();
        let image_memory_barriers = vec![
            vk::ImageMemoryBarrier::default()
                .old_layout(self.layout)
                .new_layout(new_layout)
                .image(self.image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ),
        ];
        command_buffer.pipeline_barriers(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            &image_memory_barriers,
        );

        self.layout = new_layout;
    }
}

impl Drop for RenderImage {
    fn drop(&mut self) {
        if self.managed {
            if let Some(alloc) = self.allocator.as_ref() {
                unsafe {
                    alloc.destroy_image(self.image, self.allocation.as_mut().unwrap());
                }
            }
        }
    }
}

pub struct ImageView {
    pub view: vk::ImageView,
    device: Arc<Device>,
}

impl ImageView {
    pub fn new(image: &RenderImage) -> Self {
        let aspect = RenderImage::get_aspect_from_format(image.format);

        let create_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let view = unsafe { image.device.create_image_view(&create_info, None) }
            .expect("Failed to create Vulkan image view");

        Self {
            view,
            device: image.device.clone(),
        }
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
        }
    }
}

#[derive(Default)]
pub struct RenderTexture {
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl RenderTexture {
    pub fn new(view: &ImageView, sampler: &RenderSampler) -> Self {
        Self {
            view: view.view,
            sampler: sampler.sampler,
        }
    }
}
