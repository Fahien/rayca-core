// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{ffi::CString, path::Path, rc::Rc};

use ash::vk;

use crate::*;

pub struct ShaderModule {
    pub shader: vk::ShaderModule,
    pub stage: vk::ShaderStageFlags,
    pub device: Rc<ash::Device>,
}

impl ShaderModule {
    pub fn new(dev: &Dev, shader_module: vk::ShaderModule, stage: vk::ShaderStageFlags) -> Self {
        Self {
            shader: shader_module,
            stage,
            device: dev.device.clone(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(
        dev: &Dev,
        shader_path: P,
        stage: vk::ShaderStageFlags,
    ) -> Self {
        #[cfg(not(target_os = "android"))]
        let shader_data = Self::build_slang_shader(shader_path);

        #[cfg(target_os = "android")]
        let shader_data = std::fs::read(shader_path).expect("Failed to read shader file");

        Self::from_data(dev, &shader_data, stage)
    }

    pub fn from_data(dev: &Dev, shader_data: &[u8], stage: vk::ShaderStageFlags) -> Self {
        Self::new(dev, Self::build_shader_module(dev, shader_data), stage)
    }

    #[cfg(not(target_os = "android"))]
    fn build_slang_shader<P: AsRef<Path>>(shader_path: P) -> Vec<u8> {
        let global_session = slang::GlobalSession::new().unwrap();

        let targets = [slang::TargetDesc::default()
            .format(slang::CompileTarget::Spirv)
            .profile(global_session.find_profile("sm_6_5"))];

        let search_path = std::ffi::CString::new(".").unwrap();
        let search_paths = [search_path.as_ptr()];

        // All compiler options are available through this builder.
        let session_options = slang::CompilerOptions::default()
            .optimization(slang::OptimizationLevel::High)
            .matrix_layout_row(true);

        let session_desc = slang::SessionDesc::default()
            .targets(&targets)
            .search_paths(&search_paths)
            .options(&session_options);

        let session = global_session.create_session(&session_desc).unwrap();

        let module = session
            .load_module(&shader_path.as_ref().to_str().unwrap())
            .unwrap();

        let entry_point = module.find_entry_point_by_name("main").unwrap();

        use slang::Downcast;
        let program = session
            .create_composite_component_type(&[
                module.downcast().clone(),
                entry_point.downcast().clone(),
            ])
            .expect("Failed to create program");

        let linked_program = program.link().expect("Failed to link program");

        let shader_blob = linked_program
            .entry_point_code(0, 0)
            .expect("Failed to get entry point code");

        shader_blob.as_slice().to_vec()
    }

    fn build_shader_module(dev: &Dev, shader_data: &[u8]) -> vk::ShaderModule {
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
        unsafe { dev.device.create_shader_module(&create_info, None) }
            .expect("Failed to create Vulkan shader module")
    }

    /// The entrypoint c string should stay alive until the pipeline has been created
    pub fn get_stage<'a>(&self, entrypoint: &'a CString) -> vk::PipelineShaderStageCreateInfo<'a> {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(self.stage)
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
