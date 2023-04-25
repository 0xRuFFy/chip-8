use rand::Rng;
use std::fmt;

use sdl2::{
    render::Canvas,
    video::Window, pixels::Color, rect::Rect
};

macro_rules! invalid_opcode {
    ($opcode:expr) => {
        panic!("Invalid opcode: 0x{:x}", $opcode);
    };
}

const MEMORY_SIZE: usize = 4096;
const MEMORY_START: u16 = 0x200;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const INSTRUCTION_SIZE: u16 = 2;

const FONTSET_SIZE: usize = 80;
const FONTSET_START: usize = 0x050;
const FONTSET: &'static [u8; FONTSET_SIZE] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn get_nibble(value: u16, index: u8) -> u8 {
    ((value & (0x000f << (4 * index))) >> (4 * index)) as u8
}

fn get_addr(value: u16) -> u16 {
    value & 0x0fff
}

fn get_kk(value: u16) -> u8 {
    (value & 0x00ff) as u8
}

#[allow(dead_code)]
pub struct C8Cpu {
    memory: [u8; MEMORY_SIZE],      // 4KB memory
    v: [u8; REGISTER_COUNT],        // 16 8-bit general purpose registers
    i: u16,                         // 16-bit index register
    dt: u8,                         // 8-bit delay timer
    st: u8,                         // 8-bit sound timer
    stack: [u16; STACK_SIZE],       // 16 16-bit stack
    pc: u16,                        // 16-bit program counter
    sp: u8,                         // 8-bit stack pointer
    keypad: u16,                    // 16-bit keypad
    display: [u64; DISPLAY_HEIGHT], // 64x32 display

    pub draw_flag: bool,
    pub waiting_for_key: bool,
    pub waiting_for_key_register: u8,
}

impl C8Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; REGISTER_COUNT],
            i: 0,
            dt: 0,
            st: 0,
            stack: [0; STACK_SIZE],
            pc: MEMORY_START, // first 512 bytes used to be reserved for the interpreter
            sp: 0,
            keypad: 0,
            display: [0; DISPLAY_HEIGHT],
            draw_flag: false,
            waiting_for_key: false,
            waiting_for_key_register: 0,
        };
        cpu.load_fontset();

        cpu
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.memory[i + MEMORY_START as usize] = rom[i];
        }
    }

    fn load_fontset(&mut self) {
        for i in 0..FONTSET_SIZE {
            self.memory[i + FONTSET_START] = FONTSET[i];
        }
    }

    fn inc_pc(&mut self) {
        self.pc += INSTRUCTION_SIZE;
    }

    fn fetch(&self) -> u16 {
        let mut opcode: u16 = 0;
        for i in 0..INSTRUCTION_SIZE as usize {
            opcode <<= 8;
            opcode |= self.memory[self.pc as usize + i] as u16;
        }
        opcode
    }

    fn cls(&mut self) {
        for i in 0..DISPLAY_HEIGHT {
            self.display[i] = 0;
        }
        self.draw_flag = true;
    }

    #[allow(dead_code)]
    pub fn single_cycle(&mut self) {
        let opcode = self.fetch();
        println!("opcode: {:04x}", opcode);
        let opcode_ms_nibble = get_nibble(opcode, 3);
        let x = get_nibble(opcode, 2) as usize;
        let y = get_nibble(opcode, 1) as usize;
        let n = get_nibble(opcode, 0) as usize;
        let kk = get_kk(opcode);
        let addr = get_addr(opcode);

        match opcode_ms_nibble {
            0x0 => match opcode {
                0x00e0 => {
                    // CLS
                    self.cls();
                    self.inc_pc();
                }
                0x00ee => {
                    // RET
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {
                    invalid_opcode!(opcode);
                }
            },
            0x1 => {
                // JP addr
                self.pc = addr;
            }
            0x2 => {
                // CALL addr
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }
            0x3 => {
                // SE Vx, byte
                if self.v[x] == kk {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            0x4 => {
                // SNE Vx, byte
                if self.v[x] != kk {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            0x5 => {
                match n {
                    0 => {
                        // SE Vx, Vy
                        if self.v[x] == y as u8 {
                            self.inc_pc();
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
                self.inc_pc();
            }
            0x6 => {
                // LD Vx, byte
                self.v[x] = kk;
                self.pc += INSTRUCTION_SIZE;
            }
            0x7 => {
                // ADD Vx, byte
                self.v[x] = self.v[x].wrapping_add(kk);
                self.inc_pc();
            }
            0x8 => {
                match n {
                    0 => {
                        // LD Vx, Vy
                        self.v[x] = self.v[y];
                    }
                    1 => {
                        // OR Vx, Vy
                        self.v[x] |= self.v[y];
                    }
                    2 => {
                        // AND Vx, Vy
                        self.v[x] &= self.v[y];
                    }
                    3 => {
                        // XOR Vx, Vy
                        self.v[x] ^= self.v[y];
                    }
                    4 => {
                        // ADD Vx, Vy
                        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = result;
                        self.v[0xf] = overflow as u8;
                    }
                    5 => {
                        // SUB Vx, Vy
                        self.v[0xf] = 0;
                        if self.v[x] > self.v[y] {
                            self.v[0xf] = 1;
                        }
                        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    }
                    6 => {
                        // SHR Vx {, Vy}
                        self.v[0xf] = self.v[x] & 0x1;
                        self.v[x] >>= 1; // divide by 2
                    }
                    7 => {
                        // SUBN Vx, Vy
                        self.v[0xf] = 0;
                        if self.v[y] > self.v[x] {
                            self.v[0xf] = 1;
                        }
                        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    }
                    0xe => {
                        // SHL Vx {, Vy}
                        self.v[0xf] = self.v[x] >> 7;
                        self.v[x] <<= 1; // multiply by 2
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
                self.inc_pc();
            }
            0x9 => {
                match n {
                    0 => {
                        // SNE Vx, Vy
                        if self.v[x] != self.v[y] {
                            self.inc_pc();
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
                self.inc_pc();
            }
            0xa => {
                // LD I, addr
                self.i = addr;
                self.inc_pc();
            }
            0xb => {
                // JP V0, addr
                self.pc = addr + self.v[0] as u16;
            }
            0xc => {
                // RND Vx, byte
                self.v[x] = rand::thread_rng().gen::<u8>() & kk;
                self.inc_pc();
            }
            0xd => {
                // DRW Vx, Vy, nibble
                let _x = self.v[x] as usize;
                let _y = self.v[y] as usize;

                self.v[0xf] = 0; // reset collision flag

                for i in 0..n {
                    let sprite = self.memory[(self.i + i as u16) as usize];
                    for j in 0..8 {
                        let pixel = (sprite & (0x80 >> j)) >> (7 - j);
                        let __x = (_x + j) % DISPLAY_WIDTH;
                        let __y = (_y + i as usize) % DISPLAY_HEIGHT;
                        if pixel == 1 && self.display[__y] & (0x1 << __x) != 0 {
                            self.v[0xf] = 1;
                        }
                        self.display[__y] ^= (pixel as u64) << __x;
                    }
                }
                self.inc_pc();
                self.draw_flag = true;
            }
            0xe => {
                let key = self.v[x];
                match kk {
                    0x9e => {
                        // SKP Vx
                        if self.keypad & (0x1 << key) != 0 {
                            self.inc_pc();
                        }
                    }
                    0xa1 => {
                        // SKNP Vx
                        if self.keypad & (0x1 << key) == 0 {
                            self.inc_pc();
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
                self.inc_pc();
            }
            0xf => {
                match kk {
                    0x07 => {
                        // LD Vx, DT
                        self.v[x] = self.dt;
                    }
                    0x0a => {
                        // LD Vx, K
                        self.waiting_for_key = true;
                        self.waiting_for_key_register = x as u8;
                    }
                    0x15 => {
                        // LD DT, Vx
                        self.dt = self.v[x];
                    }
                    0x18 => {
                        // LD ST, Vx
                        self.st = self.v[x];
                    }
                    0x1e => {
                        // ADD I, Vx
                        self.i = self.i.wrapping_add(self.v[x] as u16);
                    }
                    0x29 => {
                        // LD F, Vx
                        let digit = self.v[x] & 0xf;
                        self.i = (FONTSET_START as u16) + (digit as u16 * 5);
                    }
                    0x33 => {
                        // LD B, Vx
                        let mut value = self.v[x];
                        for i in 0..3 {
                            self.memory[(self.i + i) as usize] = value % 10;
                            value /= 10;
                        }
                    }
                    0x55 => {
                        // LD [I], Vx
                        for i in 0..=x {
                            self.memory[(self.i + i as u16) as usize] = self.v[i];
                        }
                    }
                    0x65 => {
                        // LD Vx, [I]
                        for i in 0..=x {
                            self.v[i] = self.memory[(self.i + i as u16) as usize];
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
                self.inc_pc();
            }
            _ => {
                invalid_opcode!(opcode);
            }
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, scale: usize) {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for i in 0..DISPLAY_HEIGHT {
            for j in 0..DISPLAY_WIDTH {
                if self.display[i] & (0x1 << j) != 0 {
                    canvas
                        .fill_rect(Rect::new(
                            (j * scale) as i32,
                            (i * scale) as i32,
                            scale as u32,
                            scale as u32,
                        ))
                        .unwrap();
                }
            }
        }

        canvas.present();
    }
}

impl fmt::Display for C8Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C8Cpu {{\n")?;
        write!(f, "    Memory Size: {}kB\n", MEMORY_SIZE)?;
        write!(f, "    Register Count: {}\n", REGISTER_COUNT)?;
        write!(f, "    V: [ ")?;
        for i in 0..REGISTER_COUNT {
            write!(f, "0x{:x}, ", self.v[i])?;
        }
        write!(f, "]\n")?;
        write!(f, "    I: 0x{:x}\n", self.i)?;
        write!(f, "    DT: 0x{:x}\n", self.dt)?;
        write!(f, "    ST: 0x{:x}\n", self.st)?;
        write!(f, "    Stack Size: {}\n", STACK_SIZE)?;
        write!(f, "    Stack: [ ")?;
        for i in 0..STACK_SIZE {
            write!(f, "0x{:x}, ", self.stack[i])?;
        }
        write!(f, "]\n")?;
        write!(f, "    PC: 0x{:x}\n", self.pc)?;
        write!(f, "    SP: 0x{:x}\n", self.sp)?;
        write!(f, "}}")
    }
}
