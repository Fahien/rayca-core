// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::{mem::*, rc::Rc};

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
    gltf: Model,
    pub images: Pack<RenderImage>,
    pub views: Pack<ImageView>,
    pub samplers: Pack<RenderSampler>,
    pub textures: Pack<RenderTexture>,
    pub primitives: Pack<RenderPrimitive>,

    /// Useful for constructing the model continuously
    allocator: Rc<vk_mem::Allocator>,
}

impl RenderModel {
    pub fn new(allocator: &Rc<vk_mem::Allocator>) -> Self {
        Self {
            gltf: Default::default(),
            images: Pack::new(),
            views: Pack::new(),
            samplers: Pack::new(),
            textures: Pack::new(),
            primitives: Pack::new(),
            allocator: allocator.clone(),
        }
    }

    pub fn push_camera(&mut self, camera: Camera) -> Handle<Camera> {
        self.gltf.cameras.push(camera)
    }

    pub fn push_node(&mut self, node: Node) -> Handle<Node> {
        self.gltf.nodes.push(node)
    }

    pub fn push_to_scene(&mut self, node: Handle<Node>) {
        self.gltf.scene.children.push(node)
    }

    pub fn push_material(&mut self, material: Material) -> Handle<Material> {
        self.gltf.materials.push(material)
    }

    pub fn push_primitive(&mut self, primitive: Primitive) -> Handle<Primitive> {
        self.primitives
            .push(RenderPrimitive::from_gltf(&self.allocator, &primitive));
        self.gltf.primitives.push(primitive)
    }

    pub fn push_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.gltf.meshes.push(mesh)
    }

    pub fn push_script(&mut self, script: Script) -> Handle<Script> {
        self.gltf.scripts.push(script)
    }

    pub fn get_node(&self, node: Handle<Node>) -> Option<&Node> {
        self.gltf.nodes.get(node)
    }

    pub fn get_node_mut(&mut self, node: Handle<Node>) -> Option<&mut Node> {
        self.gltf.nodes.get_mut(node)
    }

    pub fn get_camera(&self, camera: Handle<Camera>) -> Option<&Camera> {
        self.gltf.cameras.get(camera)
    }

    pub fn get_camera_mut(&mut self, camera: Handle<Camera>) -> Option<&mut Camera> {
        self.gltf.cameras.get_mut(camera)
    }

    pub fn get_mesh(&self, mesh: Handle<Mesh>) -> Option<&Mesh> {
        self.gltf.meshes.get(mesh)
    }

    pub fn get_mesh_mut(&mut self, mesh: Handle<Mesh>) -> Option<&mut Mesh> {
        self.gltf.meshes.get_mut(mesh)
    }

    pub fn get_primitive(&self, primitive: Handle<Primitive>) -> Option<&Primitive> {
        self.gltf.primitives.get(primitive)
    }

    pub fn get_primitive_mut(&mut self, primitive: Handle<Primitive>) -> Option<&mut Primitive> {
        self.gltf.primitives.get_mut(primitive)
    }

    pub fn get_material(&self, material: Handle<Material>) -> Option<&Material> {
        self.gltf.materials.get(material)
    }

    pub fn get_material_mut(&mut self, material: Handle<Material>) -> Option<&mut Material> {
        self.gltf.materials.get_mut(material)
    }

    pub fn get_scene(&self) -> &Node {
        &self.gltf.scene
    }

    pub fn get_scene_mut(&mut self) -> &mut Node {
        &mut self.gltf.scene
    }

    pub fn get_gltf(&self) -> &Model {
        &self.gltf
    }

    pub fn get_gltf_mut(&mut self) -> &mut Model {
        &mut self.gltf
    }
}
