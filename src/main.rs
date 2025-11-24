extern crate sdl2;

pub mod chip;
pub mod io;

use std::env;
use std::fs;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_name = if args.len() > 1 {
        &args[1]
    } else {
        "roms/1-chip8-logo.ch8"
    };

    let rom = fs::read(file_name).expect("Failed to read ROM file");
    let mut chip8 = chip::Chip::new(&rom);

    'running: loop {
        if !chip8.handle_events() {
            break 'running;
        }

        for _ in 0..12 {
            let opcode = chip8.fetch();
            chip8.execute(opcode);
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
