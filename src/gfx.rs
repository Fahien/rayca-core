// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::rc::Rc;

use crate::*;

pub struct Vkr {
    pub frames: SwapchainFrames,
    pub pass: Pass,
    pub dev: Dev,
    pub surface: Surface,
    pub debug: Debug,
    pub ctx: Ctx,
    pub events: Option<Events>,
}

impl Vkr {
    pub fn new(win: &mut Win) -> Self {
        let mut events = Events::new(win);
        let ctx = Ctx::builder().win(win).build();
        let debug = Debug::new(&ctx);

        // Pump events to ensure the window is created and ready
        loop {
            events.update(win);
            if win.window.is_some() || win.exit {
                break;
            }
        }

        let surface = Surface::new(&win, &ctx);
        let dev = Dev::new(&ctx, Some(&surface));
        let pass = Pass::new(&dev);

        let frames = SwapchainFrames::new(&ctx, &surface, &dev, win.size, &pass);

        Self {
            events: Some(events),
            ctx,
            debug,
            surface,
            dev,
            pass,
            frames,
        }
    }

    fn recreate_swapchain(&mut self, size: Size2) {
        self.dev.wait();
        // Drop swapchain?
        // Current must be reset to avoid LAYOUT_UNDEFINED validation errors
        self.frames.swapchain = Swapchain::new(
            &self.ctx,
            &self.surface,
            &self.dev,
            size,
            Some(self.frames.swapchain.swapchain),
        );
        for i in 0..self.frames.swapchain.images.len() {
            let frame = &mut self.frames.frames[i].as_mut().unwrap();
            // Only this semaphore must be recreated to avoid validation errors
            // The image drawn one is still in use at the moment
            frame.cache.image_ready = Semaphore::new(&self.dev.device.device);
            frame.buffer =
                Framebuffer::new(&self.dev, &self.frames.swapchain.images[i], &self.pass);
        }
    }

    pub fn update(&mut self, win: &mut Win) {
        if let Some(events) = self.events.as_mut() {
            events.update(win);
        }
        if win.exit {
            return;
        }
        if win.is_resized() {
            println!("Window resized to: {}x{}", win.size.width, win.size.height);
            self.recreate_swapchain(win.size);
        }
    }

    pub fn next_frame(&mut self, win: &Win) -> Result<Option<Frame>, vk::Result> {
        match self.frames.next_frame() {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                println!("Swapchain out of date, recreating...");
                self.recreate_swapchain(win.size);
                Ok(None)
            }
            Err(result) => Err(result),
            Ok(frame) => Ok(Some(frame)),
        }
    }

    pub fn present(&mut self, win: &Win, frame: Frame) -> Result<(), vk::Result> {
        match self.frames.present(&self.dev, frame) {
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                println!("Swapchain out of date, recreating...");
                self.recreate_swapchain(win.size);
                Ok(())
            }
            Err(result) => Err(result),
            _ => Ok(()),
        }
    }
}

pub struct Dev {
    pub surface_format: vk::SurfaceFormatKHR,
    pub graphics_command_pool: CommandPool,
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

        let graphics_command_pool = CommandPool::new(&device);

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
        self.graphics_command_pool.destroy();
    }
}
