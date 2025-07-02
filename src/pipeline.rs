// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{any::Any, rc::Rc};

use crate::*;
use ash::vk;

pub trait Pipeline: Any {
    fn as_any(&self) -> &dyn Any;
    fn get_name(&self) -> &String;
    fn get_set_layouts(&self) -> &[vk::DescriptorSetLayout];
    fn get_layout(&self) -> vk::PipelineLayout;
    fn get_pipeline(&self) -> vk::Pipeline;
    fn get_device(&self) -> &ash::Device;

    fn bind(&self, cache: &FrameCache) {
        unsafe {
            cache.device.cmd_bind_pipeline(
                cache.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.get_pipeline(),
            );
        }
    }

    fn draw(&self, cache: &FrameCache, vertex_buffer: &Buffer) {
        let first_binding = 0;
        let buffers = [vertex_buffer.buffer];
        let offsets = [vk::DeviceSize::default()];
        unsafe {
            cache.device.cmd_bind_vertex_buffers(
                cache.command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
        }

        let vertex_count = vertex_buffer.size as u32 / std::mem::size_of::<Vertex>() as u32;
        unsafe {
            cache
                .device
                .cmd_draw(cache.command_buffer, vertex_count, 1, 0, 0);
        }
    }
}

pub trait RenderPipeline: Pipeline {
    /// This needs to be manually implemented, as the generator does not know where to
    /// find the various buffers to bind and in which order and frequency to bind them
    fn render(&self, frame: &mut Frame, model: &RenderModel, nodes: &[Handle<Node>]);
}

pub trait PipelinePool {
    /// Returns the render pipeline at position `index`
    fn get_at(&self, index: u32) -> &dyn RenderPipeline;
}

pub struct DefaultPipeline {
    set_layouts: Vec<vk::DescriptorSetLayout>,
    layout: vk::PipelineLayout,
    pub graphics: vk::Pipeline,
    device: Rc<ash::Device>,
    name: String,
}

impl DefaultPipeline {
    pub fn new<V: VertexInput>(
        dev: &mut Dev,
        vert: vk::PipelineShaderStageCreateInfo,
        frag: vk::PipelineShaderStageCreateInfo,
        topology: vk::PrimitiveTopology,
        pass: &Pass,
        width: u32,
        height: u32,
    ) -> Self {
        let set_layout_bindings = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER) // delta time?
            .descriptor_count(1) // can specify more?
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        let arr_bindings = vec![set_layout_bindings];

        let set_layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&arr_bindings);

        let set_layout = unsafe {
            dev.device
                .create_descriptor_set_layout(&set_layout_info, None)
        }
        .expect("Failed to create Vulkan descriptor set layout");

        let set_layouts = vec![set_layout];

        // Pipeline layout (device, descriptorset layouts, shader reflection?)
        let layout = {
            let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
            unsafe { dev.device.create_pipeline_layout(&create_info, None) }
                .expect("Failed to create Vulkan pipeline layout")
        };

        // Graphics pipeline (shaders, renderpass)
        let graphics = {
            let vertex_attributes = V::get_attributes();
            let vertex_bindings = V::get_bindings();

            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_attributes)
                .vertex_binding_descriptions(&vertex_bindings);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(topology)
                .primitive_restart_enable(false);

            let raster_state = vk::PipelineRasterizationStateCreateInfo::default()
                .line_width(1.0)
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let viewports = [vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(width as f32)
                .height(height as f32)
                .min_depth(0.0)
                .max_depth(1.0)];

            let scissors = [vk::Rect2D::default()
                .offset(vk::Offset2D::default().x(0).y(0))
                .extent(vk::Extent2D::default().width(width).height(height))];

            let view_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(&viewports)
                .scissors(&scissors);

            let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .sample_shading_enable(false)
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false);

            let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA)];

            let blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(&blend_attachment);

            let stages = [vert, frag];

            let create_info = [vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .layout(layout)
                .render_pass(pass.render)
                .subpass(0)
                .vertex_input_state(&vertex_input)
                .input_assembly_state(&input_assembly)
                .rasterization_state(&raster_state)
                .viewport_state(&view_state)
                .multisample_state(&multisample_state)
                .color_blend_state(&blend_state)];
            let pipelines = unsafe {
                dev.device
                    .create_graphics_pipelines(vk::PipelineCache::null(), &create_info, None)
            }
            .expect("Failed to create Vulkan graphics pipeline");

            pipelines[0]
        };

        Self {
            set_layouts,
            layout,
            graphics,
            device: dev.device.device.clone(),
            name: String::from("LegacyPipeline"),
        }
    }

    pub fn bind_model_buffer(
        &self,
        cache: &mut FrameCache,
        model: &RenderModel,
        node: Handle<Node>,
    ) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;

        // A model buffer must already available at this point
        let buffer = cache.uniforms.get_mut(&node).unwrap();
        buffer.upload(&model.gltf.nodes.get(node).unwrap().trs.to_mat4());

        if let Some(sets) = cache.descriptors.sets.get(&(self.get_layout(), node)) {
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    cache.command_buffer,
                    graphics_bind_point,
                    self.get_layout(),
                    0,
                    sets,
                    &[],
                );
            }
        } else {
            let sets = cache.descriptors.allocate(self.get_set_layouts());

            // Update immediately the descriptor sets
            let buffer_info = [vk::DescriptorBufferInfo::default()
                .range(std::mem::size_of::<Mat4>() as vk::DeviceSize)
                .buffer(buffer.buffer)];

            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(sets[0])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_info);

            unsafe {
                self.device.update_descriptor_sets(&[descriptor_write], &[]);

                self.device.cmd_bind_descriptor_sets(
                    cache.command_buffer,
                    graphics_bind_point,
                    self.get_layout(),
                    0,
                    &sets,
                    &[],
                );
            }

            cache
                .descriptors
                .sets
                .insert((self.get_layout(), node), sets);
        }
    }
}

impl Pipeline for DefaultPipeline {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_set_layouts(&self) -> &[vk::DescriptorSetLayout] {
        &self.set_layouts
    }

    fn get_layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    fn get_pipeline(&self) -> vk::Pipeline {
        self.graphics
    }

    fn get_device(&self) -> &ash::Device {
        &self.device
    }
}

impl RenderPipeline for DefaultPipeline {
    fn render(&self, frame: &mut Frame, model: &RenderModel, nodes: &[Handle<Node>]) {
        self.bind(&frame.cache);
        for node_handle in nodes.iter().copied() {
            self.bind_model_buffer(&mut frame.cache, model, node_handle);
            let node = model.gltf.nodes.get(node_handle).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let vertex_buffer = &model.vertex_buffers.get(mesh.primitive.id.into()).unwrap();
            self.draw(&frame.cache, vertex_buffer);
        }
    }
}

impl Drop for DefaultPipeline {
    fn drop(&mut self) {
        unsafe {
            for &set_layout in &self.set_layouts {
                self.device.destroy_descriptor_set_layout(set_layout, None);
            }
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_pipeline(self.graphics, None);
        }
    }
}
