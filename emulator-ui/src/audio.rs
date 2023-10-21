use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;


pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}


pub struct AudioDriver {
    device: AudioDevice<SquareWave>,
}


impl AudioDriver {
    pub fn new(audio_subsystem: &AudioSubsystem) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: None       // default sample size
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            SquareWave {
                phase_inc: 255.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

        AudioDriver {
            device: device,

        }
    }

    pub fn play(&self) {
        self.device.resume();
    }

    pub fn pause(&self) {
        self.device.pause();
    }
}