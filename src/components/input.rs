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
            };
        }

        let pressed_keys = self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect::<Vec<Keycode>>();

        // TODO parse pressed_keys

        Ok([false; 16])
    }
}
