use input::pointer::Pointer;
use input::keyboard::Keyboard;
use input::inputmodifier::InputModifierState;

pub struct InputSystem {
    mod_state: InputModifierState,
    keyboard: Keyboard,
    mouse: Pointer
}

impl  InputSystem {
    pub fn new() -> InputSystem {
        let mod_state = InputModifierState::new();
        let s = InputSystem{mod_state: mod_state,
                            keyboard: Keyboard::new(),
                            mouse: Pointer::new()};
        return s;
    }

    pub fn mod_state<'a>(&'a self) -> &'a InputModifierState {
        return &self.mod_state;
    }

    pub fn mod_state_mut<'a>(&'a mut self) -> &'a mut InputModifierState {
        return &mut self.mod_state;
    }

    pub fn keyboard<'a>(&'a self) -> &'a Keyboard {
        return &self.keyboard;
    }

    pub fn keyboard_mut<'a>(&'a mut self) -> &'a mut Keyboard {
        return &mut self.keyboard;
    }

    pub fn mouse<'a>(&'a self) -> &'a Pointer {
        return &self.mouse;
    }

    pub fn mouse_mut<'a>(&'a mut self) -> &'a mut Pointer {
        return &mut self.mouse;
    }
}
