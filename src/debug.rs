// Copyright © 2021-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    borrow::Cow,
    ffi::{CStr, c_void},
};

use ash::{ext, vk};

use crate::Ctx;

unsafe extern "system" fn vk_debug(
    _msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> u32 {
    if std::thread::panicking() {
        return vk::FALSE;
    }

    let callback_data = unsafe { &*callback_data };
    let message = if callback_data.p_message.is_null() {
        Cow::from("No message")
    } else {
        unsafe { CStr::from_ptr(callback_data.p_message).to_string_lossy() }
    };
    eprintln!("{:?}", message);
    vk::FALSE
}

pub struct Debug {
    loader: ext::debug_utils::Instance,
    callback: vk::DebugUtilsMessengerEXT,
}

impl Debug {
    pub fn new(ctx: &Ctx) -> Self {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
            )
            .pfn_user_callback(Some(vk_debug));

        let loader = { ext::debug_utils::Instance::new(&ctx.entry, &ctx.instance) };
        let callback = unsafe {
            loader
                .create_debug_utils_messenger(&debug_info, None)
                .expect("Failed to create Vulkan debug callback")
        };

        Self { loader, callback }
    }
}

impl Drop for Debug {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.callback, None);
        }
    }
}
