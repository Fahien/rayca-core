// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::any::Any;

use crate::*;
use rayca_pipe::*;

pipewriter!(
    Present,
    "shaders/present.vert.slang",
    "shaders/present.frag.slang"
);

pipewriter!(
    Normal,
    "shaders/present.vert.slang",
    "shaders/normal.frag.slang"
);

pipewriter!(
    Depth,
    "shaders/present.vert.slang",
    "shaders/depth.frag.slang"
);

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
        model: Option<&RenderModel>,
        camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    );
}

pub trait PipelinePool {
    /// Returns the render pipeline at position `index`
    fn get_at(&self, index: u32) -> &dyn RenderPipeline;
}

impl RenderPipeline for PipelinePresent {
    fn render(
        &self,
        frame: &mut Frame,
        _model: Option<&RenderModel>,
        _camera_nodes: &[Handle<Node>],
        _nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);

        let color_view_handle = vk::Handle::as_raw(frame.buffer.color_view.view);
        let key = DescriptorKey::builder()
            .layout(self.get_layout())
            .node(Handle::new(color_view_handle as _))
            .build();
        let color_texture = RenderTexture::new(
            &frame.buffer.color_view,
            &frame.cache.fallback.white_sampler,
        );
        let normal_texture = RenderTexture::new(
            &frame.buffer.normal_view,
            &frame.cache.fallback.white_sampler,
        );
        let depth_texture = RenderTexture::new(
            &frame.buffer.depth_view,
            &frame.cache.fallback.white_sampler,
        );
        self.bind_color_and_normal_and_depth(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            &color_texture,
            &normal_texture,
            &depth_texture,
        );
        self.draw(&frame.cache, &frame.cache.fallback.present_primitive);
    }
}

impl RenderPipeline for PipelineNormal {
    fn render(
        &self,
        frame: &mut Frame,
        _model: Option<&RenderModel>,
        _camera_nodes: &[Handle<Node>],
        _nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);

        let color_view_handle = vk::Handle::as_raw(frame.buffer.color_view.view);
        let key = DescriptorKey::builder()
            .layout(self.get_layout())
            .node(Handle::new(color_view_handle as _))
            .build();
        let color_texture = RenderTexture::new(
            &frame.buffer.color_view,
            &frame.cache.fallback.white_sampler,
        );
        let normal_texture = RenderTexture::new(
            &frame.buffer.normal_view,
            &frame.cache.fallback.white_sampler,
        );
        let depth_texture = RenderTexture::new(
            &frame.buffer.depth_view,
            &frame.cache.fallback.white_sampler,
        );
        self.bind_color_and_normal_and_depth(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            &color_texture,
            &normal_texture,
            &depth_texture,
        );
        self.draw(&frame.cache, &frame.cache.fallback.present_primitive);
    }
}

impl RenderPipeline for PipelineDepth {
    fn render(
        &self,
        frame: &mut Frame,
        _model: Option<&RenderModel>,
        _camera_nodes: &[Handle<Node>],
        _nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);

        let color_view_handle = vk::Handle::as_raw(frame.buffer.color_view.view);
        let key = DescriptorKey::builder()
            .layout(self.get_layout())
            .node(Handle::new(color_view_handle as _))
            .build();
        let color_texture = RenderTexture::new(
            &frame.buffer.color_view,
            &frame.cache.fallback.white_sampler,
        );
        let normal_texture = RenderTexture::new(
            &frame.buffer.normal_view,
            &frame.cache.fallback.white_sampler,
        );
        let depth_texture = RenderTexture::new(
            &frame.buffer.depth_view,
            &frame.cache.fallback.white_sampler,
        );
        self.bind_color_and_normal_and_depth(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            &color_texture,
            &normal_texture,
            &depth_texture,
        );
        self.draw(&frame.cache, &frame.cache.fallback.present_primitive);
    }
}
