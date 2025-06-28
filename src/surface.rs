// Copyright © 2021-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

use ash::{khr, vk};
use winit::raw_window_handle::*;

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    pub ext: khr::surface::Instance,
}

impl Surface {
    pub fn new(win: &Win, ctx: &Ctx) -> Self {
        let window = win.window.as_ref().unwrap();

        let display_handle = window
            .display_handle()
            .expect("Failed to get display handle")
            .as_raw();
        let window_handle = window
            .window_handle()
            .expect("Failed to get window handle")
            .as_raw();
        let surface = match (display_handle, window_handle) {
            (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
                let wayland_surface =
                    khr::wayland_surface::Instance::new(&ctx.entry, &ctx.instance);
                let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
                    .display(display.display.as_ptr())
                    .surface(window.surface.as_ptr());
                unsafe { wayland_surface.create_wayland_surface(&create_info, None) }
                    .expect("Failed to create wayland surface")
            }
            #[cfg(target_os = "macos")]
            (RawDisplayHandle::AppKit(_display), RawWindowHandle::AppKit(window)) => {
                let metal_surface =
                    ash::ext::metal_surface::Instance::new(&ctx.entry, &ctx.instance);
                // On-screen rendering requires a layer of type CAMetalLayer
                let metal_layer = unsafe { raw_window_metal::Layer::from_ns_view(window.ns_view) };
                let create_info =
                    vk::MetalSurfaceCreateInfoEXT::default().layer(metal_layer.as_ptr().as_ptr());
                unsafe { metal_surface.create_metal_surface(&create_info, None) }
                    .expect("Failed to create metal surface")
            }
            _ => unimplemented!("{:?}", display_handle),
        };

        let ext = khr::surface::Instance::new(&ctx.entry, &ctx.instance);

        Self { surface, ext }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.ext.destroy_surface(self.surface, None);
        }
    }
}
