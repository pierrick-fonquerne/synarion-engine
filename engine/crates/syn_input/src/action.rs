//! Action mapping system.

use hashbrown::HashMap;
use crate::keyboard::KeyCode;
use crate::mouse::MouseButton;
use crate::gamepad::GamepadButton;

/// An input binding.
#[derive(Debug, Clone)]
pub enum InputBinding {
    /// Keyboard key.
    Key(KeyCode),
    /// Mouse button.
    Mouse(MouseButton),
    /// Gamepad button.
    Gamepad(GamepadButton),
}

/// A named action.
#[derive(Debug, Clone)]
pub struct Action {
    /// The action name.
    pub name: String,
    /// Input bindings for this action.
    pub bindings: Vec<InputBinding>,
}

/// Maps action names to their bindings.
#[derive(Debug, Default)]
pub struct ActionMap {
    actions: HashMap<String, Action>,
}

impl ActionMap {
    /// Creates a new empty action map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an action.
    pub fn register(&mut self, action: Action) {
        self.actions.insert(action.name.clone(), action);
    }

    /// Gets an action by name.
    pub fn get(&self, name: &str) -> Option<&Action> {
        self.actions.get(name)
    }
}
