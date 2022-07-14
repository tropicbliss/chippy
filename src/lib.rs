use byteorder::ReadBytesExt;
use fixedbitset::FixedBitSet;
use macroquad::{
    audio::{self, Sound},
    prelude::*,
};
use std::{fs::File, io::ErrorKind, path::Path};
use thiserror::Error;

const BEEP_SOUND: &[u8] = include_bytes!("../assets/sound.wav");

const FONT_SET: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xf0, 0x10, 0xf0, 0x80, 0xf0, 0xf0,
    0x10, 0xf0, 0x10, 0xf0, 0x90, 0x90, 0xf0, 0x10, 0x10, 0xf0, 0x80, 0xf0, 0x10, 0xf0, 0xf0, 0x80,
    0xf0, 0x90, 0xf0, 0xf0, 0x10, 0x20, 0x40, 0x40, 0xf0, 0x90, 0xf0, 0x90, 0xf0, 0xf0, 0x90, 0xf0,
    0x10, 0xf0, 0xf0, 0x90, 0xf0, 0x90, 0x90, 0xe0, 0x90, 0xe0, 0x90, 0xe0, 0xf0, 0x80, 0x80, 0x80,
    0xf0, 0xe0, 0x90, 0x90, 0x90, 0xe0, 0xf0, 0x80, 0xf0, 0x80, 0xf0, 0xf0, 0x80, 0xf0, 0x80, 0x80,
];

const KEY_MAP: [KeyCode; 16] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
];

pub struct CPU {
    registers: [u8; 16],
    program_counter: u16,
    memory: [u8; 4096],
    stack: [u16; 16],
    stack_pointer: u8,
    sound_timer: u8,
    delay_timer: u8,
    index_register: u16,
    framebuffer: FixedBitSet,
    sound: Sound,
    keys: FixedBitSet,
    display_width: u8,
    display_height: u8,
}

impl CPU {
    pub async fn new() -> Self {
        Self {
            registers: [0; 16],
            program_counter: 0x200,
            memory: [0; 4096],
            stack: [0; 16],
            stack_pointer: 0,
            sound_timer: 0,
            delay_timer: 0,
            index_register: 0,
            framebuffer: FixedBitSet::with_capacity((64 * 32) as usize),
            sound: audio::load_sound_from_bytes(BEEP_SOUND).await.unwrap(),
            keys: FixedBitSet::with_capacity(16),
            display_width: 64,
            display_height: 32,
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Chip8Error> {
        const MEMORY_START: u16 = 0x200;
        let mut file = File::open(path.as_ref())?;
        let mut opcodes = Vec::with_capacity(4096 - MEMORY_START as usize);
        loop {
            let result = file.read_u8();
            let opcode = match result {
                Ok(op) => op,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e.into()),
            };
            opcodes.push(opcode);
        }
        for (idx, f) in FONT_SET.into_iter().enumerate() {
            self.memory[idx] = f;
        }
        for (idx, d) in opcodes.into_iter().enumerate() {
            self.memory[MEMORY_START as usize + idx] = d;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Chip8Error> {
        let mut timer: u8 = 0;
        loop {
            timer += 1;
            if timer % 5 == 0 {
                self.tick();
                timer = 0;
            }
            for (idx, current_key) in KEY_MAP.into_iter().enumerate() {
                self.keys.set(idx, is_key_down(current_key));
            }
            let op_byte1 = self.memory[self.program_counter as usize] as u16;
            let op_byte2 = self.memory[self.program_counter as usize + 1] as u16;
            let mut opcode: u16 = op_byte1 << 8 | op_byte2;
            if self.program_counter == 0x200 && opcode == 0x1260 {
                // Init 64x64 hires mode
                self.display_width = 64;
                self.display_height = 64;
                opcode = 0x12C0;
                self.framebuffer = FixedBitSet::with_capacity(
                    self.display_height as usize * self.display_width as usize,
                );
            }
            let op_1 = (opcode & 0xF000) >> 12;
            let op_2 = (opcode & 0x0F00) >> 8;
            let op_3 = (opcode & 0x00F0) >> 4;
            let op_4 = opcode & 0x000F;
            let x = op_2 as u8;
            let y = op_3 as u8;
            let nnn = opcode & 0x0FFF;
            let kk = (opcode & 0x00FF) as u8;
            let n = op_4 as u8;
            self.next_instruction();
            match (op_1, op_2, op_3, op_4) {
                (0, 0, 0, 0) => return Ok(()),
                (0, 0, 0xE, 0) | (0, 2, 3, 0) => self.cls(),
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x1, _, _, _) => self.jp_addr(nnn),
                (0x2, _, _, _) => self.call_addr(nnn),
                (0x3, _, _, _) => self.se_vx_nn(x, kk),
                (0x4, _, _, _) => self.sne_vx_nn(x, kk),
                (0x5, _, _, _) => self.se_vx_vy(x, y),
                (0x6, _, _, _) => self.ld_vx_nn(x, kk),
                (0x7, _, _, _) => self.add_vx_nn(x, kk),
                (0x8, _, _, 0x0) => self.ld_vx_vy(x, y),
                (0x8, _, _, 0x1) => self.or_vx_vy(x, y),
                (0x8, _, _, 0x2) => self.and_vx_vy(x, y),
                (0x8, _, _, 0x3) => self.xor_vx_vy(x, y),
                (0x8, _, _, 0x4) => self.add_vx_vy(x, y),
                (0x8, _, _, 0x5) => self.sub_vx_vy(x, y),
                (0x8, _, _, 0x6) => self.shr_vx_vy(x),
                (0x8, _, _, 0x7) => self.subn_vx_vy(x, y),
                (0x8, _, _, 0xE) => self.shl_vx_vy(x),
                (0x9, _, _, _) => self.sne_vx_vy(x, y),
                (0xA, _, _, _) => self.ld_i_addr(nnn),
                (0xB, _, _, _) => self.jp_v0_addr(nnn),
                (0xC, _, _, _) => self.rnd_vx_nn(x, kk),
                (0xD, _, _, _) => self.drw_vx_vy_n(x, y, n),
                (0xE, _, 0x9, 0xE) => self.skp_vx(x),
                (0xE, _, 0xA, 0x1) => self.sknp_vx(x),
                (0xF, _, 0x0, 0x7) => self.ld_vx_dt(x),
                (0xF, _, 0x0, 0xA) => self.ld_vx_n(x),
                (0xF, _, 0x1, 0x5) => self.ld_dt_vx(x),
                (0xF, _, 0x1, 0x8) => self.ld_st_vx(x),
                (0xF, _, 0x1, 0xE) => self.add_i_vx(x),
                (0xF, _, 0x2, 0x9) => self.ld_f_vx(x),
                (0xF, _, 0x3, 0x3) => self.ld_b_vx(x),
                (0xF, _, 0x5, 0x5) => self.ld_i_vx(x),
                (0xF, _, 0x6, 0x5) => self.ld_vx_i(x),
                _ => return Err(Chip8Error::IllegalInstruction(opcode)),
            }
            let mut idx = 0;
            let width_multiplier = screen_width() / self.display_width as f32;
            let height_multiplier = screen_height() / self.display_height as f32;
            for row in 0..self.display_height {
                for col in 0..self.display_width {
                    let cell = self.framebuffer[idx];
                    let colour = if cell { GREEN } else { BLACK };
                    draw_rectangle(
                        col as f32 * width_multiplier,
                        row as f32 * height_multiplier,
                        width_multiplier,
                        height_multiplier,
                        colour,
                    );
                    idx += 1;
                }
            }
            next_frame().await;
        }
    }

    fn clear_display(&mut self) {
        self.framebuffer.clear();
        clear_background(BLACK);
    }

    fn enable_sound(&self) {
        audio::play_sound_once(self.sound.clone());
    }

    fn draw_pixel(&mut self, x: usize, y: usize, value: u8) -> bool {
        let idx = y * self.display_width as usize + x;
        let collision = self.framebuffer[idx];
        self.framebuffer.set(idx, (value == 1) ^ collision);
        collision
    }

    fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn next_instruction(&mut self) {
        self.program_counter += 2;
    }

    fn undo_instruction(&mut self) {
        self.program_counter -= 2;
    }

    // 00E0 - Clear the display
    fn cls(&mut self) {
        self.clear_display();
    }

    // 00EE - Return from a subroutine
    fn ret(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    // 1nnn - Jump to location nnn
    fn jp_addr(&mut self, addr: u16) {
        self.program_counter = addr;
    }

    // 2nnn - Call subroutine at nnn
    fn call_addr(&mut self, addr: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = addr;
    }

    // 3xnn - Skip next instruction if x = nn
    fn se_vx_nn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] == nn {
            self.next_instruction();
        }
    }

    // 4xnn - Skip next instruction if x != nn
    fn sne_vx_nn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] != nn {
            self.next_instruction();
        }
    }

    // 5xy0 - Skip next instruction if x = y
    fn se_vx_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.next_instruction();
        }
    }

    // 6xnn - Set x = nn
    fn ld_vx_nn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    }

    // 7xnn - Set x = x + nn
    fn add_vx_nn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] += nn;
    }

    // 8xy0 - Set x = y
    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
    }

    // 8xy1 - Set x = x OR y
    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    // 8xy2 - Set x = x AND y
    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    // 8xy3 - Set x = x XOR y
    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    // 8xy4 - Set x = x + y, set VF = carry
    fn add_vx_vy(&mut self, x: u8, y: u8) {
        let (wrapped, is_overflow) =
            self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[0xF] = if is_overflow { 1 } else { 0 };
        self.registers[x as usize] = wrapped;
    }

    // 8xy5 - Set x = x - y, set VF = NOT borrow
    fn sub_vx_vy(&mut self, x: u8, y: u8) {
        let (wrapped, is_overflow) =
            self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
        self.registers[0xF] = if is_overflow { 0 } else { 1 };
        self.registers[x as usize] = wrapped;
    }

    // 8xy6 - Set x = x SHR 1
    fn shr_vx_vy(&mut self, x: u8) {
        self.registers[0xF] = self.registers[x as usize] & 0x1;
        self.registers[x as usize] >>= 1;
    }

    // 8xy7 - Set x = y - x, set VF = NOT borrow
    fn subn_vx_vy(&mut self, x: u8, y: u8) {
        let (wrapped, is_overflow) =
            self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
        self.registers[0xF] = if is_overflow { 0 } else { 1 };
        self.registers[x as usize] = wrapped;
    }

    // 8xyE - Set x = x SHL 1
    fn shl_vx_vy(&mut self, x: u8) {
        self.registers[0xF] = self.registers[x as usize] >> 7;
        self.registers[x as usize] <<= 1;
    }

    // 9xy0 - Skip next instruction if x != y
    fn sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.next_instruction();
        }
    }

    // Annn - Set I = nnn
    fn ld_i_addr(&mut self, nnn: u16) {
        self.index_register = nnn;
    }

    // Bnnn - Jump to location nnn + V0
    fn jp_v0_addr(&mut self, nnn: u16) {
        self.program_counter = nnn + self.registers[0] as u16;
    }

    // Cxnn - Set Vx = random byte AND nn
    fn rnd_vx_nn(&mut self, x: u8, nn: u8) {
        let random = fastrand::u8(0..=u8::MAX);
        self.registers[x as usize] = random & nn;
    }

    // Dxyn - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn drw_vx_vy_n(&mut self, x: u8, y: u8, n: u8) {
        // If no pixels are erased, set VF to 0
        self.registers[0xF] = 0;
        // The interpreter reads n bytes from memory, starting at the address stored in I
        for i in 0..n {
            let line = self.memory[self.index_register as usize + i as usize];
            // Each byte is a line of eight pixels
            for position in 0..8 {
                // Get the byte to set by position
                let value = line >> (7 - position) & 0x01;
                if value == 1 {
                    // If this causes any pixels to be erased, VF is set to 1
                    let x = (self.registers[x as usize] as usize + position)
                        % self.display_width as usize; // wrap around width
                    let y = (self.registers[y as usize] as usize + i as usize)
                        % self.display_height as usize; // wrap around height
                    if self.draw_pixel(x, y, value) {
                        self.registers[0xF] = 1;
                    }
                }
            }
        }
    }

    // Ex9E - Skip next instruction if key with the value of Vx is pressed
    fn skp_vx(&mut self, x: u8) {
        if self.keys[self.registers[x as usize] as usize] {
            self.next_instruction();
        }
    }

    // ExA1 - Skip next instruction if key with the value of Vx is not pressed
    fn sknp_vx(&mut self, x: u8) {
        if !self.keys[self.registers[x as usize] as usize] {
            self.next_instruction();
        }
    }

    // Fx07 - Set Vx = delay timer value
    fn ld_vx_dt(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    // Fx0A - Wait for a key press, store the value of the key in Vx
    fn ld_vx_n(&mut self, x: u8) {
        self.undo_instruction();
        for idx in 0..16 {
            let key = self.keys[idx as usize];
            if key {
                self.registers[x as usize] = idx;
                self.next_instruction();
                break;
            }
        }
    }

    // Fx15 - Set delay timer = Vx
    fn ld_dt_vx(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    }

    // Fx18 - Set sound timer = Vx
    fn ld_st_vx(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
        if self.sound_timer > 0 {
            self.enable_sound();
        }
    }

    // Fx1E - Set I = I + Vx
    fn add_i_vx(&mut self, x: u8) {
        self.index_register += self.registers[x as usize] as u16;
    }

    // Fx29 - Set I = location of sprite for digit Vx
    fn ld_f_vx(&mut self, x: u8) {
        self.index_register = self.registers[x as usize] as u16 * 5;
    }

    // Fx33 - Store BCD representation of Vx in memory locations I, I+1, and I+2
    // BCD means binary-coded decimal
    // If VX is 0xef, or 239, we want 2, 3, and 9 in I, I+1, and I+2
    fn ld_b_vx(&mut self, x: u8) {
        self.memory[self.index_register as usize] = self.registers[x as usize] / 100;
        self.memory[self.index_register as usize + 1] = (self.registers[x as usize] / 10) % 10;
        self.memory[self.index_register as usize + 2] = self.registers[x as usize] % 10;
    }

    // Fx55 - Store registers V0 through Vx in memory starting at location I
    fn ld_i_vx(&mut self, x: u8) {
        self.memory[(self.index_register as usize)..(self.index_register + x as u16 + 1) as usize]
            .copy_from_slice(&self.registers[0..(x as usize + 1)]);
    }

    // Fx65 - Read registers V0 through Vx from memory starting at location I
    fn ld_vx_i(&mut self, x: u8) {
        self.registers[0..(x as usize + 1)].copy_from_slice(
            &self.memory
                [(self.index_register as usize)..(self.index_register + x as u16 + 1) as usize],
        );
    }
}

#[derive(Error, Debug)]
pub enum Chip8Error {
    #[error("error reading file")]
    Io(#[from] std::io::Error),

    #[error("illegal instruction: {0:04x}")]
    IllegalInstruction(u16),
}
