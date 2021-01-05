mod chip8;

use chip8::Chip8;

fn main() {
	let mut chip8 = Chip8::new(true);

	chip8.load_rom("roms/pong.rom");

    loop {
        chip8.run_cycle();
    }

	// chip8.run_cycle();
}
