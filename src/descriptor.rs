// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::Arc;

use crate::*;

#[derive(Default)]
pub struct DescriptorKeyBuilder {
    pub layout: vk::PipelineLayout,
    pub model: Handle<RenderModel>,
    pub node: Handle<Node>,
    pub material: Handle<Material>,
    pub camera: Handle<Camera>,
}

impl DescriptorKeyBuilder {
    pub fn layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn model(mut self, model: Handle<RenderModel>) -> Self {
        self.model = model;
        self
    }

    pub fn node(mut self, node: Handle<Node>) -> Self {
        self.node = node;
        self
    }

    pub fn material(mut self, material: Handle<Material>) -> Self {
        self.material = material;
        self
    }

    pub fn camera(mut self, camera: Handle<Camera>) -> Self {
        self.camera = camera;
        self
    }

    pub fn build(self) -> DescriptorKey {
        DescriptorKey {
            layout: self.layout,
            model: self.model,
            node: self.node,
            material: self.material,
            camera: self.camera,
        }
    }
}

#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct DescriptorKey {
    /// Unique per pipeline
    pub layout: vk::PipelineLayout,

    /// Unique per glTF model
    pub model: Handle<RenderModel>,

    /// Unique per node, for model transforms
    pub node: Handle<Node>,

    /// Unique per material, for material buffers
    pub material: Handle<Material>,

    /// Unique per camera, for view and projection matrices
    pub camera: Handle<Camera>,
}

impl DescriptorKey {
    pub fn builder() -> DescriptorKeyBuilder {
        DescriptorKeyBuilder::default()
    }
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
    device: Arc<ash::Device>,
}

impl Descriptors {
    pub fn new(device: &Device) -> Self {
        let uniform_pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(device.properties.limits.max_descriptor_set_uniform_buffers) // Support 8 uniforms for 3 pipelines
            .ty(vk::DescriptorType::UNIFORM_BUFFER);
        let sampler_pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(device.properties.limits.max_descriptor_set_sampled_images) // Support 8 materials for 3 pipelines
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
        let input_pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(
                device
                    .properties
                    .limits
                    .max_descriptor_set_input_attachments,
            )
            .ty(vk::DescriptorType::INPUT_ATTACHMENT);

        let pool_sizes = vec![uniform_pool_size, sampler_pool_size, input_pool_size];
        let max_sets = device.properties.limits.max_descriptor_set_uniform_buffers
            + device.properties.limits.max_descriptor_set_sampled_images
            + device
                .properties
                .limits
                .max_descriptor_set_input_attachments;

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(max_sets);

        let pool = unsafe { device.create_descriptor_pool(&create_info, None) }
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

    #[test]
    fn key() {
        let key0 = DescriptorKey::builder().node(Handle::new(0)).build();
        let key1 = DescriptorKey::builder().material(Handle::new(0)).build();

        let mut hasher0 = DefaultHasher::new();
        let mut hasher1 = DefaultHasher::new();

        key0.hash(&mut hasher0);
        key1.hash(&mut hasher1);

        assert_ne!(hasher0.finish(), hasher1.finish());
    }
}
