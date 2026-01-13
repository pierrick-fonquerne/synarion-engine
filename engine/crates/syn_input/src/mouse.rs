//! Mouse input handling.

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Additional buttons.
    Other(u16),
}

/// Mouse input state.
#[derive(Debug, Clone, Default)]
pub struct MouseState {
    /// Current position.
    pub position: (f32, f32),
    /// Delta movement since last frame.
    pub delta: (f32, f32),
    /// Scroll wheel delta.
    pub scroll_delta: f32,
}
