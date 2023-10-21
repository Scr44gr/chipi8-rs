const STACK_SIZE: usize = 16;

pub struct Stack {
    stack: [u16; STACK_SIZE],
    sp: u8,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }

    pub fn pop(&mut self) -> u16 {
        let index = self.sp as usize;
        if self.sp > 0 {
            self.sp -= 1;
        }
        self.stack[index]
    }

    pub fn reset(&mut self) {
        self.sp = 0;
    }

    pub fn get(&self) -> u16 {
        self.stack[self.sp as usize]
    }

    pub fn set(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
    }

    pub fn increment_sp(&mut self) {
        self.sp += 1;
    }
}
