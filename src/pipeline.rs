// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::any::Any;

use crate::*;

pub trait Pipeline: Any {
    fn as_any(&self) -> &dyn Any;
    fn get_name(&self) -> &String;
    fn get_set_layouts(&self) -> &[vk::DescriptorSetLayout];
    fn get_layout(&self) -> vk::PipelineLayout;
    fn get_pipeline(&self) -> vk::Pipeline;
    fn get_device(&self) -> &ash::Device;
    fn get_vertex_size(&self) -> usize;

    fn bind(&self, cache: &FrameCache) {
        cache.command_buffer.bind_pipeline(self.get_pipeline());
    }

    fn draw(&self, cache: &FrameCache, primitive: &RenderPrimitive) {
        cache.command_buffer.bind_vertex_buffer(&primitive.vertices);

        if let Some(indices) = &primitive.indices {
            // Draw indexed if primitive has indices
            cache
                .command_buffer
                .bind_index_buffer(indices, primitive.index_type);

            cache
                .command_buffer
                .draw_indexed(primitive.get_index_count(), 0, 0);
        } else {
            // Draw without indices
            cache.command_buffer.draw(primitive.vertex_count);
        }
    }
}

pub trait RenderPipeline: Pipeline {
    /// This needs to be manually implemented, as the generator does not know where to
    /// find the various buffers to bind and in which order and frequency to bind them
    fn render(
        &self,
        frame: &mut Frame,
        model: &RenderModel,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    );
}

pub trait PipelinePool {
    /// Returns the render pipeline at position `index`
    fn get_at(&self, index: u32) -> &dyn RenderPipeline;
}
