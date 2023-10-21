use std::path::PathBuf;
use emulator_core::chip8_vm::keyboard::Keycode as Chip8Keycode;
use sdl2::keyboard::Keycode as SdlKeycode;
use rfd::FileDialog;

pub fn open_file_dialog() -> Option<PathBuf> {
    let file_dialog = FileDialog::new();
    file_dialog.add_filter("CHIP-8 ROMs", &["ch8"]).pick_file()
}

pub fn to_chip8_keycode(key: SdlKeycode) -> Chip8Keycode {
    match key {
        SdlKeycode::Num1 => Chip8Keycode::Num1,
        SdlKeycode::Num2 => Chip8Keycode::Num2,
        SdlKeycode::Num3 => Chip8Keycode::Num3,
        SdlKeycode::Num4 => Chip8Keycode::C,
        SdlKeycode::Q => Chip8Keycode::Num4,
        SdlKeycode::W => Chip8Keycode::Num5,
        SdlKeycode::E => Chip8Keycode::Num6,
        SdlKeycode::R => Chip8Keycode::D,
        SdlKeycode::A => Chip8Keycode::Num7,
        SdlKeycode::S => Chip8Keycode::Num8,
        SdlKeycode::D => Chip8Keycode::Num9,
        SdlKeycode::F => Chip8Keycode::E,
        SdlKeycode::Z => Chip8Keycode::A,
        SdlKeycode::X => Chip8Keycode::Num0,
        SdlKeycode::C => Chip8Keycode::B,
        SdlKeycode::V => Chip8Keycode::F,
        _ => Chip8Keycode::Num0,
    }
}