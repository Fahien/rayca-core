// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crate::*;

pub struct Queue {
    pub queue: vk::Queue,
    device: Arc<ash::Device>,
}

impl Queue {
    pub fn new(device: &Device) -> Self {
        let queue = unsafe { device.get_device_queue(device.graphics_queue_index, 0) };
        Queue {
            queue,
            device: device.device.clone(),
        }
    }

    pub fn get_handle(&self) -> vk::Queue {
        self.queue
    }

    pub fn submit(&self, submits: &[vk::SubmitInfo], fence: Option<&mut Fence>) {
        let fence = match fence {
            Some(fence) => {
                fence.can_wait = true;
                fence.fence
            }
            None => vk::Fence::null(),
        };

        unsafe { self.device.queue_submit(self.queue, submits, fence) }
            .expect("Failed to submit to Vulkan queue")
    }

    pub fn submit_draw(
        &self,
        command_buffer: &CommandBuffer,
        wait: &Semaphore,
        signal: &Semaphore,
        fence: Option<&mut Fence>,
    ) {
        // Wait for the image to be available ..
        let waits = [wait.semaphore];
        // .. at color attachment output stage
        let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [command_buffer.command_buffer];
        let signals = [signal.semaphore];

        let submits = [vk::SubmitInfo::default()
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signals)];

        self.submit(&submits, fence);
    }

    pub fn present(
        &self,
        image_index: u32,
        swapchain: &Swapchain,
        wait: &Semaphore,
    ) -> Result<(), vk::Result> {
        let pres_image_indices = [image_index];
        let pres_swapchains = [swapchain.swapchain];
        let pres_semaphores = [wait.semaphore];
        let present_info = vk::PresentInfoKHR::default()
            .image_indices(&pres_image_indices)
            .swapchains(&pres_swapchains)
            .wait_semaphores(&pres_semaphores);

        let ret = unsafe { swapchain.ext.queue_present(self.queue, &present_info) };

        match ret {
            Ok(false) => Ok(()),
            // Suboptimal
            Ok(true) => Err(vk::Result::ERROR_OUT_OF_DATE_KHR),
            Err(result) => Err(result),
        }
    }

    pub fn submit_and_wait(&self, command_buffer: &CommandBuffer) {
        let command_buffers = [command_buffer.command_buffer];
        let mut fence = Fence::unsignaled(&self.device);

        let submits = [vk::SubmitInfo::default().command_buffers(&command_buffers)];

        self.submit(&submits, Some(&mut fence));
        fence.wait();
    }
}

pub struct GraphicsQueue {
    pub command_pool: CommandPool,
    pub queue: Queue,
}

impl GraphicsQueue {
    pub fn new(device: &Device) -> Self {
        Self {
            queue: Queue::new(device),
            command_pool: CommandPool::new(device),
        }
    }
}

impl std::ops::Deref for GraphicsQueue {
    type Target = Queue;
    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl std::ops::DerefMut for GraphicsQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}
