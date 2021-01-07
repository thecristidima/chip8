mod components;

use std::{thread, time::Duration};

use std::env;

use components::display::Display;
use components::input::Input;
use components::processor::Processor;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        panic!("Usage: cargo run -- $path_to_rom");
    }

    let mut cpu = Processor::new();
    cpu.load_rom(&args[1]);

    let sdl_context = sdl2::init().unwrap();

    let mut display = Display::new(&sdl_context, 10);
    let mut input = Input::new(&sdl_context);

    while let Ok(pressed_keys) = input.poll() {
        let (redraw, beep) = cpu.run_cycle(pressed_keys);

        if redraw {
            display.draw(cpu.get_vram_ref());
        }

        if beep {
            // TODO Make noise
        } else {
            // TODO Stop making noise
        }

        thread::sleep(Duration::from_millis(2));
    }
}
