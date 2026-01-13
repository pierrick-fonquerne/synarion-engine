//! Gamepad/controller input handling.

/// Gamepad buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// South face button (A on Xbox, X on PlayStation).
    South,
    /// East face button (B on Xbox, Circle on PlayStation).
    East,
    /// West face button (X on Xbox, Square on PlayStation).
    West,
    /// North face button (Y on Xbox, Triangle on PlayStation).
    North,
    /// Left bumper.
    LeftBumper,
    /// Right bumper.
    RightBumper,
    /// Left stick press.
    LeftStick,
    /// Right stick press.
    RightStick,
    /// Start button.
    Start,
    /// Select/Back button.
    Select,
    /// D-pad directions.
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

/// Gamepad axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// Left stick X axis.
    LeftStickX,
    /// Left stick Y axis.
    LeftStickY,
    /// Right stick X axis.
    RightStickX,
    /// Right stick Y axis.
    RightStickY,
    /// Left trigger.
    LeftTrigger,
    /// Right trigger.
    RightTrigger,
}
