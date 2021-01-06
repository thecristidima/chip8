#![allow(non_snake_case)] // for operations such as 0xAnnn

use std::fs;

use rand;

pub struct Processor {
    mem: [u8; 4096],          // 4k memory
    v: [u8; 16],              // 16 8-bit general purpose registers
    i: usize,                 // stores memory address (16-bit)
    stack: [u16; 16],         // 16 levels
    delay_timer: u8,          // 60 Hz timer
    sound_timer: u8,          // same
    pc: usize,                // program counter (16-bit)
    sp: usize,                // stack pointer (8-bit)
    vram: [[u8; 64]; 32],     // display is 64 wide x 32 tall
    pressed_keys: [bool; 16], // array of pressed keys
    wait_for_key: bool,       // halt execution until user presses key

    enable_logs: bool, // debug purposes
}

impl Processor {
    pub fn new(enable_logs: bool) -> Processor {
        Processor {
            mem: [0; 4096],
            v: [0; 16],
            i: 0,
            stack: [0; 16],

            delay_timer: 0,
            sound_timer: 0,

            pc: 0x200,
            sp: 0,
            vram: [[0; 64]; 32],

            pressed_keys: [false; 16],
            wait_for_key: false,
            enable_logs,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom = fs::read(path).expect("ROM file can be found");

        self.mem[512..512 + rom.len()].copy_from_slice(&rom);

        if self.enable_logs {
            println!("Loaded ROM, total bytes: {}\n", rom.len());
        }
    }

    pub fn run_cycle(&mut self) {
        let instruction = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;

        if self.enable_logs {
            println!(
                "Reading instruction - high byte {:#2x}, low byte {:#2x} - {:#4x}",
                self.mem[self.pc],
                self.mem[self.pc + 1],
                instruction
            );
        }

        self.run_instruction(instruction);
    }

    fn run_instruction(&mut self, instr: u16) {
        let nibbles = (
            ((instr & 0xF000) >> 12) as u8,
            ((instr & 0x0F00) >> 8) as u8,
            ((instr & 0x00F0) >> 4) as u8,
            ((instr & 0x000F) >> 0) as u8,
        );

        let nnn = instr & 0x0FFF;
        let kk = (instr & 0x00FF) as u8;
        let x = nibbles.1; // _low_ 4 bits of _high_ byte
        let y = nibbles.2; // _high_ 4 bits of _low_ byte
        let n = nibbles.3; // _low_ 4 bits of _low_ byte

        match nibbles {
            (0x00, 0x00, 0x0E, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0E, 0x0E) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, _) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0E) => self.op_8xyE(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0A, _, _, _) => self.op_Annn(nnn),
            (0x0B, _, _, _) => self.op_Bnnn(nnn),
            (0x0C, _, _, _) => self.op_Cxkk(x, kk),
            (0x0D, _, _, _) => self.op_Dxyn(x, y, n),
            (0x0E, _, 0x09, 0x0E) => self.op_Ex9E(x),
            (0x0E, _, 0x0A, 0x01) => self.op_ExA1(x),
            (0x0F, _, _, 0x07) => self.op_Fx07(x),
            (0x0F, _, _, 0x0A) => self.op_Fx0A(x),
            (0x0F, _, 0x01, 0x05) => self.op_Fx15(x),
            (0x0F, _, 0x01, 0x08) => self.op_Fx18(x),
            (0x0F, _, 0x01, 0x0E) => self.op_Fx1E(x),
            (0x0F, _, 0x02, 0x09) => self.op_Fx29(x),
            (0x0F, _, 0x03, 0x03) => self.op_Fx33(x),
            (0x0F, _, 0x05, 0x05) => self.op_Fx55(x),
            (0x0F, _, 0x06, 0x05) => self.op_Fx65(x),
            _ => self.debug_and_panic(instr),
        }

        if !self.wait_for_key {
            println!("Before change: {}", self.pc);
            self.pc += 2; // advance PC to next instruction
            println!("After change: {}", self.pc);
        }
    }

    // TODO Copy-paste documentation to each function
    fn op_00e0(&mut self) {
        self.vram = [[0; 64]; 32];
    }

    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize] as usize;
        self.sp -= 1; // Maybe check if this underflows?
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize;
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc as u16;
        self.pc = nnn as usize;
    }

    fn op_3xkk(&mut self, x: u8, kk: u8) {
        let vx = self.v[x as usize];
        if vx == kk {
            self.pc += 2;
        }
    }

    fn op_4xkk(&mut self, x: u8, kk: u8) {
        let vx = self.v[x as usize];
        if vx != kk {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: u8, y: u8) {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];
        if vx == vy {
            self.pc += 2;
        }
    }

    fn op_6xkk(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;
    }

    // TODO check for overflow?
    // documentation doesn't mention it, probably not an issue
    fn op_7xkk(&mut self, x: u8, kk: u8) {
        self.v[x as usize] += kk;
    }

    fn op_8xy0(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
    }

    fn op_8xy2(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
    }

    fn op_8xy3(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
    }

    fn op_8xy4(&mut self, x: u8, y: u8) {
        let vx = self.v[x as usize] as u16;
        let vy = self.v[y as usize] as u16;

        self.v[x as usize] = ((vx + vy) % 0x100) as u8;

        let set_carry = (vx + vy) > u8::MAX as u16;
        if set_carry {
            self.v[0x0F] = 1;
        }
    }

    fn op_8xy5(&mut self, x: u8, y: u8) {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        self.v[x as usize] = vx.wrapping_sub(vy);

        self.v[0x0F] = if vx > vy { 1 } else { 0 };
    }

    fn op_8xy6(&mut self, x: u8) {
        self.v[0x0F] = if self.v[x as usize] % 2 == 1 { 1 } else { 0 };

        self.v[x as usize] >>= 1;
    }

    fn op_8xy7(&mut self, x: u8, y: u8) {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        self.v[0x0F] = if vy > vx { 1 } else { 0 };

        self.v[x as usize] = vy.wrapping_sub(vx);
    }

    fn op_8xyE(&mut self, x: u8) {
        self.v[0x0F] = if self.v[x as usize] & 0b1000_0000 != 0 {
            1
        } else {
            0
        };

        self.v[x as usize] <<= 2;
    }

    fn op_9xy0(&mut self, x: u8, y: u8) {
        if self.v[x as usize] != self.v[y as usize] {
            self.pc += 2;
        }
    }

    fn op_Annn(&mut self, nnn: u16) {
        self.i = nnn as usize;
    }

    fn op_Bnnn(&mut self, nnn: u16) {
        self.pc = (self.v[0] as u16 + nnn) as usize;
    }

    fn op_Cxkk(&mut self, x: u8, kk: u8) {
        let rand_byte = rand::random::<u8>();

        self.v[x as usize] = rand_byte & kk;
    }

    fn op_Dxyn(&mut self, x: u8, y: u8, n: u8) {
        self.v[0x0F] = 0; // initially assume we don't flip any display bits

        for i in 0..n {
            let data = self.mem[self.i + i as usize];

            // wrap line here, wrap column in draw_line
            self.draw_line(x as usize, ((y + i) % 32) as usize, data);
        }
    }

    fn draw_line(&mut self, x: usize, y: usize, data: u8) {
        for bit_pos in 0..8 {
            let bit = data & (0x80 >> bit_pos);

            // TODO I only considered a bit erased if we _flip_ it to 0
            // In case you run out of ideas while debugging, play around with this
            if self.vram[y][(x + bit_pos) % 64] == 1 && bit == 1 {
                self.v[0x0F] = 1;
            }

            self.vram[y][(x + bit_pos) % 64] ^= bit;
        }
    }

    fn op_Ex9E(&mut self, x: u8) {
        if self.pressed_keys[self.v[x as usize] as usize] {
            self.pc += 2;
        }
    }

    fn op_ExA1(&mut self, x: u8) {
        if !self.pressed_keys[self.v[x as usize] as usize] {
            self.pc += 2;
        }
    }

    fn op_Fx07(&mut self, x: u8) {
        self.v[x as usize] = self.delay_timer;
    }

    fn op_Fx0A(&mut self, x: u8) {
        let initial_wait_for_key = self.wait_for_key;

        self.wait_for_key = true;

        for (idx, key_pressed) in self.pressed_keys.iter().enumerate() {
            if *key_pressed {
                self.wait_for_key = false;
                self.v[x as usize] = idx as u8;
                break;
            }
        }

        if self.wait_for_key && !initial_wait_for_key && self.enable_logs {
            println!("Waiting for user input...");
        }
    }

    fn op_Fx15(&mut self, x: u8) {
        self.delay_timer = self.v[x as usize];
    }

    fn op_Fx18(&mut self, x: u8) {
        self.sound_timer = self.v[x as usize];
    }

    fn op_Fx1E(&mut self, x: u8) {
        self.i += self.v[x as usize] as usize;
    }

    fn op_Fx29(&mut self, x: u8) {
        self.i = self.v[x as usize] as usize * 5;
    }

    fn op_Fx33(&mut self, x: u8) {
        let vx = self.v[x as usize];

        self.mem[self.i] = vx / 100 % 10;
        self.mem[self.i + 1] = vx / 10 % 10;
        self.mem[self.i + 2] = vx % 10;
    }

    fn op_Fx55(&mut self, x: u8) {
        for idx in 0..=x {
            self.mem[self.i + idx as usize] = self.v[idx as usize];
        }
    }

    fn op_Fx65(&mut self, x: u8) {
        for idx in 0..=x {
            self.v[idx as usize] = self.mem[self.i + idx as usize];
        }
    }

    // Debug functions below
    fn debug(&self) {
        println!("\n\nPrinting current state:\n");

        println!("Vx: {:?}", self.v);
        println!("I: {:?}", self.i);
        println!("Stack (and pointer): {:?} -> {}", self.stack, self.sp);
        println!("PC: {}", self.pc);
        println!("Delay timer: {}", self.delay_timer);
        println!("Sound timer: {}", self.sound_timer);
        println!("Pressed keys: {:?}", self.pressed_keys);
        println!("Wait for user input? {}", self.wait_for_key);

        print!("\n\n");
    }

    fn debug_and_panic(&self, instruction: u16) {
        self.debug();

        panic!(
            "Ran into an error with instruction (might be unknown): {:#4x}",
            instruction
        );
    }
}
