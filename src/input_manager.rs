///
/// Stores the input state.
/// 

use std::collections;

#[derive(Copy, Clone)]
pub enum ButtonCode {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    A,
    B
}

#[derive(PartialEq, Copy, Clone)]
enum ButtonState {
    PRESSED,
    DOWN,
    RELEASED,
    UP,
}

pub struct InputManager {
    button_states: [ButtonState; 6],
    key_button_map: collections::HashMap<winit::event::VirtualKeyCode, ButtonCode>
}

impl InputManager {
    /// Creates a new instance of the input manager.
    pub fn new() -> Self {
        Self {
            button_states: [ButtonState::UP; 6],
            key_button_map: collections::HashMap::new()
        }
    }

    /// Returns true if button is down.
    pub fn is_button_down(&self, button_code: ButtonCode) -> bool {
        return match self.button_states[button_code as usize] {
            ButtonState::DOWN => true,
            ButtonState::PRESSED => true,
            _ => false,
        }
    }

    /// Returns true if button has been pressed this frame.
    pub fn is_button_pressed(&self, button_code: ButtonCode) -> bool {
        return self.button_states[button_code as usize] == ButtonState::PRESSED;
    }

    /// Sets a button as released this frame.
    pub fn set_button_released(&mut self, button_code: ButtonCode) {
        self.button_states[button_code as usize] = ButtonState::RELEASED;
    }

    /// Sets a button as pressed this frame.
    pub fn set_button_pressed(&mut self, button_code: ButtonCode) {
        if self.button_states[button_code as usize] != ButtonState::DOWN  {
            self.button_states[button_code as usize] = ButtonState::PRESSED;
        }
    }

    /// Updates the general state of the input manager.
    /// Should be run once per frame.
    pub fn update(&mut self) {
        for button_state in &mut self.button_states {
            *button_state = match button_state {
                ButtonState::PRESSED => ButtonState::DOWN,
                ButtonState::RELEASED => ButtonState::UP,
                _ => *button_state
            }
        };
    }

    /// Converts a key to a ButtonCode.
    /// If key is not a button, returns None.
    pub fn key_to_button(&self, key: winit::event::VirtualKeyCode) -> Option<ButtonCode> {
        if self.key_button_map.contains_key(&key) {
            return Some(self.key_button_map[&key]);
        }
        None
    }

    /// Sets up a mapping between a key and a button.
    pub fn map_key_to_button(&mut self, key: winit::event::VirtualKeyCode, button_code: ButtonCode) {
        self.key_button_map.insert(key, button_code);
    }
}