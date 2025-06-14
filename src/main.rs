extern crate sdl2;

pub mod chip;
pub mod io;

use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

fn main() {
    // let file_name = env::args().next().unwrap();
    let rom = fs::read("roms/ibm_logo.ch8").unwrap();

    let mut chip8 = chip::Chip::new(&rom);

    loop {
        let opcode = chip8.fetch();
        chip8.execute(opcode);
        thread::sleep(Duration::from_millis(2))
    }
}
