// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use ash::vk;
use std::{mem::*, sync::Arc};

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
        let mut ret = vec![
            vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA),
        ];
        if Self::get_subpass() == 0 {
            ret.push(ret[0]);
        }
        ret
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

/// Very simple vertex used for the presentation pass
#[repr(C)]
pub struct PresentVertex {
    /// The shader just needs x and y
    pub pos: Vec2,
}

impl PresentVertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
        }
    }

    pub fn write_set(
        device: &ash::Device,
        set: vk::DescriptorSet,
        albedo: &ImageView,
        sampler: &RenderSampler,
    ) {
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(albedo.view)
            .sampler(sampler.sampler)];

        let image_write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .image_info(&image_info);

        let writes = vec![image_write];

        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }
    }
}

impl VertexInput for PresentVertex {
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
            // position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
        ]
    }

    fn get_subpass() -> u32 {
        1
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
    dev: Arc<Dev>,
}

impl RenderModel {
    pub fn new(dev: &Arc<Dev>) -> Self {
        Self {
            gltf: Default::default(),
            images: Pack::new(),
            views: Pack::new(),
            samplers: Pack::new(),
            textures: Pack::new(),
            primitives: Pack::new(),
            dev: dev.clone(),
        }
    }

    /// Returns a render model with a default camera positioned at the origin,
    /// looking down the negative Z axis
    pub fn default(dev: &Arc<Dev>) -> Self {
        let mut ret = Self::new(dev);
        let hcamera = ret.push_camera(Camera::default());
        let hnode = ret.push_node(
            Node::builder()
                .camera(hcamera)
                // Slightly move the camera up
                .trs(Trs::builder().translation(Vec3::new(0.0, 2.0, 0.0)).build())
                .build(),
        );
        ret.push_to_scene(hnode);
        ret
    }

    pub fn new_with_gltf(dev: &Arc<Dev>, assets: &Assets, gltf: Model) -> Self {
        let mut ret = Self::new(dev);

        // Load images concurrently
        use rayon::iter::*;

        let render_images: Vec<RenderImage> = gltf
            .images
            .par_iter()
            .map(|image| {
                RenderImage::load(&dev.allocator, &dev.graphics_queue, assets.load(&image.uri))
            })
            .collect();
        for image in render_images {
            ret.push_render_image(image);
        }
        for sampler in gltf.samplers.iter() {
            ret.push_render_sampler(sampler);
        }
        for texture in gltf.textures.iter() {
            ret.push_render_texture(texture);
        }
        for primitive in gltf.primitives.iter() {
            ret.push_render_primitive(primitive);
        }

        ret.gltf = gltf;
        ret
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

    fn push_render_image(&mut self, image: RenderImage) {
        let view = ImageView::new(&image);
        self.images.push(image);
        self.views.push(view);
    }

    pub fn push_image(&mut self, image: Image, assets: &Assets) -> Handle<Image> {
        let image_asset = assets.load(&image.uri);
        let render_image =
            RenderImage::load(&self.dev.allocator, &self.dev.graphics_queue, image_asset);
        self.push_render_image(render_image);
        self.gltf.images.push(image)
    }

    fn push_render_sampler(&mut self, _sampler: &Sampler) {
        let sampler = RenderSampler::new(&self.dev.device.device);
        self.samplers.push(sampler);
    }

    pub fn push_sampler(&mut self, sampler: Sampler) -> Handle<Sampler> {
        self.push_render_sampler(&sampler);
        self.gltf.samplers.push(sampler)
    }

    fn push_render_texture(&mut self, texture: &Texture) {
        let view = self.views.get(texture.image.id.into()).unwrap();
        let sampler = match self.samplers.get(texture.sampler.id.into()) {
            Some(s) => s,
            None => &self.dev.fallback.white_sampler,
        };
        let texture = RenderTexture::new(&view, &sampler);
        self.textures.push(texture);
    }

    pub fn push_texture(&mut self, texture: Texture) -> Handle<Texture> {
        self.push_render_texture(&texture);
        self.gltf.textures.push(texture)
    }

    pub fn push_material(&mut self, material: Material) -> Handle<Material> {
        self.gltf.materials.push(material)
    }

    fn push_render_primitive(&mut self, primitive: &Primitive) {
        self.primitives
            .push(RenderPrimitive::from_gltf(&self.dev.allocator, &primitive));
    }

    pub fn push_primitive(&mut self, primitive: Primitive) -> Handle<Primitive> {
        self.push_render_primitive(&primitive);
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

    pub fn get_root(&self) -> &Node {
        &self.gltf.scene
    }

    pub fn get_root_mut(&mut self) -> &mut Node {
        &mut self.gltf.scene
    }

    pub fn get_gltf(&self) -> &Model {
        &self.gltf
    }

    pub fn get_gltf_mut(&mut self) -> &mut Model {
        &mut self.gltf
    }

    pub fn get_first_node_with_camera(&self) -> Handle<Node> {
        // For the moment, return the first camera in the first model
        for hnode in self.gltf.nodes.get_handles() {
            let node = self.gltf.nodes.get(hnode).unwrap();
            if node.camera.is_valid() {
                return hnode;
            }
        }
        Handle::NONE
    }
}
