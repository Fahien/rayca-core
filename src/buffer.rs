// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;
use vk_mem::Alloc;

use crate::*;

pub struct Buffer {
    allocation: vk_mem::Allocation,
    pub buffer: vk::Buffer,
    usage: vk::BufferUsageFlags,
    pub size: vk::DeviceSize,
    allocator: Rc<vk_mem::Allocator>,
}

impl Buffer {
    fn create_buffer(
        allocator: &vk_mem::Allocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> (vk::Buffer, vk_mem::Allocation) {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        // Vulkan memory
        let create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            required_flags: vk::MemoryPropertyFlags::HOST_VISIBLE,
            preferred_flags: vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_CACHED,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &create_info) }
            .expect("Failed to create Vulkan buffer");

        (buffer, allocation)
    }

    pub fn new<T>(allocator: &Rc<vk_mem::Allocator>, usage: vk::BufferUsageFlags) -> Self {
        let size = std::mem::size_of::<T>() as vk::DeviceSize;

        let (buffer, allocation) = Self::create_buffer(allocator, size, usage);

        Self {
            allocation,
            buffer,
            usage,
            size,
            allocator: allocator.clone(),
        }
    }

    pub fn upload<T>(&mut self, data: &T) {
        self.upload_raw(data as *const T, std::mem::size_of::<T>() as vk::DeviceSize);
    }

    pub fn upload_raw<T>(&mut self, src: *const T, size: vk::DeviceSize) {
        let data = unsafe { self.allocator.map_memory(&mut self.allocation) }
            .expect("Failed to map Vulkan memory");
        unsafe { data.copy_from(src as _, size as usize) };
        unsafe { self.allocator.unmap_memory(&mut self.allocation) };
    }

    pub fn upload_arr<T>(&mut self, arr: &[T]) {
        // Create a new buffer if not enough size for the vector
        let size = std::mem::size_of_val(arr) as vk::DeviceSize;
        if size as vk::DeviceSize != self.size {
            unsafe {
                self.allocator
                    .destroy_buffer(self.buffer, &mut self.allocation)
            };

            self.size = size;
            let (buffer, allocation) = Self::create_buffer(&self.allocator, size, self.usage);
            self.buffer = buffer;
            self.allocation = allocation;
        }

        self.upload_raw(arr.as_ptr(), size);
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.allocator
                .destroy_buffer(self.buffer, &mut self.allocation)
        };
    }
}
