mod chip8;

use chip8::Chip8;

fn main() {

	let chip8 = Chip8::new();

	chip8.run_cycle();
}
