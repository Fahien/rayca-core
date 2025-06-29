// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::Path;

pub struct SlangProgram {}

impl SlangProgram {
    pub fn get_entry_point_code<P: AsRef<Path>>(
        shader_path: P,
        entry_point_name: &str,
    ) -> Option<Vec<u8>> {
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

        let entry_point = module.find_entry_point_by_name(entry_point_name).unwrap();

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

        Some(shader_blob.as_slice().to_vec())
    }
}
