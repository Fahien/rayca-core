// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crate::*;

/// Container of fallback resources for a frame such as
/// A white 1x1 pixel texture (image, view, and sampler)
pub struct Fallback {
    _white_image: RenderImage,
    _white_view: ImageView,
    pub white_sampler: RenderSampler,
    pub white_texture: RenderTexture,
    pub white_buffer: Buffer,
    pub white_material: Material,

    /// A triangle that covers the whole screen
    pub present_primitive: RenderPrimitive,
}

impl Fallback {
    pub fn new(allocator: &Arc<Allocator>, graphics_queue: &GraphicsQueue) -> Self {
        let white = [255, 255, 255, 255];
        let white_image = RenderImage::from_data(
            allocator,
            graphics_queue,
            &white,
            1,
            1,
            vk::Format::R8G8B8A8_SRGB,
        );
        let white_view = ImageView::new(&white_image);
        let white_sampler = RenderSampler::new(&allocator.device.device);
        let white_texture = RenderTexture::new(&white_view, &white_sampler);
        let mut white_buffer =
            Buffer::new::<Color>(allocator, vk::BufferUsageFlags::UNIFORM_BUFFER);
        white_buffer.upload(&Color::WHITE);
        let white_material = Material::default();

        // Y pointing down
        let present_vertices = vec![
            PresentVertex::new(-1.0, -1.0),
            PresentVertex::new(-1.0, 3.0),
            PresentVertex::new(3.0, -1.0),
        ];
        let present_primitive = RenderPrimitive::new(allocator, &present_vertices);

        Self {
            _white_image: white_image,
            _white_view: white_view,
            white_sampler,
            white_texture,
            white_buffer,
            white_material,
            present_primitive,
        }
    }
}
