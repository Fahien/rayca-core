// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, rc::Rc};

use ash::vk;

use crate::*;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DescriptorKey {
    // Unique per pipeline
    pub pipeline_layout: vk::PipelineLayout,
    // Unique per node, for model transforms
    pub node: Handle<Node>,
    // Unique per material, for material buffers
    pub material: Handle<Material>,
}

pub enum DescriptorEntry<'s> {
    Get(&'s [vk::DescriptorSet]),
    Created(&'s [vk::DescriptorSet]),
}

/// Per-frame resource which contains a descriptor pool and a vector
/// of descriptor sets of each pipeline layout used for rendering.
pub struct Descriptors {
    /// These descriptor sets are for model matrix uniforms, therefore we need
    /// NxM descriptor sets where N is the number of pipeline layouts, and M are
    /// nodes with the model matrix
    sets: HashMap<DescriptorKey, Vec<vk::DescriptorSet>>,
    pool: vk::DescriptorPool,
    device: Rc<ash::Device>,
}

impl Descriptors {
    pub fn new(device: &Device) -> Self {
        let pool = unsafe {
            let uniform_pool_size = vk::DescriptorPoolSize::default()
                .descriptor_count(4) // Support 1 model matrix and 1 view matrix for 2 pipelines
                .ty(vk::DescriptorType::UNIFORM_BUFFER);
            let sampler_pool_size = vk::DescriptorPoolSize::default()
                .descriptor_count(2) // Support 1 material for 2 pipelines
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);

            let pool_sizes = vec![uniform_pool_size, sampler_pool_size];
            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                // @todo Use a parameter instead of 2 for frame count
                .max_sets(2 * 2) // Support 2 frames with 2 pipelines
                ;
            device.create_descriptor_pool(&create_info, None)
        }
        .expect("Failed to create Vulkan descriptor pool");

        Self {
            sets: HashMap::new(),
            pool,
            device: device.device.clone(),
        }
    }

    pub fn allocate(&self, layouts: &[vk::DescriptorSetLayout]) -> Vec<vk::DescriptorSet> {
        assert!(!layouts.is_empty());
        let create_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(layouts);

        unsafe { self.device.allocate_descriptor_sets(&create_info) }
            .expect("Failed to allocate Vulkan descriptor sets")
    }

    #[allow(clippy::map_entry)]
    pub fn get_or_create<'a>(
        &'a mut self,
        key: DescriptorKey,
        layouts: &[vk::DescriptorSetLayout],
    ) -> DescriptorEntry<'a> {
        if self.sets.contains_key(&key) {
            DescriptorEntry::Get(self.sets.get(&key).unwrap())
        } else {
            self.sets.insert(key, self.allocate(layouts));
            DescriptorEntry::Created(self.sets.get(&key).unwrap())
        }
    }
}

impl Drop for Descriptors {
    fn drop(&mut self) {
        unsafe { self.device.destroy_descriptor_pool(self.pool, None) };
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::*;
    use ash::vk;

    #[test]
    fn key() {
        let key0 = DescriptorKey {
            pipeline_layout: vk::PipelineLayout::null(),
            node: Handle::new(0),
            material: Handle::NONE,
        };
        let key1 = DescriptorKey {
            pipeline_layout: vk::PipelineLayout::null(),
            node: Handle::NONE,
            material: Handle::new(0),
        };

        let mut hasher0 = DefaultHasher::new();
        let mut hasher1 = DefaultHasher::new();

        key0.hash(&mut hasher0);
        key1.hash(&mut hasher1);

        assert_ne!(hasher0.finish(), hasher1.finish());
    }
}
