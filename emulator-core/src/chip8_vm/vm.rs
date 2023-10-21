use core::panic;

use super::display::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::keyboard::Keypad;
use super::stack::Stack;
use crate::resources::FONTSET;


use crate::chip8_vm::timers::Timers;

const MEMORY_SIZE: usize = 4096;
const FONTSET_SIZE: usize = 80;
const NUM_REGISTERS: usize = 16;
const PROGRAM_START_ADDRESS: usize = 0x200;

struct Registers {
    v: [u8; NUM_REGISTERS], // general purpose registers
    i: u16,                 // index register
    pc: u16,                // program counter
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum CpuState {
    Running,
    Halted,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ProgramCounterState {
    Next,
    Skip,
    Jump(u16),
    Unknown,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Instructions {
    ClearScreen = 0x0000,
    Return = 0x000E,
    Jump = 0x1000,
    Call = 0x2000,
    SkipIfEqual = 0x3000,
    SkipIfNotEqual = 0x4000,
    SkipIfVxEqualVy = 0x5000,
    SetVx = 0x6000,
    AddVx = 0x7000,
    SetVxVy = 0x8000,
    SetVxOrVy = 0x8001,
    SetVxAndVy = 0x8002,
    SetVxXorVy = 0x8003,
    AddVxVy = 0x8004,
    SubVxVy = 0x8005,
    ShiftRight = 0x8006,
    SubVyVx = 0x8007,
    ShiftLeft = 0x800E,
    SkipIfVxNotVy = 0x9000,
    SetI = 0xA000,
    JumpV0 = 0xB000,
    Random = 0xC000,
    Draw = 0xD000,
    SkipIfPressed = 0xE09E,
    SkipIfNotPressed = 0xE0A1,
    SetVxToDelayTimer = 0xF007,
    WaitForKeyPress = 0xF00A,
    SetDelayTimer = 0xF015,
    SetSoundTimer = 0xF018,
    AddVxToI = 0xF01E,
    SetIToSprite = 0xF029,
    StoreBCD = 0xF033,
    StoreRegisters = 0xF055,
    LoadRegisters = 0xF065,
    Unknown = 0xFFFF,
}

pub struct Chip8VM {
    ram: [u8; MEMORY_SIZE],
    stack: Stack,
    registers: Registers,
    pub keypad: Keypad,
    pub timers: Timers,
    pub display: Display,
    state: CpuState,
    program_counter_state: ProgramCounterState,
    pub current_instruction: Instructions,
}

impl Chip8VM {
    pub fn new() -> Self {
        Chip8VM {
            ram: [0; MEMORY_SIZE],
            stack: Stack::new(),
            registers: Registers {
                v: [0; NUM_REGISTERS],
                i: 0,
                pc: PROGRAM_START_ADDRESS as u16,
            },
            timers: Timers::new(),
            keypad: Keypad::new(),
            display: Display::new(),
            state: CpuState::Halted,
            program_counter_state: ProgramCounterState::Unknown,
            current_instruction: Instructions::Unknown,
        }
    }
    pub fn reset(&mut self) {
        self.ram = [0; MEMORY_SIZE];
        self.stack.reset();
        self.registers = Registers {
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: 0x200,
        };
        self.timers.reset();
        self.keypad.reset();
        self.display.clear();
        self.state = CpuState::Halted;
        self.program_counter_state = ProgramCounterState::Next;
        self.current_instruction = Instructions::Unknown;
    }
    pub fn init_fontset(&mut self) {
        for i in 0..FONTSET_SIZE {
            self.ram[i] = FONTSET[i];
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.ram[i + PROGRAM_START_ADDRESS] = *byte
        }
    }

    pub fn cycle(&mut self) {
        self.state = CpuState::Running;
        let opcode = self.fetch();
        if self.execute(opcode) == CpuState::Halted {
            panic!("Unknown opcode: {:X?}", opcode);
        }
        match self.program_counter_state {
            ProgramCounterState::Next => self.next_instruction(),
            ProgramCounterState::Skip => self.skip_next_instruction(),
            ProgramCounterState::Jump(addr) => self.jump_next_instruction(addr),
            _ => (),
        }
    }

    fn next_instruction(&mut self) {
        self.registers.pc += 2;
    }

    fn skip_next_instruction(&mut self) {
        self.registers.pc += 4;
    }

    fn jump_next_instruction(&mut self, addr: u16) {
        self.registers.pc = addr;
    }

    fn fetch(&mut self) -> u16 {
        let pc = self.registers.pc as usize;
        let hi = self.ram[pc] as u16;
        let lo = self.ram[pc + 1] as u16;

        (hi<< 8) | lo
    }

    fn execute(&mut self, opcode: u16) -> CpuState {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let nn = (opcode & 0x00FF) as u16;
        let n = (opcode & 0x000F) as u16;
        let jump_addr = opcode & 0x0FFF;
        self.program_counter_state = ProgramCounterState::Next;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => {
                    self.display.clear();
                    self.current_instruction = Instructions::Draw;
                }
                0x000E => {
                    self.program_counter_state = ProgramCounterState::Jump(self.stack.pop());
                    self.current_instruction = Instructions::Return;
                }
                _ => {
                    self.state = CpuState::Halted;
                    self.current_instruction = Instructions::Unknown;
                }
            },
            0x1000 => {
                self.program_counter_state = ProgramCounterState::Jump(jump_addr);
                self.current_instruction = Instructions::Jump;
            }
            0x2000 => {
                if self.stack.get() != 0 {
                    self.stack.increment_sp();
                }
                self.stack.set(self.registers.pc + 2);
                self.program_counter_state = ProgramCounterState::Jump(jump_addr);
                self.current_instruction = Instructions::Call;
            }
            0x3000 => {
                if self.registers.v[x] == nn as u8 {
                    self.program_counter_state = ProgramCounterState::Skip;
                }
                self.current_instruction = Instructions::SkipIfEqual;
            }
            0x4000 => {
                if self.registers.v[x] != nn as u8 {
                    self.program_counter_state = ProgramCounterState::Skip;
                }
                self.current_instruction = Instructions::SkipIfNotEqual;
            }
            0x5000 => {
                if self.registers.v[x] == self.registers.v[y] {
                    self.program_counter_state = ProgramCounterState::Skip;
                }
                self.current_instruction = Instructions::SkipIfVxEqualVy;
            }
            0x6000 => {
                self.registers.v[x] = nn as u8;
                self.current_instruction = Instructions::SetVx;
            }
            0x7000 => {
                self.registers.v[x] = self.registers.v[x].wrapping_add(nn as u8);
                self.current_instruction = Instructions::AddVx;
            }
            0x8000 => match opcode & 0x000F {
                0x0000 => {
                    self.registers.v[x] = self.registers.v[y];
                    self.current_instruction = Instructions::SetVxVy;
                }
                0x0001 => {
                    self.registers.v[x] |= self.registers.v[y];
                    self.current_instruction = Instructions::SetVxOrVy;
                }
                0x0002 => {
                    self.registers.v[x] &= self.registers.v[y];
                    self.current_instruction = Instructions::SetVxAndVy;
                }
                0x0003 => {
                    self.registers.v[x] ^= self.registers.v[y];
                    self.current_instruction = Instructions::SetVxXorVy;
                }
                0x0004 => {
                    let vy = self.registers.v[x] as u16 + self.registers.v[y] as u16;
                    self.registers.v[0xF] = (vy > 0xFF) as u8;
                    self.registers.v[x] = vy as u8;
                    self.current_instruction = Instructions::AddVxVy;
                }
                0x0005 => {
                    self.registers.v[0xF] = (self.registers.v[x] > self.registers.v[y]) as u8;
                    self.registers.v[x] = self.registers.v[x].wrapping_sub(self.registers.v[y]);
                    self.current_instruction = Instructions::SubVxVy;
                }
                0x0006 => {
                    self.registers.v[0xF] = (self.registers.v[x] % 2 == 1) as u8;
                    self.registers.v[x] = self.registers.v[x] >> 1;
                    self.current_instruction = Instructions::ShiftRight;
                }
                0x0007 => {
                    self.registers.v[0xF] = (self.registers.v[y] > self.registers.v[x]) as u8;
                    self.registers.v[x] = self.registers.v[y].wrapping_sub(self.registers.v[x]);
                    self.current_instruction = Instructions::SubVyVx;
                }
                0x000E => {
                    self.registers.v[0xF] = !(self.registers.v[x] < 0x80) as u8;
                    self.registers.v[x] <<= 1;
                    self.current_instruction = Instructions::ShiftLeft;
                }
                _ => {
                    self.state = CpuState::Halted;
                    self.current_instruction = Instructions::Unknown;
                    panic!("Unknown opcode: {:X?}", opcode);
                }
            },
            0x9000 => {
                if self.registers.v[x] != self.registers.v[y] {
                    self.program_counter_state = ProgramCounterState::Skip;
                }
                self.current_instruction = Instructions::SkipIfVxNotVy;
            }
            0xA000 => {
                self.registers.i = jump_addr;
                self.current_instruction = Instructions::SetI;
            }
            0xB000 => {
                self.program_counter_state =
                    ProgramCounterState::Jump(jump_addr + self.registers.v[0] as u16 );
                self.current_instruction = Instructions::JumpV0;
            }
            0xC000 => {
                self.registers.v[x] = (rand::random::<u8>()) & (opcode & 0x00FF) as u8;
                self.current_instruction = Instructions::Random;
            }

            0xD000 => {
                self.registers.v[0xF] = 0;
                self.current_instruction = Instructions::Draw;
                
                for yline in 0..n {
                    let pixel = self.ram[(self.registers.i + yline as u16) as usize];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let x_coord = (self.registers.v[x] as usize + xline) % SCREEN_WIDTH;
                            let y_coord =
                                (self.registers.v[y] as usize + yline as usize) % SCREEN_HEIGHT;
                            if self.display.buffer[y_coord][x_coord] == 1 {
                                // collision detected
                                self.registers.v[0xF] = 1;
                            }
                            self.display.draw_pixel(x_coord, y_coord);
                        }
                    }
                }
            }
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    if self.keypad.get_key(self.registers.v[x]) != 0 {
                        self.program_counter_state = ProgramCounterState::Skip;
                    }
                    self.current_instruction = Instructions::SkipIfPressed;
                }
                0x00A1 => {
                    if self.keypad.get_key(self.registers.v[x]) == 0 {
                        self.program_counter_state = ProgramCounterState::Skip;
                    }
                    self.current_instruction = Instructions::SkipIfNotPressed;
                }
                _ => {
                    self.state = CpuState::Halted;
                    self.current_instruction = Instructions::Unknown;
                }
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => {
                    self.registers.v[x] = self.timers.get_delay_timer();
                    self.current_instruction = Instructions::SetVxToDelayTimer;
                }
                0x000A => {
                    let mut key_pressed = false;
                    for i in 0..self.keypad.len() {
                        if self.keypad.get_key(i as u8) != 0 {
                            self.registers.v[x] = i as u8;
                            key_pressed = true;
                        }
                    }
                    if !key_pressed {
                        self.program_counter_state = ProgramCounterState::Jump(self.registers.pc - 2);
                    }
                    self.current_instruction = Instructions::WaitForKeyPress;
                }
                0x0015 => {
                    self.timers.set_delay_timer(self.registers.v[x]);
                    self.current_instruction = Instructions::SetDelayTimer;
                }
                0x0018 => {
                    self.timers.set_sound_timer(self.registers.v[x]);
                    self.current_instruction = Instructions::SetSoundTimer;
                }
                0x001E => {
                    self.registers.i = self.registers.i.wrapping_add(self.registers.v[x] as u16);
                    self.registers.v[0xF] = (self.registers.i > 0xFFF) as u8;
                    self.current_instruction = Instructions::AddVxToI;
                }
                0x0029 => {
                    self.registers.i = self.registers.v[x] as u16 * 5;
                    self.current_instruction = Instructions::SetIToSprite;
                }
                0x0033 => {
                    self.ram[self.registers.i as usize] = self.registers.v[x] / 100;
                    self.ram[self.registers.i as usize + 1] = (self.registers.v[x] / 10) % 10;
                    self.ram[self.registers.i as usize + 2] = (self.registers.v[x] % 100) % 10;
                    self.current_instruction = Instructions::StoreBCD;
                }
                0x0055 => {
                    for i in 0..x + 1 {
                        self.ram[self.registers.i as usize + i] = self.registers.v[i];
                    }
                    self.registers.i += x as u16 + 1;
                    self.current_instruction = Instructions::StoreRegisters;
                }
                0x0065 => {
                    for i in 0..x + 1 {
                        self.registers.v[i] = self.ram[self.registers.i as usize + i];
                    }
                    self.registers.i += x as u16 + 1;
                    self.current_instruction = Instructions::LoadRegisters;
                }
                _ => {
                    self.state = CpuState::Halted;
                    self.current_instruction = Instructions::Unknown;
                }
            },
            _ => {
                self.state = CpuState::Halted;
                self.current_instruction = Instructions::Unknown;
            }
        }
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8VM::new();
        let rom = vec![0x00, 0x01, 0x02, 0x03];
        chip8.load_rom(rom);
        assert_eq!(chip8.ram[0x200], 0x00);
        assert_eq!(chip8.ram[0x201], 0x01);
        assert_eq!(chip8.ram[0x202], 0x02);
        assert_eq!(chip8.ram[0x203], 0x03);
    }
    #[test]
    fn test_load_fontset() {
        let mut chip8 = Chip8VM::new();
        chip8.init_fontset();

        for (i, byte) in FONTSET.iter().enumerate() {
            assert_eq!(chip8.ram[i], *byte);
        }
    }

    #[test]
    fn test_read_opcode() {
        let mut chip8 = Chip8VM::new();
        let rom = vec![0x00, 0x01, 0x02, 0x03, 0x02, 0x03];
        chip8.load_rom(rom);
        assert_eq!(chip8.fetch(), 0x0001);
        chip8.registers.pc += 0x2;
        assert_eq!(chip8.fetch(), 0x0203);
        chip8.registers.pc += 0x2;
        assert_eq!(chip8.fetch(), 0x0203);
    }

    #[test]
    fn test_read_memory() {
        let mut chip8 = Chip8VM::new();
        let rom = vec![0x00, 0x01, 0x02, 0x03];
        chip8.load_rom(rom);
        let mut memory = [0; MEMORY_SIZE];
        memory[0x200] = 0x00;
        memory[0x201] = 0x01;
        memory[0x202] = 0x02;
        memory[0x203] = 0x03;
        assert_eq!(chip8.ram, memory);
    }
}
