// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use ash::vk;
use vk_mem::Alloc;

use crate::*;

pub struct RenderBuffer {
    allocation: vk_mem::Allocation,
    pub buffer: vk::Buffer,
    usage: vk::BufferUsageFlags,
    pub size: vk::DeviceSize,
    pub allocator: Arc<Allocator>,
}

impl RenderBuffer {
    fn create_buffer(
        allocator: &vk_mem::Allocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> (vk::Buffer, vk_mem::Allocation) {
        let buffer_info = vk::BufferCreateInfo::default()
            // Minimum size is 16 bytes
            .size(size.max(16))
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

    pub fn new_with_size(
        allocator: &Arc<Allocator>,
        usage: vk::BufferUsageFlags,
        size: vk::DeviceSize,
    ) -> Self {
        let (buffer, allocation) = Self::create_buffer(allocator, size, usage);

        Self {
            allocation,
            buffer,
            size,
            usage,
            allocator: allocator.clone(),
        }
    }

    pub fn new<T>(allocator: &Arc<Allocator>, usage: vk::BufferUsageFlags) -> Self {
        let size = std::mem::size_of::<T>() as vk::DeviceSize;
        Self::new_with_size(allocator, usage, size)
    }

    pub fn from_data(allocator: &Arc<Allocator>, data: &[u8], usage: vk::BufferUsageFlags) -> Self {
        let mut buffer = Self::new_with_size(allocator, usage, data.len() as vk::DeviceSize);
        buffer.upload_arr(data);
        buffer
    }

    /// Loads data from a png image in `path` directly into a staging buffer
    pub fn load(allocator: &Arc<Allocator>, image: ::image::RgbaImage) -> Self {
        let size = image.len();
        let usage = vk::BufferUsageFlags::TRANSFER_SRC;

        // Create staging buffer
        let (buffer, mut allocation) =
            Self::create_buffer(allocator, size as vk::DeviceSize, usage);

        let data =
            unsafe { allocator.map_memory(&mut allocation) }.expect("Failed to map Vulkan memory");

        // Allocate the output buffer
        let buf = unsafe { std::slice::from_raw_parts_mut(data, size) };

        let bytes = image.into_vec();
        buf.copy_from_slice(&bytes);

        unsafe { allocator.unmap_memory(&mut allocation) };

        Self {
            allocation,
            buffer,
            usage,
            size: size as vk::DeviceSize,
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
        if size != self.size {
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

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            self.allocator
                .destroy_buffer(self.buffer, &mut self.allocation)
        };
    }
}
