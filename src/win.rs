// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

pub struct WinBuilder {
    title: String,
    size: PhysicalSize<u32>,
    #[cfg(target_os = "android")]
    app: Option<AndroidApp>,
}

impl Default for WinBuilder {
    fn default() -> Self {
        Self {
            title: "Rayca".into(),
            size: PhysicalSize::new(480, 480),
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

    pub fn size(mut self, size: PhysicalSize<u32>) -> Self {
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

    pub size: PhysicalSize<u32>,

    window_id: Option<WindowId>,
    pub window: Option<Window>,

    pub resized: bool,
    pub exit: bool,
}

impl Win {
    pub fn builder() -> WinBuilder {
        WinBuilder::default()
    }

    #[cfg(not(target_os = "android"))]
    pub fn new<S: Into<String>>(name: S, size: PhysicalSize<u32>) -> Self {
        Self {
            name: name.into(),
            size,
            window_id: None,
            window: None,
            resized: false,
            exit: false,
        }
    }

    #[cfg(target_os = "android")]
    pub fn new<S: Into<String>>(name: S, size: PhysicalSize<u32>, android_app: AndroidApp) -> Self {
        Self {
            name: name.into(),
            android_app,
            size,
            window_id: None,
            window: None,
            resized: false,
            exit: false,
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
            WindowEvent::CloseRequested => {
                self.window = None;
                self.exit = true;
            }
            WindowEvent::RedrawRequested => {}
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(self.name.clone())
            .with_inner_size(PhysicalSize::new(self.size.width, self.size.height));
        let window = event_loop
            .create_window(attrs)
            .expect("Failed to create window");
        self.window_id = Some(window.id());
        self.window = Some(window);
    }
}
