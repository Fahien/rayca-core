// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_geometry::*;

use ash::vk;

use crate::*;

pub trait VertexInput {
    fn get_topology() -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::TRIANGLE_LIST
    }

    fn get_bindings() -> Vec<vk::VertexInputBindingDescription>;
    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription>;
}

impl VertexInput for LineVertex {
    fn get_topology() -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::LINE_LIST
    }

    fn get_bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<LineVertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX),
        ]
    }

    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(0),
        ]
    }
}

impl VertexInput for Vertex {
    fn get_bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<Self>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX),
        ]
    }

    fn get_attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(0),
        ]
    }
}
