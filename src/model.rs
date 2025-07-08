// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::mem::*;

use crate::*;

pub trait VertexInput {
    fn get_topology() -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::TRIANGLE_LIST
    }

    fn get_bindings() -> Vec<vk::VertexInputBindingDescription>;
    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription>;

    fn get_depth_state<'a>() -> vk::PipelineDepthStencilStateCreateInfo<'a> {
        vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::GREATER)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
    }

    fn get_color_blend() -> Vec<vk::PipelineColorBlendAttachmentState> {
        vec![
            vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA),
        ]
    }

    fn get_subpass() -> u32 {
        0
    }
}

impl VertexInput for LineVertex {
    fn get_topology() -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::LINE_STRIP
    }

    fn get_bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(size_of::<Self>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX),
        ]
    }

    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}

impl VertexInput for Vertex {
    fn get_bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(size_of::<Self>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX),
        ]
    }

    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, ext.color) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, ext.normal) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, ext.uv) as u32),
        ]
    }
}

/// Model representation useful for the renderer
pub struct RenderModel {
    pub gltf: Model,
    pub images: Pack<RenderImage>,
    pub views: Pack<ImageView>,
    pub samplers: Pack<RenderSampler>,
    pub textures: Pack<RenderTexture>,
    pub primitives: Pack<RenderPrimitive>,
}

impl Default for RenderModel {
    fn default() -> Self {
        Self {
            gltf: Default::default(),
            images: Pack::new(),
            views: Pack::new(),
            samplers: Pack::new(),
            textures: Pack::new(),
            primitives: Pack::new(),
        }
    }
}
