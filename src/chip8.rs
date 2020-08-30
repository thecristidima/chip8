pub struct Chip8 {
	// 4KB of memory
	// First 512 bytes were used by interpreter, but we can ignore this limitation
	mem:			[u8; 4096],

	// 16 8bit V registers V0 to VF
	// During addition VF is a carry flag, while during substraction it is a "no borrow" flag
	v_reg:			[u8; 16],

	// Single 16bit register I (address register)
	i_reg:			u16,
	
	// 48 bytes stack for 12 levels of nesting
	stack:			[u32; 12],

	// Two timers - delay and sound
	delay_timer:	u8,
	sound_timer:	u8,

	// Program counter
	pc:				u32,
}

impl Chip8 {
	pub fn new() -> Chip8 {
		Chip8 {
			mem: 	[0; 4096],
			v_reg:	[0; 16],
			i_reg:	0,
			stack:	[0; 12],
			
			delay_timer: 0,
			sound_timer: 0,

			pc: 0x200
		}
	}

	pub fn run_cycle(self) {
		println!("Cycle?");
	}
}
