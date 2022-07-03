mod chip8;
use chip8::Chip8;
use raylib::prelude::*;
use std::{env, process::exit};

const SCREEN_WIDTH: i32 = 64;
const SCREEN_HEIGHT: i32 = 32;

const MODIFIER: i32 = 10;

const DISPLAY_WIDTH: i32 = SCREEN_WIDTH * MODIFIER;
const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT * MODIFIER;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: chip8 chip8application");
        exit(1)
    }

    let mut emulator = Chip8::init();
    emulator.load_application(&args[1]);

    let (mut rl, thread) = raylib::init()
        .size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .title("Chip8 Emulator")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        emulator.emulate_cycle();
        if emulator.drawFlag {
            d.clear_background(Color::WHITE);
            for y in 0..32 as i32 {
                for x in 0..64 {
                    if emulator.gfx[((y * 64) + x) as usize] == 0 {
                        d.draw_pixel(x, y, Color::WHITE)
                    } else {
                        d.draw_pixel(x, y, Color::BLACK)
                    }
                }
            }
            emulator.drawFlag = false;
        }
    }
    //println!("Hello, world!");
}
