// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;

use super::*;

fn size_of(index_type: vk::IndexType) -> usize {
    match index_type {
        vk::IndexType::UINT16 => std::mem::size_of::<u16>(),
        vk::IndexType::UINT32 => std::mem::size_of::<u32>(),
        vk::IndexType::UINT8_EXT => std::mem::size_of::<u8>(),
        _ => unreachable!(),
    }
}

pub struct RenderPrimitive {
    pub vertex_count: u32,
    pub vertices: Buffer,
    pub indices: Option<Buffer>,
    pub index_type: vk::IndexType,
}

impl RenderPrimitive {
    pub fn empty<T>(allocator: &Rc<vk_mem::Allocator>) -> Self {
        Self {
            vertex_count: 0,
            vertices: Buffer::new::<T>(allocator, vk::BufferUsageFlags::VERTEX_BUFFER),
            indices: None,
            index_type: vk::IndexType::UINT16,
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
            index_type: vk::IndexType::UINT16,
        }
    }

    pub fn set_indices(&mut self, ii: &[u8], index_type: vk::IndexType) {
        if self.indices.is_none() {
            self.indices.replace(Buffer::new::<u8>(
                &self.vertices.allocator,
                vk::BufferUsageFlags::INDEX_BUFFER,
            ));
        }
        self.indices.as_mut().unwrap().upload_arr(ii);
        self.index_type = index_type;
    }

    pub fn get_index_count(&self) -> u32 {
        if let Some(indices) = &self.indices {
            indices.size as u32 / size_of(self.index_type) as u32
        } else {
            0
        }
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
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

        let mut ret = Self::new(allocator, &vertices);
        ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16);
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
        ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16);
        ret
    }

    pub fn from_gltf(allocator: &Rc<vk_mem::Allocator>, gltf_primitive: &Primitive) -> Self {
        // Convert vertices
        let mut ret = match gltf_primitive.mode {
            PrimitiveMode::Points => todo!(),
            PrimitiveMode::LineLoop => todo!(),
            PrimitiveMode::Lines | PrimitiveMode::LineStrip => {
                let vertices: Vec<LineVertex> = gltf_primitive
                    .vertices
                    .iter()
                    .map(LineVertex::from)
                    .collect();
                Self::new(allocator, &vertices)
            }
            PrimitiveMode::Triangles => Self::new(allocator, &gltf_primitive.vertices),
            PrimitiveMode::TriangleStrip => todo!(),
            PrimitiveMode::TriangleFan => todo!(),
        };

        // Convert indices
        if let Some(indices) = &gltf_primitive.indices {
            match indices.index_type {
                ComponentType::I8 => {
                    let indices: &[i8] = unsafe {
                        std::slice::from_raw_parts(
                            indices.indices.as_ptr() as _,
                            indices.indices.len(),
                        )
                    };
                    let indices: Vec<u16> = indices.iter().copied().map(|i| i as u16).collect();
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16)
                }
                ComponentType::U8 => {
                    let indices: Vec<u16> =
                        indices.indices.iter().copied().map(u16::from).collect();
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16)
                }
                ComponentType::I16 => {
                    assert_eq!(indices.indices.len() % std::mem::size_of::<i16>(), 0);
                    let indices: &[i16] = unsafe {
                        std::slice::from_raw_parts(
                            indices.indices.as_ptr() as _,
                            indices.indices.len() / std::mem::size_of::<i16>(),
                        )
                    };
                    let indices: Vec<u16> = indices.iter().copied().map(|i| i as u16).collect();
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16)
                }
                ComponentType::U16 => {
                    assert_eq!(indices.indices.len() % std::mem::size_of::<u16>(), 0);
                    let indices: &[u16] = unsafe {
                        std::slice::from_raw_parts(
                            indices.indices.as_ptr() as _,
                            indices.indices.len() / std::mem::size_of::<u16>(),
                        )
                    };
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT16)
                }
                ComponentType::U32 => {
                    assert_eq!(indices.indices.len() % std::mem::size_of::<u32>(), 0);
                    let indices: &[u32] = unsafe {
                        std::slice::from_raw_parts(
                            indices.indices.as_ptr() as _,
                            indices.indices.len() / std::mem::size_of::<u32>(),
                        )
                    };
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT32)
                }
                ComponentType::F32 => {
                    assert_eq!(indices.indices.len() % std::mem::size_of::<f32>(), 0);
                    let indices: &[f32] = unsafe {
                        std::slice::from_raw_parts(
                            indices.indices.as_ptr() as _,
                            indices.indices.len() / std::mem::size_of::<f32>(),
                        )
                    };
                    let indices: Vec<u32> = indices.iter().copied().map(|i| i as u32).collect();
                    ret.set_indices(indices.as_bytes(), vk::IndexType::UINT32)
                }
            }
        }

        ret
    }
}
