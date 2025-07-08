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
    pub fn empty<T>(allocator: &Rc<vk_mem::Allocator>) -> Self {
        Self {
            vertex_count: 0,
            vertices: Buffer::new::<T>(allocator, vk::BufferUsageFlags::VERTEX_BUFFER),
            indices: None,
        }
    }

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
        if self.indices.is_none() {
            self.indices.replace(Buffer::new::<u16>(
                &self.vertices.allocator,
                vk::BufferUsageFlags::INDEX_BUFFER,
            ));
        }
        self.indices.as_mut().unwrap().upload_arr(ii);
    }

    /// Returns a new primitive quad with side length 1 centered at the origin
    pub fn quad(allocator: &Rc<vk_mem::Allocator>, uv_scale: Vec2) -> Self {
        let vertices = vec![
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.0))
                .uv(Vec2::new(0.0, 1.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.0))
                .uv(Vec2::new(1.0, 1.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.0))
                .uv(Vec2::new(1.0, 0.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.0))
                .uv(Vec2::new(0.0, 0.0) * uv_scale)
                .build(),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];

        let mut ret = Self::new(allocator, &vertices);
        ret.set_indices(&indices);
        ret
    }

    pub fn cube(allocator: &Rc<vk_mem::Allocator>) -> Self {
        let vertices = vec![
            // Front
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Right
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Back
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Left
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Top
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Bottom
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
        ];

        let indices: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3, // front face
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // back
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // top
            20, 21, 22, 20, 22, 23, // bottom
        ];

        let mut ret = Self::new(allocator, &vertices);
        ret.set_indices(&indices);
        ret
    }
}
