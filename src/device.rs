// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{ffi::CStr, sync::Arc};

use ash::{khr, vk};

use crate::*;

pub struct Device {
    pub graphics_queue_index: u32,
    pub properties: vk::PhysicalDeviceProperties,
    pub physical: vk::PhysicalDevice,
    pub device: Arc<ash::Device>,
}

impl Device {
    fn get_graphics_queue_index(
        instance: &ash::Instance,
        physical: vk::PhysicalDevice,
        surface: Option<&Surface>,
    ) -> u32 {
        // Queue information (instance, physical device)
        let queue_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical) };

        let mut graphics_queue_index = u32::MAX;

        for (i, queue) in queue_properties.iter().enumerate() {
            let mut supports_presentation = true;

            if let Some(surface) = surface {
                supports_presentation = unsafe {
                    surface.ext.get_physical_device_surface_support(
                        physical,
                        i as u32,
                        surface.surface,
                    )
                }
                .expect("Failed to check presentation support for Vulkan physical device");
            }

            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) && supports_presentation {
                graphics_queue_index = i as u32;
                break;
            }
        }

        assert!(
            graphics_queue_index != u32::MAX,
            "Failed to find graphics queue"
        );

        graphics_queue_index
    }

    pub fn new(instance: &ash::Instance, surface: Option<&Surface>) -> Self {
        // Physical device
        let physical = {
            let phydevs = unsafe {
                instance
                    .enumerate_physical_devices()
                    .expect("Failed to enumerate Vulkan physical devices")
            };
            phydevs[0]
        };
        let properties = unsafe { instance.get_physical_device_properties(physical) };
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
        println!("Physical device: {:?}", name);

        let graphics_queue_index = Self::get_graphics_queue_index(instance, physical, surface);

        // Logical device (physical device, surface, device required extensions (swapchain), queue information)
        let queue_infos = vec![
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(graphics_queue_index)
                // Highest priority for a single graphics queue
                .queue_priorities(&[1.0]),
        ];

        let mut device_extensions = vec![];

        #[cfg(target_os = "macos")]
        device_extensions.push(khr::portability_subset::NAME.as_ptr());

        if surface.is_some() {
            device_extensions.push(khr::swapchain::NAME.as_ptr());
        }

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions);

        let device = unsafe { instance.create_device(physical, &device_create_info, None) }
            .expect("Failed to create Vulkan logical device");

        let properties = unsafe { instance.get_physical_device_properties(physical) };

        Self {
            graphics_queue_index,
            properties,
            physical,
            device: Arc::new(device),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        assert_eq!(Arc::strong_count(&self.device), 1);
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl std::ops::Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
