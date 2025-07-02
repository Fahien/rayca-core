// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;

pub struct Semaphore {
    pub semaphore: vk::Semaphore,
    device: Rc<ash::Device>,
}

impl Semaphore {
    pub fn new(device: &Rc<ash::Device>) -> Self {
        let create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe { device.create_semaphore(&create_info, None) }
            .expect("Failed to create Vulkan semaphore");

        Self {
            semaphore,
            device: device.clone(),
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe { self.device.destroy_semaphore(self.semaphore, None) };
    }
}

pub struct Fence {
    /// Ideally only Queue::submit should be allowed to modify this flag
    pub can_wait: bool,
    pub fence: vk::Fence,
    device: Rc<ash::Device>,
}

impl Fence {
    pub fn new(device: &Rc<ash::Device>, flags: vk::FenceCreateFlags) -> Self {
        let can_wait = flags.contains(vk::FenceCreateFlags::SIGNALED);

        let create_info = vk::FenceCreateInfo::default().flags(flags);
        let fence = unsafe { device.create_fence(&create_info, None) }
            .expect("Failed to create Vulkan fence");

        Self {
            can_wait,
            fence,
            device: device.clone(),
        }
    }

    pub fn unsignaled(device: &Rc<ash::Device>) -> Self {
        Self::new(device, vk::FenceCreateFlags::default())
    }

    pub fn signaled(device: &Rc<ash::Device>) -> Self {
        Self::new(device, vk::FenceCreateFlags::SIGNALED)
    }

    pub fn wait(&mut self) {
        if self.can_wait {
            unsafe { self.device.wait_for_fences(&[self.fence], true, u64::MAX) }
                .expect("Failed waiting for Vulkan fence");
            self.can_wait = false;
        }
    }

    pub fn reset(&mut self) {
        self.can_wait = false;
        unsafe { self.device.reset_fences(&[self.fence]) }.expect("Failed to reset Vulkan fence");
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.wait();
        unsafe {
            self.device.destroy_fence(self.fence, None);
        }
    }
}
