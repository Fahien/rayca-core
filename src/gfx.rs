// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::rc::Rc;

use crate::*;

pub struct Vkr {
    pub debug: Debug,
    pub ctx: Ctx,
}

impl Vkr {
    pub fn new(win: &Win) -> Self {
        let ctx = Ctx::builder().win(win).build();
        let debug = Debug::new(&ctx);

        Self { ctx, debug }
    }
}

pub struct Dev {
    pub surface_format: vk::SurfaceFormatKHR,
    pub graphics_command_pool: vk::CommandPool,
    pub graphics_queue: Queue,
    /// Needs to be public if we want to create buffers outside this module.
    /// The allocator is shared between the various buffers to release resources on drop.
    pub allocator: Rc<vk_mem::Allocator>,
    pub device: Device,
}

impl Dev {
    pub fn new(ctx: &Ctx, surface: Option<&Surface>) -> Self {
        let device = Device::new(&ctx.instance, surface);

        let graphics_queue = Queue::new(&device);

        // Command pool
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.graphics_queue_index);
        let graphics_command_pool = {
            unsafe {
                device
                    .create_command_pool(&create_info, None)
                    .expect("Failed to create Vulkan command pool")
            }
        };

        // Surface format
        let mut surface_format = vk::SurfaceFormatKHR::default()
            .format(vk::Format::R8G8B8A8_SRGB)
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR);

        if let Some(surface) = surface {
            surface_format = {
                let surface_formats = unsafe {
                    surface
                        .ext
                        .get_physical_device_surface_formats(device.physical, surface.surface)
                }
                .expect("Failed to get Vulkan physical device surface formats");

                surface_formats[1]
            }
        }
        println!("Surface format: {:?}", surface_format.format);

        let allocator = {
            let create_info =
                vk_mem::AllocatorCreateInfo::new(&ctx.instance, &device, device.physical);
            unsafe { vk_mem::Allocator::new(create_info) }
        }
        .expect("Failed to create Vulkan allocator");

        Self {
            surface_format,
            graphics_command_pool,
            graphics_queue,
            allocator: Rc::new(allocator),
            device,
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for Vulkan device");
        }
    }
}

impl Drop for Dev {
    fn drop(&mut self) {
        self.wait();
        assert_eq!(Rc::strong_count(&self.allocator), 1);
        unsafe {
            self.device
                .destroy_command_pool(self.graphics_command_pool, None);
        }
    }
}
