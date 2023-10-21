
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayState {
    Draw,
    Clear,
    Noop,
}
pub struct Display {
    pub buffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub state: DisplayState,
}

impl Display {

    pub fn new() -> Self {
        Display {
            buffer: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            state: DisplayState::Noop,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize) {
        self.state = DisplayState::Draw;
        let pixel = &mut self.buffer[y][x];
        *pixel ^= 1;
    }

    pub fn get_color_buffer(&mut self) -> Vec<u8> {
        let mut color_buffer = Vec::new();
    
        for row in &self.buffer {
            for pixel in row {
                if *pixel == 0 {
                    color_buffer.extend_from_slice(&[0, 0, 0, 255]);
                } else {
                    color_buffer.extend_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    
        color_buffer
    }
}
