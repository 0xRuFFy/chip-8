mod c8_cpu;

// extern crate sdl2;

use c8_cpu::C8Cpu;
use std::fs;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const SCALE: u32 = 10;

fn main() {

    let rom = fs::read("ch8/tictactoe.ch8").expect("Unable to read file");
    println!("rom size: {}b", rom.len());
    
    let mut cpu = C8Cpu::new();
    cpu.load_rom(rom);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip-8", 64 * SCALE, 32 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        cpu.single_cycle();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        if cpu.draw_flag {
            cpu.render(&mut canvas, SCALE as usize)
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
