// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{any::Any, rc::Rc};

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

pub struct DefaultPipeline {
    vertex_size: usize,
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
                .min_depth(1.0)
                .max_depth(0.0)];

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

            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false);

            let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA)];

            let blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(&blend_attachment);

            let stages = [vert, frag];

            let dynamic_states = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&[vk::DynamicState::VIEWPORT]);

            let create_info = [vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .layout(layout)
                .render_pass(pass.render)
                .subpass(V::get_subpass())
                .vertex_input_state(&vertex_input)
                .input_assembly_state(&input_assembly)
                .rasterization_state(&raster_state)
                .viewport_state(&view_state)
                .multisample_state(&multisample_state)
                .depth_stencil_state(&depth_state)
                .color_blend_state(&blend_state)
                .dynamic_state(&dynamic_states)];
            let pipelines = unsafe {
                dev.device
                    .create_graphics_pipelines(vk::PipelineCache::null(), &create_info, None)
            }
            .expect("Failed to create Vulkan graphics pipeline");

            pipelines[0]
        };

        Self {
            vertex_size: std::mem::size_of::<V>(),
            set_layouts,
            layout,
            graphics,
            device: dev.device.device.clone(),
            name: String::from("LegacyPipeline"),
        }
    }

    pub fn bind_model_buffer(&self, cache: &mut FrameCache, model: &Model, node: Handle<Node>) {
        // A model buffer must already available at this point
        let buffer = cache.model_buffers.get_mut(&node).unwrap();
        buffer.upload(&model.nodes.get(node).unwrap().trs.to_mat4());

        let key = DescriptorKey::builder()
            .layout(self.get_layout())
            .node(node)
            .build();
        let sets: &[vk::DescriptorSet] =
            match cache.descriptors.get_or_create(key, self.get_set_layouts()) {
                DescriptorEntry::Created(sets) => {
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
                    }
                    sets
                }
                DescriptorEntry::Get(sets) => sets,
            };

        cache
            .command_buffer
            .bind_descriptor_sets(self.get_layout(), sets, 0);
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

    fn get_vertex_size(&self) -> usize {
        self.vertex_size
    }
}

impl RenderPipeline for DefaultPipeline {
    fn render(
        &self,
        frame: &mut Frame,
        model: &RenderModel,
        _camera_nodes: &[Handle<Node>],
        nodes: &[Handle<Node>],
    ) {
        self.bind(&frame.cache);
        for node_handle in nodes.iter().copied() {
            self.bind_model_buffer(&mut frame.cache, &model.gltf, node_handle);
            let node = model.gltf.nodes.get(node_handle).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let vertex_buffer = &model.primitives.get(mesh.primitive.id.into()).unwrap();
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
