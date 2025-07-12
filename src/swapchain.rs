// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::{khr, vk};

use crate::*;

pub struct Swapchain {
    pub images: Vec<RenderImage>,
    pub swapchain: vk::SwapchainKHR,
    pub ext: khr::swapchain::Device,
    pub current_transform: vk::SurfaceTransformFlagsKHR,
}

impl Swapchain {
    pub fn new(
        ctx: &Ctx,
        surface: &Surface,
        dev: &Dev,
        mut size: Size2,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Self {
        // Swapchain (instance, logical device, surface formats)
        let ext = khr::swapchain::Device::new(&ctx.instance, &dev.device);

        // This needs to be queried to prevent validation layers complaining
        let surface_capabilities = unsafe {
            surface
                .ext
                .get_physical_device_surface_capabilities(dev.device.physical, surface.surface)
        }
        .expect("Failed to get Vulkan physical device surface capabilities");

        let current_transform = surface_capabilities.current_transform;
        println!("Surface transform: {:?}", current_transform);

        if current_transform.contains(vk::SurfaceTransformFlagsKHR::ROTATE_90)
            || current_transform.contains(vk::SurfaceTransformFlagsKHR::ROTATE_270)
        {
            // Pre-rotation: always use native orientation i.e. if rotated, use width and height of identity transform
            size = Size2::new(size.height, size.width);
        }

        let mut extent = surface_capabilities.min_image_extent;
        extent.width = extent.width.max(size.width);
        extent.height = extent.height.max(size.height);

        let swapchain = {
            let mut create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface.surface)
                .min_image_count(3)
                .image_format(dev.surface_format.format)
                .image_color_space(dev.surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true);
            if let Some(old_swapchain) = old_swapchain {
                create_info = create_info.old_swapchain(old_swapchain);
            }
            unsafe { ext.create_swapchain(&create_info, None) }
                .expect("Failed to create Vulkan swapchain")
        };

        let swapchain_images = unsafe { ext.get_swapchain_images(swapchain) }
            .expect("Failed to get Vulkan swapchain images");

        let mut images = Vec::new();
        for image in swapchain_images.into_iter() {
            images.push(RenderImage::unmanaged(
                image,
                size,
                dev.surface_format.format,
                dev.surface_format.color_space,
            ));
        }

        Self {
            images,
            swapchain,
            ext,
            current_transform,
        }
    }

    /// Prerotation to apply only to presentation pass.
    pub fn get_prerotation_trs(current_transform: vk::SurfaceTransformFlagsKHR) -> Trs {
        let angle_radians = -std::f32::consts::PI
            * match current_transform {
                vk::SurfaceTransformFlagsKHR::ROTATE_90 => 0.5,
                vk::SurfaceTransformFlagsKHR::ROTATE_270 => 1.5,
                vk::SurfaceTransformFlagsKHR::ROTATE_180 => 1.0,
                _ => 0.0,
            };

        Trs::builder()
            .rotation(Quat::axis_angle(Vec3::Z_AXIS, angle_radians))
            .build()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.ext.destroy_swapchain(self.swapchain, None);
        }
    }
}
