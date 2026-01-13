//! Keyboard input handling.

/// Keyboard key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A key.
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    /// N key.
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    /// Number keys.
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    /// Function keys.
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    /// Special keys.
    Escape, Space, Enter, Tab, Backspace,
    /// Arrow keys.
    Left, Right, Up, Down,
    /// Modifier keys.
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt,
}

/// The state of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Key is not pressed.
    Released,
    /// Key was just pressed this frame.
    JustPressed,
    /// Key is being held down.
    Pressed,
    /// Key was just released this frame.
    JustReleased,
}
