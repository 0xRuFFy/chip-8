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

    let rom = fs::read("ch8/pong.ch8").expect("Unable to read file");
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
                Event::KeyDown { keycode: Some(Keycode::Num0), .. } => {
                    cpu.set_key(0x0);
                },
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    cpu.set_key(0x1);
                },
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    cpu.set_key(0x2);
                },
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                    cpu.set_key(0x3);
                },
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                    cpu.set_key(0x4);
                },
                Event::KeyDown { keycode: Some(Keycode::Num5), .. } => {
                    cpu.set_key(0x5);
                },
                Event::KeyDown { keycode: Some(Keycode::Num6), .. } => {
                    cpu.set_key(0x6);
                },
                Event::KeyDown { keycode: Some(Keycode::Num7), .. } => {
                    cpu.set_key(0x7);
                },
                Event::KeyDown { keycode: Some(Keycode::Num8), .. } => {
                    cpu.set_key(0x8);
                },
                Event::KeyDown { keycode: Some(Keycode::Num9), .. } => {
                    cpu.set_key(0x9);
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    cpu.set_key(0xA);
                },
                Event::KeyDown { keycode: Some(Keycode::B), .. } => {
                    cpu.set_key(0xB);
                },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    cpu.set_key(0xC);
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    cpu.set_key(0xD);
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    cpu.set_key(0xE);
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    cpu.set_key(0xF);
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
