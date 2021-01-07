#![allow(non_snake_case)] // for operations such as 0xAnnn

use std::fs;

use rand;

const FONT_SET: [u8; 80] = [
    // 16 sprites, each is 5 bytes long
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

enum ProgramCounterAction {
    Advance,
    SkipNext,
    Jump(usize),
    WaitForKey,
}

impl ProgramCounterAction {
    fn action_if(
        condition: bool,
        action_true: ProgramCounterAction,
        action_false: ProgramCounterAction,
    ) -> ProgramCounterAction {
        if condition {
            action_true
        } else {
            action_false
        }
    }

    fn skip_if(condition: bool) -> ProgramCounterAction {
        ProgramCounterAction::action_if(
            condition,
            ProgramCounterAction::SkipNext,
            ProgramCounterAction::Advance,
        )
    }
}

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
    redraw: bool,             // helper flag set whenever DRAW is called
}

impl Processor {
    pub fn new() -> Processor {
        let mut mem = [0; 4096];

        mem[0..80].copy_from_slice(&FONT_SET);

        Processor {
            mem,
            v: [0; 16],
            i: 0,
            stack: [0; 16],

            delay_timer: 0,
            sound_timer: 0,

            pc: 0x200,
            sp: 0,
            vram: [[0; 64]; 32],

            pressed_keys: [false; 16],
            redraw: false,
        }
    }

    pub fn get_vram_ref(&self) -> &[[u8; 64]; 32] {
        &self.vram
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom = fs::read(path).expect("ROM file can be found");

        self.mem[512..512 + rom.len()].copy_from_slice(&rom);

        println!("Loaded ROM, total bytes: {}\n", rom.len());
    }

    pub fn run_cycle(&mut self, pressed_keys: [bool; 16]) -> (bool, bool) {
        self.redraw = false;
        self.pressed_keys = pressed_keys;

        let instruction = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;

        self.run_instruction(instruction);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        let beep = if self.sound_timer > 0 { true } else { false };

        (self.redraw, beep)
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

        let pc_action = match nibbles {
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
            _ => ProgramCounterAction::Advance,
        };

        match pc_action {
            ProgramCounterAction::Advance => self.pc += 2,
            ProgramCounterAction::SkipNext => self.pc += 4,
            ProgramCounterAction::Jump(addr) => self.pc = addr,
            ProgramCounterAction::WaitForKey => {}
        };
    }

    // 00E0 - CLS
    // Clear the display.
    fn op_00e0(&mut self) -> ProgramCounterAction {
        self.vram = [[0; 64]; 32];

        ProgramCounterAction::Advance
    }

    // 00EE - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top
    // of the stack, then subtracts 1 from the stack pointer.
    fn op_00ee(&mut self) -> ProgramCounterAction {
        self.pc = self.stack[self.sp as usize] as usize;
        self.sp -= 1; // Maybe check if this underflows?

        ProgramCounterAction::Advance
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: u16) -> ProgramCounterAction {
        ProgramCounterAction::Jump(nnn as usize)
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC
    // on the top of the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, nnn: u16) -> ProgramCounterAction {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc as u16;

        ProgramCounterAction::Jump(nnn as usize)
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal,
    // increments the program counter by 2.
    fn op_3xkk(&mut self, x: u8, kk: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];

        ProgramCounterAction::skip_if(vx == kk)
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: u8, kk: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];

        ProgramCounterAction::skip_if(vx != kk)
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        ProgramCounterAction::skip_if(vx == vy)
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    fn op_6xkk(&mut self, x: u8, kk: u8) -> ProgramCounterAction {
        self.v[x as usize] = kk;

        ProgramCounterAction::Advance
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Note: I'm doing a wrapping add here because one of the test ROMs
    // will run this instruction and the result won't fit in a u8.
    // I personally think this is a flaw in the test ROM, but other
    // programs aren't affected by this change (probably because they
    // are designed better).
    fn op_7xkk(&mut self, x: u8, kk: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);

        ProgramCounterAction::Advance
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    fn op_8xy0(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.v[y as usize];

        ProgramCounterAction::Advance
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];

        ProgramCounterAction::Advance
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];

        ProgramCounterAction::Advance
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];

        ProgramCounterAction::Advance
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize] as u16;
        let vy = self.v[y as usize] as u16;

        self.v[x as usize] = ((vx + vy) % 0x100) as u8;

        let set_carry = (vx + vy) > u8::MAX as u16;
        if set_carry {
            self.v[0x0F] = 1;
        }

        ProgramCounterAction::Advance
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        self.v[x as usize] = vx.wrapping_sub(vy);

        self.v[0x0F] = if vx > vy { 1 } else { 0 };

        ProgramCounterAction::Advance
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx >> 1.
    fn op_8xy6(&mut self, x: u8) -> ProgramCounterAction {
        self.v[0x0F] = self.v[x as usize] % 2;

        self.v[x as usize] >>= 1;

        ProgramCounterAction::Advance
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        self.v[0x0F] = if vy > vx { 1 } else { 0 };

        self.v[x as usize] = vy.wrapping_sub(vx);

        ProgramCounterAction::Advance
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx << 1.
    fn op_8xyE(&mut self, x: u8) -> ProgramCounterAction {
        self.v[0x0F] = if self.v[x as usize] & 0b1000_0000 != 0 {
            1
        } else {
            0
        };

        self.v[x as usize] <<= 1;

        ProgramCounterAction::Advance
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: u8, y: u8) -> ProgramCounterAction {
        ProgramCounterAction::skip_if(self.v[x as usize] != self.v[y as usize])
    }

    // Annn - LD I, addr
    // Set I = nnn.
    fn op_Annn(&mut self, nnn: u16) -> ProgramCounterAction {
        self.i = nnn as usize;

        ProgramCounterAction::Advance
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    fn op_Bnnn(&mut self, nnn: u16) -> ProgramCounterAction {
        ProgramCounterAction::Jump((self.v[0] as u16 + nnn) as usize)
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    fn op_Cxkk(&mut self, x: u8, kk: u8) -> ProgramCounterAction {
        let rand_byte = rand::random::<u8>();

        self.v[x as usize] = rand_byte & kk;

        ProgramCounterAction::Advance
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    //
    // The interpreter reads n bytes from memory, starting at the address stored in I.
    // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    // If the sprite is positioned so part of it is outside the coordinates of the display,
    // it wraps around to the opposite side of the screen.
    fn op_Dxyn(&mut self, x: u8, y: u8, n: u8) -> ProgramCounterAction {
        self.redraw = true;
        self.v[0x0F] = 0; // initially assume we don't flip any display bits

        let x = self.v[x as usize] as usize;
        let y = self.v[y as usize] as usize;

        for i in 0..n as usize {
            let data = self.mem[self.i + i];

            // wrap line here, wrap column in draw_line
            self.draw_line(x, (y + i) % 32, data);
        }

        ProgramCounterAction::Advance
    }

    fn draw_line(&mut self, x: usize, y: usize, data: u8) {
        for bit_pos in 0..8 {
            let bit = data & (0x80 >> bit_pos);

            if self.vram[y][(x + bit_pos) % 64] == 1 && bit == 1 {
                self.v[0x0F] = 1;
            }

            self.vram[y][(x + bit_pos) % 64] ^= bit;
        }
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    fn op_Ex9E(&mut self, x: u8) -> ProgramCounterAction {
        ProgramCounterAction::skip_if(self.pressed_keys[self.v[x as usize] as usize])
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_ExA1(&mut self, x: u8) -> ProgramCounterAction {
        ProgramCounterAction::skip_if(!self.pressed_keys[self.v[x as usize] as usize])
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    fn op_Fx07(&mut self, x: u8) -> ProgramCounterAction {
        self.v[x as usize] = self.delay_timer;

        ProgramCounterAction::Advance
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    fn op_Fx0A(&mut self, x: u8) -> ProgramCounterAction {
        let mut user_pressed_key = false;

        for (idx, key_pressed) in self.pressed_keys.iter().enumerate() {
            if *key_pressed {
                user_pressed_key = true;
                self.v[x as usize] = idx as u8;
                break;
            }
        }

        ProgramCounterAction::action_if(
            user_pressed_key,
            ProgramCounterAction::Advance,
            ProgramCounterAction::WaitForKey,
        )
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    fn op_Fx15(&mut self, x: u8) -> ProgramCounterAction {
        self.delay_timer = self.v[x as usize];

        ProgramCounterAction::Advance
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    fn op_Fx18(&mut self, x: u8) -> ProgramCounterAction {
        self.sound_timer = self.v[x as usize];

        ProgramCounterAction::Advance
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    fn op_Fx1E(&mut self, x: u8) -> ProgramCounterAction {
        self.i += self.v[x as usize] as usize;

        ProgramCounterAction::Advance
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    fn op_Fx29(&mut self, x: u8) -> ProgramCounterAction {
        self.i = self.v[x as usize] as usize * 5;

        ProgramCounterAction::Advance
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // Note: Hundreds digit at I, tens digit at I+1, last digit at I+2.
    fn op_Fx33(&mut self, x: u8) -> ProgramCounterAction {
        let vx = self.v[x as usize];

        self.mem[self.i] = vx / 100 % 10;
        self.mem[self.i + 1] = vx / 10 % 10;
        self.mem[self.i + 2] = vx % 10;

        ProgramCounterAction::Advance
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    fn op_Fx55(&mut self, x: u8) -> ProgramCounterAction {
        for idx in 0..=x {
            self.mem[self.i + idx as usize] = self.v[idx as usize];
        }

        ProgramCounterAction::Advance
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    fn op_Fx65(&mut self, x: u8) -> ProgramCounterAction {
        for idx in 0..=x {
            self.v[idx as usize] = self.mem[self.i + idx as usize];
        }

        ProgramCounterAction::Advance
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        let instruction = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;

        println!("PC: {}, I: {}, SP: {}", self.pc, self.i, self.sp);
        println!("Vx: {:?}", self.v);
        println!("Next instruction: {:#4x}\n", instruction);
    }
}

#[cfg(test)]
mod pc_tests {
    use super::*;

    #[test]
    fn test_pc_advance() {
        let mut cpu = Processor::new();

        cpu.mem[cpu.pc] = 0x00;
        cpu.mem[cpu.pc + 1] = 0xE0;

        cpu.run_cycle([false; 16]);

        assert_eq!(
            0x202, cpu.pc,
            "PC should be incremented by 2 after a normal instruction"
        );
    }

    #[test]
    fn test_pc_skip() {
        let mut cpu = Processor::new();

        cpu.mem[cpu.pc] = 0x30;
        cpu.mem[cpu.pc + 1] = 0x00;

        cpu.run_cycle([false; 16]);

        assert_eq!(
            0x204, cpu.pc,
            "PC should be incremented by 4 after a skip instruction"
        );
    }

    #[test]
    fn test_pc_jump() {
        let mut cpu = Processor::new();

        cpu.mem[cpu.pc] = 0x1A;
        cpu.mem[cpu.pc + 1] = 0xBC;

        cpu.run_cycle([false; 16]);

        assert_eq!(
            0xABC, cpu.pc,
            "PC should be set to address value after a jump instruction"
        );
    }
}
