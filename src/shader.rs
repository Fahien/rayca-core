// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{ffi::CString, path::Path, rc::Rc};

use ash::vk;

use crate::*;

pub struct ShaderModule {
    pub shader: vk::ShaderModule,
    pub device: Rc<ash::Device>,
}

impl ShaderModule {
    #[cfg(target_os = "android")]
    pub fn create_shaders(
        android_app: &AndroidApp,
        device: &Rc<ash::Device>,
        vert_path: &str,
        frag_path: &str,
    ) -> (Self, Self) {
        use std::{ffi::CString, str::FromStr};

        let vert_path = vert_path.replace(".slang", ".spv");
        let frag_path = frag_path.replace(".slang", ".spv");

        let c_vert_path =
            CString::from_str(&vert_path).expect("Failed to create CStr for vertex shader path");
        let c_frag_path =
            CString::from_str(&frag_path).expect("Failed to create CStr for fragment shader path");

        let msg = format!("Failed to open shader: {}", vert_path);
        let mut vert_asset = android_app
            .asset_manager()
            .open(c_vert_path.as_c_str())
            .expect(&msg);

        let msg = format!("Failed to open shader: {}", frag_path);
        let mut frag_asset = android_app
            .asset_manager()
            .open(c_frag_path.as_c_str())
            .expect(&msg);

        let vert_data = vert_asset
            .buffer()
            .expect("Failed to read vertex shader data");

        let frag_data = frag_asset
            .buffer()
            .expect("Failed to read fragment shader data");

        (
            Self::from_data(device, vert_data),
            Self::from_data(device, frag_data),
        )
    }

    #[cfg(not(target_os = "android"))]
    pub fn create_shaders(
        device: &Rc<ash::Device>,
        vert_path: &str,
        frag_path: &str,
    ) -> (Self, Self) {
        let vert_data = SlangProgram::get_entry_point_code(vert_path, "main").unwrap();
        let frag_data = SlangProgram::get_entry_point_code(frag_path, "main").unwrap();

        (
            Self::from_data(device, &vert_data),
            Self::from_data(device, &frag_data),
        )
    }

    pub fn new(device: &Rc<ash::Device>, shader_module: vk::ShaderModule) -> Self {
        Self {
            shader: shader_module,
            device: device.clone(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(device: &Rc<ash::Device>, shader_path: P) -> Self {
        let shader_data = std::fs::read(shader_path).expect("Failed to read shader file");
        Self::from_data(device, &shader_data)
    }

    pub fn from_data(device: &Rc<ash::Device>, shader_data: &[u8]) -> Self {
        Self::new(device, Self::build_shader_module(device, shader_data))
    }

    fn build_shader_module(device: &Rc<ash::Device>, shader_data: &[u8]) -> vk::ShaderModule {
        assert_eq!(shader_data.len() % 4, 0);
        let mut shader_bytecode = vec![0u32; shader_data.len() / size_of::<u32>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                shader_data.as_ptr(),
                shader_bytecode.as_mut_ptr() as _,
                shader_data.len(),
            );
        }

        let create_info = vk::ShaderModuleCreateInfo::default().code(&shader_bytecode);
        unsafe { device.create_shader_module(&create_info, None) }
            .expect("Failed to create Vulkan shader module")
    }

    /// The entrypoint c string should stay alive until the pipeline has been created
    pub fn get_stage<'a>(
        &self,
        entrypoint: &'a CString,
        stage: vk::ShaderStageFlags,
    ) -> vk::PipelineShaderStageCreateInfo<'a> {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(stage)
            .module(self.shader)
            .name(entrypoint)
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader, None);
        }
    }
}
