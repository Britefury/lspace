use gdk::ffi as gdk_ffi;

pub const BUTTON1: u16 =    0b0000000000000001;
pub const BUTTON2: u16 =    0b0000000000000010;
pub const BUTTON3: u16 =    0b0000000000000100;
pub const BUTTON4: u16 =    0b0000000000001000;
pub const BUTTON5: u16 =    0b0000000000010000;
const BUTTONS_MASK: u16 =   0b0000000000011111;

pub const KEY_CTRL: u16 =   0b0000000100000000;
pub const KEY_SHIFT: u16 =  0b0000001000000000;
pub const KEY_ALT: u16 =    0b0000010000000000;
pub const KEY_SUPER: u16 =  0b0000100000000000;
const KEYS_MASK: u16 =      0b0000111100000000;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InputModifierState {
    value: u16,
}

impl InputModifierState {
    pub fn new() -> InputModifierState {
        return InputModifierState{value: 0};
    }

    pub fn from_values(value: u16) -> InputModifierState {
        return InputModifierState{value: value};
    }

    pub fn from_buttons_and_keys(buttons: u8, keys: u8) -> InputModifierState {
        let value = (buttons as u16) | ((keys as u16) << 8);
        return InputModifierState{value: value};
    }


    pub fn buttons(&self) -> u8 {
        return (self.value & BUTTONS_MASK) as u8;
    }

    pub fn keys(&self) -> u8 {
        return ((self.value & KEYS_MASK) >> 8) as u8;
    }


    pub fn test(&self, mask: u16) -> bool {
        return (self.value & mask) != 0;
    }

    pub fn test_buttons(&self, mask: u8) -> bool {
        return (self.buttons() & mask) != 0;
    }

    pub fn test_keys(&self, mask: u8) -> bool {
        return (self.keys() & mask) != 0;
    }


    pub fn set_buttons(&mut self, mask: u8, value: u8) {
        let mut buttons = self.buttons();
        buttons = (value & mask) | (buttons & !mask);
        self.value = (buttons as u16) | (self.value & !BUTTONS_MASK);
    }

    pub fn set_keys(&mut self, mask: u8, value: u8) {
        let mut keys = self.keys();
        keys = (value & mask) | (keys & !mask);
        self.value = ((keys as u16) << 8) | (self.value & !KEYS_MASK);
    }

    pub fn set_flag(&mut self, mask: u16, value: bool) {
        if value {
            self.value = self.value | mask;
        } else {
            self.value = self.value & !mask;
        }
    }

    pub fn update_from_gdk_mod(&mut self, gdk_state: gdk_ffi::GdkModifierType) {
        self.set_flag(BUTTON1, gdk_state.contains(gdk_ffi::GDK_BUTTON1_MASK));
        self.set_flag(BUTTON2, gdk_state.contains(gdk_ffi::GDK_BUTTON2_MASK));
        self.set_flag(BUTTON3, gdk_state.contains(gdk_ffi::GDK_BUTTON3_MASK));
        self.set_flag(BUTTON4, gdk_state.contains(gdk_ffi::GDK_BUTTON4_MASK));
        self.set_flag(BUTTON5, gdk_state.contains(gdk_ffi::GDK_BUTTON5_MASK));

        self.set_flag(KEY_CTRL, gdk_state.contains(gdk_ffi::GDK_CONTROL_MASK));
        self.set_flag(KEY_SHIFT, gdk_state.contains(gdk_ffi::GDK_SHIFT_MASK));
        self.set_flag(KEY_ALT, gdk_state.contains(gdk_ffi::GDK_MOD1_MASK));
        self.set_flag(KEY_SUPER, gdk_state.contains(gdk_ffi::GDK_SUPER_MASK));
    }


    pub fn button1(&self) -> bool {
        return self.test(BUTTON1);
    }

    pub fn button2(&self) -> bool {
        return self.test(BUTTON2);
    }

    pub fn button3(&self) -> bool {
        return self.test(BUTTON3);
    }

    pub fn button4(&self) -> bool {
        return self.test(BUTTON4);
    }

    pub fn button5(&self) -> bool {
        return self.test(BUTTON5);
    }

    pub fn button(&self, button_index: u32) -> bool {
        assert!(button_index > 0 && button_index < 8);
        return self.test(BUTTON1 << (button_index - 1));
    }

    pub fn control_key(&self) -> bool {
        return self.test(KEY_CTRL);
    }

    pub fn shift_key(&self) -> bool {
        return self.test(KEY_SHIFT);
    }

    pub fn alt_key(&self) -> bool {
        return self.test(KEY_ALT);
    }

    pub fn super_key(&self) -> bool {
        return self.test(KEY_SUPER);
    }
}
