// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use super::*;

pub struct Png<R: std::io::Read> {
    pub reader: png::Reader<R>,
}

impl<R: std::io::Read> Png<R> {
    /// Creates a png decoder without loading data yet
    pub fn new(read: R) -> Self {
        let decoder = png::Decoder::new(read);
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
    allocator: Option<Rc<vk_mem::Allocator>>,
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
        image: vk::Image,
        width: u32,
        height: u32,
        format: vk::Format,
        color_space: vk::ColorSpaceKHR,
    ) -> Self {
        let extent = vk::Extent3D::default().width(width).height(height).depth(1);

        Self {
            managed: true,
            image,
            layout: vk::ImageLayout::UNDEFINED,
            extent,
            format,
            color_space,
            allocation: None,
            allocator: None,
        }
    }

    /// Creates a new empty image
    pub fn new(
        allocator: &Rc<vk_mem::Allocator>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        let allocator = allocator.clone();

        let extent = vk::Extent3D::default().width(width).height(height).depth(1);

        let usage = if Self::is_depth_format(format) {
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
        } else {
            // Default usage is as a texture
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED
        };

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

        Self {
            managed: true,
            image,
            layout: vk::ImageLayout::UNDEFINED,
            extent,
            format,
            color_space: vk::ColorSpaceKHR::default(),
            allocation: Some(allocation),
            allocator: Some(allocator),
        }
    }

    pub fn load<R: std::io::Read>(dev: &Dev, png: &mut Png<R>) -> Self {
        let staging = Buffer::load(&dev.allocator, png);

        let png_info = png.reader.info();

        let mut image = Self::new(
            &dev.allocator,
            png_info.width,
            png_info.height,
            vk::Format::R8G8B8A8_SRGB,
        );
        image.copy_from(&staging, dev);
        image
    }

    pub fn transition(&mut self, dev: &Dev, new_layout: vk::ImageLayout) {
        // @todo Use TRANSFER pool and transfer queue?
        let command_buffer = CommandBuffer::new(&dev.graphics_command_pool);
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

        let mut fence = Fence::unsignaled(&dev.device.device);

        let commands = [command_buffer.command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        dev.graphics_queue.submit(&submits, Some(&mut fence));

        fence.wait();
    }

    pub fn copy_from(&mut self, staging: &Buffer, dev: &Dev) {
        // @todo Use TRANSFER pool and transfer queue
        let command_buffer = CommandBuffer::new(&dev.graphics_command_pool);
        command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

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

        command_buffer.end();

        let mut fence = Fence::unsignaled(&dev.device.device);

        let commands = [command_buffer.command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        dev.graphics_queue.submit(&submits, Some(&mut fence));

        fence.wait();
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
    device: Rc<ash::Device>,
}

impl ImageView {
    pub fn new(device: &Rc<ash::Device>, image: &RenderImage) -> Self {
        let device = device.clone();

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

        let view = unsafe { device.create_image_view(&create_info, None) }
            .expect("Failed to create Vulkan image view");

        Self { view, device }
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
