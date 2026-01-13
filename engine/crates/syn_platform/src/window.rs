//! Window creation and management.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Window configuration.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: String,
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether to enable VSync.
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Synarion Engine".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            vsync: true,
        }
    }
}

/// A platform window.
pub struct Window {
    inner: winit::window::Window,
}

impl Window {
    /// Returns the window's current size.
    pub fn size(&self) -> (u32, u32) {
        let size = self.inner.inner_size();
        (size.width, size.height)
    }

    /// Returns the inner winit window.
    pub fn inner(&self) -> &winit::window::Window {
        &self.inner
    }

    /// Requests a redraw.
    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.inner.window_handle()
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.inner.display_handle()
    }
}
