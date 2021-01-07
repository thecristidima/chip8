mod components;

use std::{thread, time::Duration};

use std::env;

use components::display::Display;
use components::input::Input;
use components::processor::Processor;

fn main() {
    let mut cpu = Processor::new();

    let args = env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        panic!("Usage: cargo run -- $path_to_rom");
    }

    cpu.load_rom(&args[1]);

    let sdl_context = sdl2::init().unwrap();

    let mut display = Display::new(&sdl_context, 10);
    let mut input = Input::new(&sdl_context);

    while let Ok(pressed_keys) = input.poll() {
        // Step 1. run a cpu cycle
        let (redraw, beep) = cpu.run_cycle(pressed_keys);

        // Step 2. check if vram was updated and redraw display
        if redraw {
            display.draw(cpu.get_vram_ref());

            thread::sleep(Duration::from_secs_f32(0.5));
        }

        // Step 3. Beep?
        if beep {
            // TODO Make noise
        } else {
            // TODO Stop making noise
        }

        // Step 4. Maybe add a thread sleep to make this baby run at 60Hz (check other implementations)
    }
}
