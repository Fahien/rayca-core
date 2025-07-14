// Copyright © 2024-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::time::Duration;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::pump_events::EventLoopExtPumpEvents;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;

use crate::*;

#[derive(Default, PartialEq)]
pub enum ButtonState {
    #[default]
    Released,
    JustReleased,
    Pressed,
    JustPressed,
}

impl ButtonState {
    pub fn update(&mut self) {
        if *self == ButtonState::JustPressed {
            *self = ButtonState::Pressed
        } else if *self == ButtonState::JustReleased {
            *self = ButtonState::Released
        }
    }

    pub fn is_down(&self) -> bool {
        *self == ButtonState::Pressed || *self == ButtonState::JustPressed
    }

    pub fn just_updated(&self) -> bool {
        *self == ButtonState::JustPressed || *self == ButtonState::JustReleased
    }

    pub fn press(&mut self) {
        if *self == ButtonState::JustPressed {
            *self = ButtonState::Pressed
        } else {
            *self = ButtonState::JustPressed
        }
    }

    pub fn release(&mut self) {
        if *self == ButtonState::JustReleased {
            *self = ButtonState::Released
        } else {
            *self = ButtonState::JustReleased
        }
    }
}

#[derive(Default)]
pub struct Mouse {
    pub position: Vec2,
    pub movement: Vec2,
    pub left: ButtonState,
    pub right: ButtonState,
}

impl Mouse {
    pub fn update(&mut self) {
        self.movement = Vec2::ZERO;
        self.left.update();
        self.right.update();
    }
}

#[derive(Default)]
pub struct Input {
    pub w: ButtonState,
    pub a: ButtonState,
    pub s: ButtonState,
    pub d: ButtonState,
    pub mouse: Mouse,
}

impl Input {
    pub fn update(&mut self) {
        self.mouse.update();
    }
}

pub struct Events {
    pub event_loop: EventLoop<()>,
}

impl Events {
    pub fn new(win: &mut Win) -> Self {
        let mut event_loop_builder = EventLoop::builder();

        #[cfg(target_os = "android")]
        use winit::platform::android::EventLoopBuilderExtAndroid;
        #[cfg(target_os = "android")]
        event_loop_builder.with_android_app(win.android_app.clone());

        let event_loop = event_loop_builder
            .build()
            .expect("Failed to create event loop");

        // Set the control flow to Poll to avoid blocking
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut ret = Self { event_loop };
        ret.update(win);
        ret
    }

    pub fn update(&mut self, win: &mut Win) {
        self.event_loop.pump_app_events(Some(Duration::ZERO), win);
    }
}
