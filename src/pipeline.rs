// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;
use rayca_geometry::*;

use crate::*;

pub struct Pipeline {
    pub graphics: vk::Pipeline,
    device: Rc<ash::Device>,
}

impl Pipeline {
    pub fn new(
        dev: &mut Dev,
        vert: vk::PipelineShaderStageCreateInfo,
        frag: vk::PipelineShaderStageCreateInfo,
        pass: &Pass,
        width: u32,
        height: u32,
    ) -> Self {
        // Pipeline layout (device, shader reflection?)
        let layout = {
            let create_info = vk::PipelineLayoutCreateInfo::default();
            unsafe { dev.device.create_pipeline_layout(&create_info, None) }
                .expect("Failed to create Vulkan pipeline layout")
        };

        // Graphics pipeline (shaders, renderpass)
        let graphics = {
            let vertex_bindings = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];

            let vertex_attributes = [vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0)];

            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_attributes)
                .vertex_binding_descriptions(&vertex_bindings);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
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

        unsafe {
            dev.device.destroy_pipeline_layout(layout, None);
        }

        Self {
            graphics,
            device: Rc::clone(&dev.device),
        }
    }

    pub fn draw(&self, frame: &Frame, buffer: &Buffer) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;
        unsafe {
            self.device
                .cmd_bind_pipeline(frame.command_buffer, graphics_bind_point, self.graphics)
        };

        let first_binding = 0;
        let buffers = [buffer.buffer];
        let offsets = [vk::DeviceSize::default()];
        unsafe {
            self.device.cmd_bind_vertex_buffers(
                frame.command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
            self.device.cmd_draw(frame.command_buffer, 3, 1, 0, 0);
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.graphics, None);
        }
    }
}
