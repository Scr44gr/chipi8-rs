use std::path::PathBuf;
use std::vec;

use crate::chip8_vm::vm::Chip8VM;
pub mod chip8_vm;
mod resources;
pub const MAX_FRAMES_PER_SECOND: u32 = 60;

pub struct Rom {
    title: String,
    #[allow(dead_code)]
    path: String,
    size: u16,
    data: Vec<u8>,
}

impl Rom {
    pub fn new(path: PathBuf) -> Self {
        let rom_size = std::fs::metadata(path.clone()).unwrap().len() as u16;
        let rom_data = std::fs::read(path.clone()).unwrap();
        Rom {
            title: path.file_name().unwrap().to_str().unwrap().to_string(),
            path: path.to_str().unwrap().to_string(),
            size: rom_size,
            data: rom_data,
        }
    }

    pub fn get_title(&self) -> String {
        self.title.to_string()
    }
}

pub struct Emulator {
    pub chip8_vm: Chip8VM,
    pub current_room: Rom,
}

impl Emulator {
    pub fn new() -> Self {
        let mut chip8_vm: Chip8VM = Chip8VM::new();
        chip8_vm.init_fontset();

        Emulator {
            chip8_vm,
            current_room: Rom {
                title: String::new(),
                path: String::new(),
                size: u16::MIN,
                data: vec![],
            },
        }
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        self.current_room = Rom::new(path);
        self.stop_emulation();
        self.chip8_vm.load_rom(self.current_room.data.clone());
    }

    pub fn stop_emulation(&mut self) {
        self.chip8_vm.reset();
    }

    pub fn is_draw_flag_set(&mut self) -> bool {
        self.chip8_vm.display.state == chip8_vm::display::DisplayState::Draw
    }

    pub fn is_sound_flag_set(&mut self) -> bool {
        self.chip8_vm.timers.get_sound_timer() > 0
    }

    pub fn get_color_bufer(&mut self) -> Vec<u8> {
        self.chip8_vm.display.get_color_buffer()
    }

    pub fn handle_input(&mut self, key: u8, state: bool) {
        self.chip8_vm.keypad.set_key(key, state as u8)
    }

    pub fn emulate_cycles(&mut self, number_of_cycles: u8) {
        if self.current_room.size == 0 {
            return;
        }

        for _ in 0..number_of_cycles {
            self.chip8_vm.cycle();
        }
        self.chip8_vm.timers.update_timers();

    }
}
