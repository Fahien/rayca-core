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

pub struct Image {
    /// Whether this image is manages and should be freed, or not (like swapchain images)
    managed: bool,
    pub image: vk::Image,
    layout: vk::ImageLayout,
    pub extent: vk::Extent3D,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    allocation: Option<vk_mem::Allocation>,
    allocator: Option<Rc<vk_mem::Allocator>>,
}

impl Image {
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

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .format(format)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
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

        let mut image = Image::new(
            &dev.allocator,
            png_info.width,
            png_info.height,
            vk::Format::R8G8B8A8_SRGB,
        );
        image.copy_from(&staging, dev);
        image
    }

    pub fn copy_from(&mut self, staging: &Buffer, dev: &Dev) {
        // @todo Use TRANSFER pool and transfer queue
        let command_buffer = unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(dev.graphics_command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let buffers = dev
                .device
                .allocate_command_buffers(&alloc_info)
                .expect("Failed to allocate Vulkan command buffer");
            buffers[0]
        };

        unsafe {
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            dev.device.begin_command_buffer(command_buffer, &begin_info)
        }
        .expect("Failed to begin Vulkan command buffer");

        // Undefined -> Transfer dst optimal
        unsafe {
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
            dev.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                &[],
                &[],
                &image_memory_barriers,
            );

            self.layout = new_layout;
        }

        // Copy
        unsafe {
            let dst_image_layout = self.layout;
            let regions = vec![
                vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(self.extent),
            ];
            dev.device.cmd_copy_buffer_to_image(
                command_buffer,
                staging.buffer,
                self.image,
                dst_image_layout,
                &regions,
            );
        }

        // Transfer dst optimal -> Shader read only optimal
        unsafe {
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
            dev.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                &[],
                &[],
                &image_memory_barriers,
            );

            self.layout = new_layout;
        }

        // End
        unsafe {
            dev.device
                .end_command_buffer(command_buffer)
                .expect("Failed to end Vulkan command buffer");
        }

        let mut fence = Fence::unsignaled(&dev.device.device);

        let commands = [command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        dev.graphics_queue.submit(&submits, Some(&mut fence));

        fence.wait();

        unsafe {
            dev.device
                .free_command_buffers(dev.graphics_command_pool, &commands);
        }
    }
}

impl Drop for Image {
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
    pub fn new(device: &Rc<ash::Device>, image: &Image) -> Self {
        let device = device.clone();

        let create_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
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
pub struct Texture {
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(view: vk::ImageView, sampler: vk::Sampler) -> Self {
        Self { view, sampler }
    }
}
