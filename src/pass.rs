// Copyright © 2021-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use ash::vk;

use crate::*;

pub struct Pass {
    pub render: vk::RenderPass,
    pub device: Rc<ash::Device>,
}

impl Pass {
    pub fn new(dev: &Dev) -> Self {
        // Render pass (swapchain surface format, device)
        let attachment = [vk::AttachmentDescription::default()
            .format(dev.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let attach_refs = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        // Just one subpass
        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attach_refs)];

        let present_dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let dependencies = [present_dependency];

        // Build the render pass
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        let render = unsafe { dev.device.create_render_pass(&create_info, None) }
            .expect("Failed to create Vulkan render pass");

        Self {
            render,
            device: dev.device.device.clone(),
        }
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render, None);
        }
    }
}
