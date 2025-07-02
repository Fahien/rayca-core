// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, rc::Rc};

use ash::vk;

use crate::*;

/// Per-frame resource which contains a descriptor pool and a vector
/// of descriptor sets of each pipeline layout used for rendering.
pub struct Descriptors {
    /// These descriptor sets are for model matrix uniforms, therefore we need
    /// NxM descriptor sets where N is the number of pipeline layouts, and M are
    /// nodes with the model matrix
    pub sets: HashMap<(vk::PipelineLayout, Handle<Node>), Vec<vk::DescriptorSet>>,

    pool: vk::DescriptorPool,
    device: Rc<ash::Device>,
}

impl Descriptors {
    pub fn new(device: &Device) -> Self {
        let pool = unsafe {
            // 2 uniform buffers, one for the line pipeline andd one for the main pipeline
            let uniform_pool_size = vk::DescriptorPoolSize::default()
                .descriptor_count(4)
                .ty(vk::DescriptorType::UNIFORM_BUFFER);
            // 1 sampler for the main pipeline
            let sampler_pool_size = vk::DescriptorPoolSize::default()
                .descriptor_count(2)
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);

            let pool_sizes = vec![uniform_pool_size, sampler_pool_size];
            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                // Support 2 frames?
                .max_sets(4);
            device.create_descriptor_pool(&create_info, None)
        }
        .expect("Failed to create Vulkan descriptor pool");

        Self {
            sets: HashMap::new(),
            pool,
            device: device.device.clone(),
        }
    }

    pub fn allocate(&mut self, layouts: &[vk::DescriptorSetLayout]) -> Vec<vk::DescriptorSet> {
        assert!(!layouts.is_empty());
        let create_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(layouts);

        unsafe { self.device.allocate_descriptor_sets(&create_info) }
            .expect("Failed to allocate Vulkan descriptor sets")
    }
}

impl Drop for Descriptors {
    fn drop(&mut self) {
        unsafe { self.device.destroy_descriptor_pool(self.pool, None) };
    }
}
