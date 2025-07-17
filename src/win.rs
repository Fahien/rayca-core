// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, NativeKeyCode, PhysicalKey},
    window::{Window, WindowId, WindowLevel},
};

pub struct WinBuilder {
    title: String,
    size: Size2,
    #[cfg(target_os = "android")]
    app: Option<AndroidApp>,
}

impl Default for WinBuilder {
    fn default() -> Self {
        Self {
            title: "Rayca".into(),
            size: Size2::new(480, 480),
            #[cfg(target_os = "android")]
            app: None,
        }
    }
}

impl WinBuilder {
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, size: Size2) -> Self {
        self.size = size;
        self
    }

    #[cfg(target_os = "android")]
    pub fn android_app(mut self, android_app: AndroidApp) -> Self {
        // This is a placeholder for Android-specific initialization
        // In a real application, you would set up the Android app here
        self.app = Some(android_app);
        self
    }

    #[cfg(not(target_os = "android"))]
    pub fn build(self) -> Win {
        Win::new(self.title, self.size)
    }

    #[cfg(target_os = "android")]
    pub fn build(self) -> Win {
        Win::new(
            self.title,
            self.size,
            self.app.expect("Android app must be set"),
        )
    }
}

pub struct Win {
    pub name: String,

    #[cfg(target_os = "android")]
    pub android_app: AndroidApp,

    pub size: Size2,

    window_id: Option<WindowId>,
    pub window: Option<Window>,

    resized: bool,
    pub exit: bool,

    pub input: Input,
}

impl Default for Win {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Win {
    pub fn builder() -> WinBuilder {
        WinBuilder::default()
    }

    #[cfg(not(target_os = "android"))]
    pub fn new<S: Into<String>>(name: S, size: Size2) -> Self {
        Self {
            name: name.into(),
            size,
            window_id: None,
            window: None,
            resized: false,
            exit: false,
            input: Input::default(),
        }
    }

    #[cfg(target_os = "android")]
    pub fn new<S: Into<String>>(name: S, size: Size2, android_app: AndroidApp) -> Self {
        Self {
            name: name.into(),
            android_app,
            size,
            window_id: None,
            window: None,
            resized: false,
            exit: false,
            input: Input::default(),
        }
    }

    /// Returns the flag and resets it to `false`
    pub fn is_resized(&mut self) -> bool {
        let ret = self.resized;
        self.resized = false;
        ret
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        if let Some(window) = &self.window {
            let inner_size = window.inner_size();
            PhysicalSize::new(inner_size.width, inner_size.height)
        } else {
            PhysicalSize::default()
        }
    }
}

impl ApplicationHandler for Win {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::Destroyed && self.window_id == Some(window_id) {
            self.window_id = None;
            event_loop.exit();
            return;
        }

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => match physical_key {
                PhysicalKey::Code(KeyCode::KeyQ) => match state {
                    ElementState::Pressed => self.input.q = ButtonState::JustPressed,
                    ElementState::Released => self.input.q = ButtonState::JustReleased,
                },
                PhysicalKey::Code(KeyCode::KeyW) => match state {
                    ElementState::Pressed => self.input.w = ButtonState::JustPressed,
                    ElementState::Released => self.input.w = ButtonState::JustReleased,
                },
                PhysicalKey::Code(KeyCode::KeyE) => match state {
                    ElementState::Pressed => self.input.e = ButtonState::JustPressed,
                    ElementState::Released => self.input.e = ButtonState::JustReleased,
                },
                PhysicalKey::Code(KeyCode::KeyA) => match state {
                    ElementState::Pressed => self.input.a = ButtonState::JustPressed,
                    ElementState::Released => self.input.a = ButtonState::JustReleased,
                },
                PhysicalKey::Code(KeyCode::KeyS) => match state {
                    ElementState::Pressed => self.input.s = ButtonState::JustPressed,
                    ElementState::Released => self.input.s = ButtonState::JustReleased,
                },
                PhysicalKey::Code(KeyCode::KeyD) => match state {
                    ElementState::Pressed => self.input.d = ButtonState::JustPressed,
                    ElementState::Released => self.input.d = ButtonState::JustReleased,
                },
                PhysicalKey::Unidentified(NativeKeyCode::Android(code)) => {
                    let button_state = match state {
                        ElementState::Pressed => ButtonState::JustPressed,
                        ElementState::Released => ButtonState::JustReleased,
                    };
                    match AndroidKeyCode::from(code) {
                        AndroidKeyCode::Back => self.input.android.back = button_state,
                        AndroidKeyCode::A => self.input.android.a = button_state,
                        AndroidKeyCode::B => self.input.android.b = button_state,
                        AndroidKeyCode::X => self.input.android.x = button_state,
                        AndroidKeyCode::Y => self.input.android.y = button_state,
                        AndroidKeyCode::L1 => self.input.android.l1 = button_state,
                        AndroidKeyCode::R1 => self.input.android.r1 = button_state,
                        AndroidKeyCode::L2 => self.input.android.l2 = button_state,
                        AndroidKeyCode::R2 => self.input.android.r2 = button_state,
                        AndroidKeyCode::L3 => self.input.android.l3 = button_state,
                        AndroidKeyCode::R3 => self.input.android.r3 = button_state,
                        AndroidKeyCode::Play => self.input.android.play = button_state,
                        AndroidKeyCode::Stop => self.input.android.stop = button_state,
                        _ => (),
                    }
                }
                _ => println!("Unhandled key event: {:?}", physical_key),
            },
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => self.input.mouse.left = ButtonState::JustPressed,
                ElementState::Released => self.input.mouse.left = ButtonState::JustReleased,
            },
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => match state {
                ElementState::Pressed => self.input.mouse.right = ButtonState::JustPressed,
                ElementState::Released => self.input.mouse.right = ButtonState::JustReleased,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse.movement.x = position.x as f32 - self.input.mouse.position.x;
                self.input.mouse.movement.y = position.y as f32 - self.input.mouse.position.y;
                self.input.mouse.position.x = position.x as f32;
                self.input.mouse.position.y = position.y as f32;
            }
            WindowEvent::Touch(Touch { location, .. }) => {
                self.input.android.left_axis.x = location.x as f32;
                self.input.android.left_axis.y = location.y as f32;
            }
            WindowEvent::CloseRequested => {
                self.window = None;
                self.exit = true;
            }
            WindowEvent::RedrawRequested => {}
            WindowEvent::Resized(physical_size) => {
                self.resized = true;
                self.size.width = physical_size.width;
                self.size.height = physical_size.height;
            }
            _ => println!("Unhandled window event: {:?}", event),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(self.name.clone())
                .with_window_level(WindowLevel::AlwaysOnTop)
                .with_inner_size(PhysicalSize::new(self.size.width, self.size.height));
            let window = event_loop
                .create_window(attrs)
                .expect("Failed to create window");
            self.window_id = Some(window.id());
            self.window = Some(window);
        }
    }
}
