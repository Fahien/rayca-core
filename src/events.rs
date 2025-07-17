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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidKeyCode {
    Unknown = 0x0,
    Back = 0x4,
    A = 0x60,
    B = 0x61,
    X = 0x63,
    Y = 0x64,
    L1 = 0x00000066,
    R1 = 0x00000067,
    L2 = 0x00000068,
    R2 = 0x00000069,
    L3 = 0x6A,
    R3 = 0x6B,
    Play = 0x6C,
    Stop = 0x6D,
}

impl From<u32> for AndroidKeyCode {
    fn from(code: u32) -> Self {
        match code {
            0x4 => AndroidKeyCode::Back,
            0x60 => AndroidKeyCode::A,
            0x61 => AndroidKeyCode::B,
            0x63 => AndroidKeyCode::X,
            0x64 => AndroidKeyCode::Y,
            0x00000066 => AndroidKeyCode::L1,
            0x00000067 => AndroidKeyCode::R1,
            0x00000068 => AndroidKeyCode::L2,
            0x00000069 => AndroidKeyCode::R2,
            0x6A => AndroidKeyCode::L3,
            0x6B => AndroidKeyCode::R3,
            0x6C => AndroidKeyCode::Play,
            0x6D => AndroidKeyCode::Stop,
            _ => {
                eprintln!("Unknown Android key code: {}", code);
                AndroidKeyCode::Unknown
            }
        }
    }
}

#[derive(Default)]
pub struct AndroidInput {
    pub left_axis: Vec2,
    pub back: ButtonState,
    pub a: ButtonState,
    pub b: ButtonState,
    pub x: ButtonState,
    pub y: ButtonState,
    pub l1: ButtonState,
    pub r1: ButtonState,
    pub l2: ButtonState,
    pub r2: ButtonState,
    pub l3: ButtonState,
    pub r3: ButtonState,
    pub play: ButtonState,
    pub stop: ButtonState,
}

#[derive(Default)]
pub struct Input {
    pub q: ButtonState,
    pub w: ButtonState,
    pub e: ButtonState,
    pub a: ButtonState,
    pub s: ButtonState,
    pub d: ButtonState,
    pub mouse: Mouse,

    pub android: AndroidInput,
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
