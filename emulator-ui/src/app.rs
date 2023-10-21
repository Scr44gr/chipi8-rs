use emulator_core::{Emulator, MAX_FRAMES_PER_SECOND};
use sdl2::video::Window;

use crate::utils;
use egui_backend::egui::{FullOutput, TextureId};
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{painter, sdl2::event::Event, DpiScaling, EguiStateHandler, ShaderVersion};
use egui_sdl2_gl as egui_backend;
use sdl2::video::SwapInterval;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use super::audio::AudioDriver;

const APP_TITLE: &str = "CHIPI-8 Emulator";
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 512;
const EMULATOR_CANVAS_SIZE: [f32; 2] = [400.0, 200.0];

pub struct GuiApp {
    emulator: Emulator,
    audio_device: AudioDriver,
    egui_ctx: egui::Context,
    window: Window,
    is_running: bool,
    event_pump: sdl2::EventPump,
    app_start_time: Instant,
}

impl GuiApp {
    pub fn new() -> GuiApp {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);

        let window = video_subsystem
            .window(APP_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
            .opengl()
            .resizable()
            .allow_highdpi()
            .build()
            .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();
        let emulator = Emulator::new();
        GuiApp {
            emulator: emulator,
            audio_device: AudioDriver::new(&sdl_context.audio().unwrap()),
            egui_ctx: egui::Context::default(),
            window,
            event_pump,
            is_running: true,
            app_start_time: Instant::now(),
        }
    }

    fn render_ui(
        &mut self,
        painter: &mut painter::Painter,
        egui_state: &mut EguiStateHandler,
        emulator_texture_id: TextureId,
    ) {
        egui_state.input.time = Some(self.app_start_time.elapsed().as_secs_f64());
        self.egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&self.egui_ctx, |ui| {
            // select the room
            ui.menu_button("File", |ui| {
                if ui.button("Load room").clicked() {
                    if let Some(path) = Self::get_game_path() {
                        self.emulator.load_rom(path);
                    }
                    ui.close_menu();
                }
            });
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.image(emulator_texture_id, EMULATOR_CANVAS_SIZE);
                ui.label(self.emulator.current_room.get_title());
            });
        });

        let FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = self.egui_ctx.end_frame();

        // Process ouput
        egui_state.process_output(&self.window, &platform_output);

        let paint_jobs = self.egui_ctx.tessellate(shapes);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        self.window.gl_swap_window();

        self.handle_input(egui_state, painter, repaint_after);
    }

    fn get_game_path() -> Option<PathBuf> {
        let path_buff = utils::open_file_dialog();
        path_buff
    }

    fn handle_input(
        &mut self,
        egui_state: &mut EguiStateHandler,
        painter: &mut painter::Painter,
        repaint_after: Duration,
    ) {
        if repaint_after.is_zero() {
            let event = self.event_pump.wait_event_timeout(5);
            if event.is_some() {
                match event {
                    Some(Event::Quit { .. }) => self.is_running = false,
                    Some(Event::KeyDown {
                        keycode: Some(key), ..
                    }) => {
                        let pressed_key = utils::to_chip8_keycode(key);
                        self.emulator.handle_input(pressed_key as u8, true);
                        println!("Key pressed: {:?}", key);
                    }
                    Some(Event::KeyUp {
                        keycode: Some(key), ..
                    }) => {
                        let unpressed_key = utils::to_chip8_keycode(key);
                        self.emulator.handle_input(unpressed_key as u8, false);
                    }
                    _ => {
                        // Process input event
                        egui_state.process_input(&self.window, event.unwrap(), painter);
                    }
                }
            }
        }

        let events: sdl2::event::EventPollIterator<'_> = self.event_pump.poll_iter();
        for event in events {
            match event {
                Event::Quit { .. } => self.is_running = false,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let pressed_key = utils::to_chip8_keycode(key);
                    self.emulator.handle_input(pressed_key as u8 , true);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let unpressed_key = utils::to_chip8_keycode(key);
                    self.emulator.handle_input(unpressed_key as u8, false);
                }
                _ => {
                    // Process input event
                    egui_state.process_input(&self.window, event, painter);
                }
            }
        }
    }

    fn process(&mut self) {
        let _ctx = self.window.gl_create_context().unwrap();

        let shader_ver: ShaderVersion = ShaderVersion::Adaptive;
        let (mut painter, mut egui_state) =
            egui_backend::with_sdl2(&self.window, shader_ver, DpiScaling::Custom(2.0));
        let emulator_texture_id =
            painter.new_user_texture_rgba8((64, 32), self.emulator.get_color_bufer(), false);

        while self.is_running {
            self.window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::Immediate)
                .unwrap();
            self.emulator.emulate_cycles(20);

            // create a canvas to draw on
            if self.emulator.is_draw_flag_set() {
                painter.update_user_texture_rgba8_data(
                    emulator_texture_id,
                    self.emulator.get_color_bufer()
                );
            }

            if self.emulator.is_sound_flag_set(){
                self.audio_device.play();
            }else {
                self.audio_device.pause();
            }

            self.render_ui(&mut painter, &mut egui_state, emulator_texture_id);
            std::thread::sleep(Duration::from_secs(1) / MAX_FRAMES_PER_SECOND);
        }
    }

    pub fn run(&mut self) {
        self.process();
    }
}
