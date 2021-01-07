use sdl2::{event::Event, keyboard::Keycode, EventPump, Sdl};

pub struct Input {
    event_pump: EventPump,
}

impl Input {
    pub fn new(sdl_context: &Sdl) -> Input {
        let event_pump = sdl_context.event_pump().unwrap();

        Input { event_pump }
    }

    pub fn poll(&mut self) -> Result<[bool; 16], ()> {
        for event in self.event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            }
        }

        let pressed_keys_codes = self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect::<Vec<Keycode>>();

        let mut pressed_keys = [false; 16];

        // Key mappings are as follows
        // 1 2 3 C -> 1 2 3 4
        // 4 5 6 D -> Q W E R
        // 7 8 9 E -> A S D F
        // A 0 B F -> Z X C V
        for keycode in &pressed_keys_codes {
            match *keycode {
                Keycode::Num1 => pressed_keys[0x1] = true,
                Keycode::Num2 => pressed_keys[0x2] = true,
                Keycode::Num3 => pressed_keys[0x3] = true,
                Keycode::Num4 => pressed_keys[0xC] = true,
                Keycode::Q => pressed_keys[0x4] = true,
                Keycode::W => pressed_keys[0x5] = true,
                Keycode::E => pressed_keys[0x6] = true,
                Keycode::R => pressed_keys[0xD] = true,
                Keycode::A => pressed_keys[0x7] = true,
                Keycode::S => pressed_keys[0x8] = true,
                Keycode::D => pressed_keys[0x9] = true,
                Keycode::F => pressed_keys[0xE] = true,
                Keycode::Z => pressed_keys[0xA] = true,
                Keycode::X => pressed_keys[0x0] = true,
                Keycode::C => pressed_keys[0xF] = true,
                Keycode::V => pressed_keys[0xB] = true,
                _ => {}
            }
        }

        Ok(pressed_keys)
    }
}
