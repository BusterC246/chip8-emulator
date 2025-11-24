use crate::io;
use crate::io::scancode_to_byte;

pub struct Chip {
    pc: u16,
    sp: u8,
    i: u16,
    dt: u8,
    st: u8,
    v: [u8; 16],
    stack: [u16; 16],
    mem: [u8; 4096],
    fb: [[bool; io::DISPLAY_WIDTH as usize]; io::DISPLAY_HEIGHT as usize],
    io: io::IO,
}

impl Chip {
    pub fn new(rom: &[u8]) -> Chip {
        assert!(rom.len() <= 0xDFF + 1);

        let mut mem = [0; 4096];

        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        let font_reserve = &mut mem[0x050..0x0A0];

        font_reserve.copy_from_slice(&font);

        let instructions = &mut mem[0x200..(0x200 + rom.len())];

        instructions.copy_from_slice(rom);

        Chip {
            pc: 0x200,
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            v: [0; 16],
            stack: [0; 16],
            mem,
            fb: [[false; io::DISPLAY_WIDTH as usize]; io::DISPLAY_HEIGHT as usize],
            io: io::IO::default(),
        }
    }

    pub fn handle_events(&mut self) -> bool {
        for event in self.io.event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return false;
            }
        }
        true
    }

    pub fn fetch(&mut self) -> u16 {
        let opcode: u16 =
            ((self.mem[self.pc as usize] as u16) << 8) | self.mem[(self.pc + 1) as usize] as u16;

        self.pc += 2;

        opcode
    }

    pub fn execute(&mut self, opcode: u16) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }

        let nibble: [u8; 4] = [
            (opcode >> 12) as u8,
            (opcode >> 8 & 0x000F) as u8,
            (opcode >> 4 & 0x000F) as u8,
            (opcode & 0x000F) as u8,
        ];

        match nibble[0] {
            0 => {
                // 0x00E0
                if opcode == 0x00E0 {
                    println!("CLS");

                    for row in &mut self.fb {
                        for x in row {
                            *x = false;
                        }
                    }

                    self.io.draw_fb(&self.fb).unwrap();
                // 0x00EE
                } else if opcode == 0x00EE {
                    println!("RET");

                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
            }
            // 0x1NNN
            1 => {
                println!("JP {:X}", opcode & 0x0FFF);

                self.pc = opcode & 0x0FFF;
            }
            // 0x2NNN
            2 => {
                println!("CALL {:X}", opcode & 0x0FFF);

                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = opcode & 0x0FFF;
            }
            // 0x3XNN
            3 => {
                println!("SE V{:X} {:X}", nibble[1], opcode & 0x00FF);

                if self.v[nibble[1] as usize] == (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            // 0x4XNN
            4 => {
                println!("SNE V{:X} {:X}", nibble[1], opcode & 0x00FF);

                if self.v[nibble[1] as usize] != (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            // 0x5XY0
            5 => {
                println!("SE V{:X} V{:X}", nibble[1], nibble[2]);

                if self.v[nibble[1] as usize] == self.v[nibble[2] as usize] {
                    self.pc += 2;
                }
            }
            // 0x6XNN
            6 => {
                println!("LD V{:X} {:X}", nibble[1], opcode & 0x00FF);

                self.v[nibble[1] as usize] = (opcode & 0x00FF) as u8;
            }
            // 0x7XNN
            7 => {
                println!("ADD V{:X} {:X}", nibble[1], opcode & 0x00FF);

                self.v[nibble[1] as usize] =
                    self.v[nibble[1] as usize].wrapping_add((opcode & 0x00FF) as u8);
            }
            8 => {
                match nibble[3] {
                    // 0x8XY0
                    0 => {
                        println!("LD V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[nibble[1] as usize] = self.v[nibble[2] as usize];
                    }
                    // 0x8XY1
                    1 => {
                        println!("OR V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[nibble[1] as usize] |= self.v[nibble[2] as usize];
                    }
                    // 0x8XY2
                    2 => {
                        println!("AND V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[nibble[1] as usize] &= self.v[nibble[2] as usize];
                    }
                    // 0x8XY3
                    3 => {
                        println!("XOR V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[nibble[1] as usize] ^= self.v[nibble[2] as usize];
                    }
                    // 0x8XY4
                    4 => {
                        println!("ADD V{:X} V{:X}", nibble[1], nibble[2]);

                        let sum =
                            self.v[nibble[1] as usize].overflowing_add(self.v[nibble[2] as usize]);
                        self.v[nibble[1] as usize] = sum.0;
                        self.v[0xF] = sum.1 as u8;
                    }
                    // 0x8XY5
                    5 => {
                        println!("SUB V{:X} V{:X}", nibble[1], nibble[2]);

                        let diff =
                            self.v[nibble[1] as usize].overflowing_sub(self.v[nibble[2] as usize]);
                        self.v[nibble[1] as usize] = diff.0;
                        self.v[0xF] = (!diff.1) as u8;
                    }
                    // 0x8XY6
                    6 => {
                        println!("SHR V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[0xF] = self.v[nibble[1] as usize] & 1;
                        self.v[nibble[1] as usize] >>= 1;
                    }
                    // 0x8XY7
                    7 => {
                        println!("SUBN V{:X} V{:X}", nibble[1], nibble[2]);

                        let diff =
                            self.v[nibble[2] as usize].overflowing_sub(self.v[nibble[1] as usize]);
                        self.v[nibble[1] as usize] = diff.0;
                        self.v[0xF] = (!diff.1) as u8;
                    }
                    // 0x8XYE
                    0xE => {
                        println!("SHL V{:X} V{:X}", nibble[1], nibble[2]);

                        self.v[0xF] = self.v[nibble[1] as usize] >> 7;
                        self.v[nibble[1] as usize] <<= 1;
                    }
                    _ => {}
                }
            }
            // 0x9XY0
            9 => {
                println!("SNE V{:X} V{:X}", nibble[1], nibble[2]);

                if self.v[nibble[1] as usize] != self.v[nibble[2] as usize] {
                    self.pc += 2;
                }
            }
            // 0xANNN
            0xA => {
                println!("LD I {:X}", opcode & 0x0FFF);

                self.i = opcode & 0x0FFF;
            }
            // 0xBNNN
            0xB => {
                println!("JP V0 {:X}", opcode & 0x0FFF);

                self.pc = self.v[0] as u16 + (opcode & 0x0FFF)
            }
            // 0xCXNN
            0xC => {
                println!("RND V{:X} {:X}", nibble[1], opcode & 0x00FF);

                self.v[nibble[1] as usize] = (((self.pc as u8)
                    .wrapping_mul(137)
                    .wrapping_add(self.v[0])
                    .wrapping_add(self.v[1])
                    .wrapping_add(self.dt)
                    .wrapping_add(self.st)) as u8)
                    & (opcode as u8 & 0x00FF);
            }
            // 0xDXYN
            0xD => {
                println!("DRW V{:X} V{:X} {:X}", nibble[1], nibble[2], nibble[3]);

                self.v[0xF] = 0;

                for (y_offset, byte) in self.mem
                    [self.i as usize..((self.i.wrapping_add(nibble[3] as u16)) as usize)]
                    .iter()
                    .enumerate()
                {
                    for bit in 0..8 {
                        if (byte >> (7 - bit)) & 1 == 1 {
                            let x =
                                (self.v[nibble[1] as usize].wrapping_add(bit)) % io::DISPLAY_WIDTH;
                            let y = (self.v[nibble[2] as usize].wrapping_add(y_offset as u8))
                                % io::DISPLAY_HEIGHT;

                            if self.fb[y as usize][x as usize] {
                                self.v[0xF] = 1;
                            }

                            self.fb[y as usize][x as usize] = !self.fb[y as usize][x as usize];
                        }
                    }
                }

                self.io.draw_fb(&self.fb).unwrap();
            }
            0xE => {
                // 0xEX9E
                if nibble[3] == 0xE {
                    println!("SKP V{:X}", nibble[1]);

                    if self.io.event_pump.keyboard_state().is_scancode_pressed(
                        io::byte_to_scancode(self.v[nibble[1] as usize]).unwrap(),
                    ) {
                        self.pc += 2;
                    }
                    // 0xEXA1
                } else {
                    println!("SKNP V{:X}", nibble[1]);

                    if !self.io.event_pump.keyboard_state().is_scancode_pressed(
                        io::byte_to_scancode(self.v[nibble[1] as usize]).unwrap(),
                    ) {
                        self.pc += 2;
                    }
                }
            }
            0xF => {
                match (opcode & 0x00FF) as u8 {
                    // 0xFX07
                    0x07 => {
                        println!("LD V{:X} DT", nibble[1]);

                        self.v[nibble[1] as usize] = self.dt;
                    }
                    // 0xFX0A
                    0x0A => {
                        println!("LD V{:X} K", nibble[1]);

                        let keyboard = self.io.event_pump.keyboard_state();
                        let pressed = keyboard.pressed_scancodes().find_map(scancode_to_byte);

                        if let Some(key) = pressed {
                            self.v[nibble[1] as usize] = key;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    // 0xFX15
                    0x15 => {
                        println!("LD DT V{:X}", nibble[1]);

                        self.dt = self.v[nibble[1] as usize];
                    }
                    // 0xFX18
                    0x18 => {
                        println!("LD ST V{:X}", nibble[1]);

                        self.st = self.v[nibble[1] as usize];
                    }
                    // 0xFX1E
                    0x1E => {
                        println!("ADD I V{:X}", nibble[1]);

                        self.i = self.i.wrapping_add(self.v[nibble[1] as usize] as u16);
                    }
                    // 0xFX29
                    0x29 => {
                        println!("LD F V{:X}", nibble[1]);

                        self.i = self.v[nibble[1] as usize] as u16 * 5 + 0x050;
                    }
                    // 0xFX33
                    0x33 => {
                        println!("LD B V{:X}", nibble[1]);

                        self.mem[self.i as usize] = self.v[nibble[1] as usize] / 100;
                        self.mem[self.i as usize + 1] = self.v[nibble[1] as usize] / 10 % 10;
                        self.mem[self.i as usize + 2] = self.v[nibble[1] as usize] % 10;
                    }
                    // 0xFX55
                    0x55 => {
                        println!("LD [I] V{:X}", nibble[1]);

                        let registers = &self.v[..=(nibble[1] as usize)];
                        let mem_region = &mut self.mem
                            [(self.i as usize)..((self.i + nibble[1] as u16 + 1) as usize)];

                        mem_region.copy_from_slice(registers);
                    }
                    // 0xFX65
                    0x65 => {
                        println!("LD V{:X} [I]", nibble[1]);

                        let registers = &mut self.v[..=(nibble[1] as usize)];
                        let mem_region = &self.mem
                            [(self.i as usize)..((self.i + nibble[1] as u16 + 1) as usize)];

                        registers.copy_from_slice(mem_region);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
