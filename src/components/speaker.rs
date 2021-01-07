use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    Sdl,
};

// Most of this is a shameless copy-paste of https://rust-sdl2.github.io/rust-sdl2/sdl2/audio/index.html
pub struct Speaker {
    audio_device: AudioDevice<SquareWave>,
}

impl Speaker {
    pub fn new(sdl_context: &Sdl) -> Speaker {
        let audio_subsystem = sdl_context.audio().unwrap();

        let audio_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem
            .open_playback(None, &audio_spec, |spec| SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            })
            .unwrap();

        Speaker {
            audio_device: device,
        }
    }

    pub fn play(&mut self) {
        self.audio_device.resume();
    }

    pub fn stop(&mut self) {
        self.audio_device.pause();
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
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