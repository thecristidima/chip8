mod components;

use components::display::Display;
use components::input::Input;
use components::processor::Processor;

fn main() {
    let mut cpu = Processor::new(true);

    cpu.load_rom("roms/pong.rom");

    let sdl_context = sdl2::init().unwrap();

    let display = Display::new(&sdl_context);
    let mut input = Input::new(&sdl_context);

    while let Ok(_) = input.poll() {
        // Step 1. run a cpu cycle

        // Step 2. check if vram was updated and redraw display

        // Step 3. Beep?

        // Step 4. Maybe add a thread sleep to make this baby run at 60Hz (check other implementations)
    }
}
