mod c8_cpu;

use c8_cpu::C8Cpu;
use std::fs;

fn main() {
    let rom = fs::read("ch8/test_opcode.ch8").expect("Unable to read file");
    println!("rom size: {}b", rom.len());

    let mut cpu = C8Cpu::new();
    cpu.load_rom(rom);

    println!("{}", cpu);
    cpu.run()
}
