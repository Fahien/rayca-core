// Copyright © 2021-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::{khr, vk};

use crate::*;

pub struct Swapchain {
    pub images: Vec<RenderImage>,
    pub swapchain: vk::SwapchainKHR,
    pub ext: khr::swapchain::Device,
}

impl Swapchain {
    pub fn new(ctx: &Ctx, surface: &Surface, dev: &Dev, width: u32, height: u32) -> Self {
        // Swapchain (instance, logical device, surface formats)
        let ext = khr::swapchain::Device::new(&ctx.instance, &dev.device);

        // This needs to be queried to prevent validation layers complaining
        let surface_capabilities = unsafe {
            surface
                .ext
                .get_physical_device_surface_capabilities(dev.device.physical, surface.surface)
        }
        .expect("Failed to get Vulkan physical device surface capabilities");
        println!(
            "Surface transform: {:?}",
            surface_capabilities.current_transform
        );

        let mut extent = surface_capabilities.min_image_extent;
        extent.width = extent.width.max(width);
        extent.height = extent.height.max(height);

        let swapchain = {
            let create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface.surface)
                .min_image_count(2)
                .image_format(dev.surface_format.format)
                .image_color_space(dev.surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true);
            unsafe { ext.create_swapchain(&create_info, None) }
                .expect("Failed to create Vulkan swapchain")
        };

        let swapchain_images = unsafe { ext.get_swapchain_images(swapchain) }
            .expect("Failed to get Vulkan swapchain images");

        let mut images = Vec::new();
        for image in swapchain_images.into_iter() {
            images.push(RenderImage::unmanaged(
                image,
                width,
                height,
                dev.surface_format.format,
                dev.surface_format.color_space,
            ));
        }

        Self {
            images,
            swapchain,
            ext,
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.ext.destroy_swapchain(self.swapchain, None);
        }
    }
}
