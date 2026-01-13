//! Event handling.

/// Platform events.
#[derive(Debug, Clone)]
pub enum Event {
    /// Window was resized.
    Resized { width: u32, height: u32 },
    /// Window close was requested.
    CloseRequested,
    /// Window gained or lost focus.
    Focused(bool),
    /// A redraw was requested.
    RedrawRequested,
}

/// The event loop.
pub struct EventLoop {
    inner: winit::event_loop::EventLoop<()>,
}

impl EventLoop {
    /// Creates a new event loop.
    pub fn new() -> Result<Self, winit::error::EventLoopError> {
        Ok(Self {
            inner: winit::event_loop::EventLoop::new()?,
        })
    }

    /// Returns the inner winit event loop.
    pub fn inner(&self) -> &winit::event_loop::EventLoop<()> {
        &self.inner
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create event loop")
    }
}
