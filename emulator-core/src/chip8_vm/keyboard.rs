#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keypad {
    keypad: [u8; 16],
}

impl Keypad {
    pub fn new() -> Self {
        Keypad { keypad: [0; 16] }
    }

    pub fn reset(&mut self) {
        self.keypad = [0; 16];
    }

    pub fn set_key(&mut self, key: u8, value: u8) {
        self.keypad[key as usize] = value;
    }

    pub fn get_key(&self, key: u8) -> u8 {
        self.keypad[key as usize]
    }

    pub fn len(&self) -> usize {
        self.keypad.len()
    }

    pub fn handle_input(&mut self, key: Keycode, value: u8) {
        self.keypad[key as usize] = value;
    }
}

pub enum Keycode {
    Num1 = 0x1,
    Num2 = 0x2,
    Num3 = 0x3,
    Num4 = 0x4,
    Num5 = 0x5,
    Num6 = 0x6,
    Num7 = 0x7,
    Num8 = 0x8,
    Num9 = 0x9,
    A = 0xA,
    B = 0xB,
    C = 0xC,
    D = 0xD,
    E = 0xE,
    F = 0xF,
    Num0 = 0x00,
}
