// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;

use super::*;

pub struct RenderPrimitive {
    pub vertex_count: u32,
    pub vertices: Buffer,
    pub indices: Option<Buffer>,
}

impl RenderPrimitive {
    pub fn new<T>(allocator: &Rc<vk_mem::Allocator>, vv: &[T]) -> Self {
        let vertex_count = vv.len() as u32;

        let mut vertices = Buffer::new::<T>(allocator, vk::BufferUsageFlags::VERTEX_BUFFER);
        vertices.upload_arr(vv);

        Self {
            vertex_count,
            vertices,
            indices: None,
        }
    }

    pub fn set_indices(&mut self, ii: &[u16]) {
        let mut indices =
            Buffer::new::<u16>(&self.vertices.allocator, vk::BufferUsageFlags::INDEX_BUFFER);
        indices.upload_arr(ii);
        self.indices = Some(indices);
    }

    /// Returns a new primitive quad with side length 1 centered at the origin
    pub fn quad(allocator: &Rc<vk_mem::Allocator>) -> Self {
        let vertices = vec![
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.0))
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.0))
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.0))
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.0))
                .uv(Vec2::new(0.0, 0.0))
                .build(),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];

        let mut ret = Self::new(allocator, &vertices);
        ret.set_indices(&indices);
        ret
    }
}
