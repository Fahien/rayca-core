// Copyright © 2021-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ffi::{CString, c_char};

use ash::{ext, khr, vk};

use crate::Win;

pub struct Ctx {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
}

impl Ctx {
    pub fn builder<'w>() -> CtxBuilder<'w> {
        CtxBuilder::default()
    }

    pub fn new(extensions_names: &[*const c_char]) -> Self {
        let layers = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_names: Vec<*const i8> = layers.iter().map(|name| name.as_ptr()).collect();

        let entry = unsafe { ash::Entry::load() }.expect("Failed to create ash entry");
        let app_info = vk::ApplicationInfo {
            p_application_name: "Test" as *const str as _,
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions_names)
            .enabled_layer_names(&layer_names);

        #[cfg(target_os = "macos")]
        let create_info = create_info.flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        Self { entry, instance }
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

pub struct CtxBuilder<'w> {
    debug: bool,
    win: Option<&'w Win>,
}

impl<'w> Default for CtxBuilder<'w> {
    fn default() -> Self {
        Self {
            debug: true,
            win: None,
        }
    }
}
impl<'w> CtxBuilder<'w> {
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn win(mut self, win: &'w Win) -> Self {
        self.win = Some(win);
        self
    }

    pub fn build(self) -> Ctx {
        let mut extensions_names = vec![];

        if self.debug {
            extensions_names.push(ext::debug_utils::NAME.as_ptr());
        }
        extensions_names.push(khr::surface::NAME.as_ptr());

        #[cfg(target_os = "linux")]
        extensions_names.push(khr::wayland_surface::NAME.as_ptr());

        #[cfg(target_os = "macos")]
        {
            extensions_names.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr());
            extensions_names.push(khr::get_physical_device_properties2::NAME.as_ptr());
            extensions_names.push(ext::metal_surface::NAME.as_ptr());
        }

        Ctx::new(&extensions_names)
    }
}
