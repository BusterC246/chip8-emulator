use sdl2::keyboard::Scancode;
use sdl2::pixels;

pub const DISPLAY_WIDTH: u8 = 128;
pub const DISPLAY_HEIGHT: u8 = 64;

pub struct IO {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::WindowCanvas,
    pub event_pump: sdl2::EventPump,
}

impl IO {
    pub fn draw_fb(
        &mut self,
        fb: &[[bool; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
    ) -> Result<(), String> {
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        let mut pixels: Vec<sdl2::rect::Rect> = Vec::new();

        for (y, row) in fb.iter().enumerate() {
            for (x, draw) in row.iter().enumerate() {
                if *draw {
                    pixels.push(sdl2::rect::Rect::new(x as i32 * 8, y as i32 * 8, 8, 8));
                }
            }
        }

        self.canvas
            .set_draw_color(pixels::Color::RGB(255, 255, 255));
        self.canvas.fill_rects(&pixels)?;
        self.canvas.present();

        Ok(())
    }
}

impl Default for IO {
    fn default() -> IO {
        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window("chip8", DISPLAY_WIDTH as u32 * 8, DISPLAY_HEIGHT as u32 * 8)
            .position_centered()
            .vulkan()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

        IO {
            sdl_context,
            canvas,
            event_pump,
        }
    }
}

pub fn byte_to_scancode(byte: u8) -> Option<Scancode> {
    match byte {
        0x0 => Some(Scancode::X),
        0x1 => Some(Scancode::Num1),
        0x2 => Some(Scancode::Num2),
        0x3 => Some(Scancode::Num3),
        0x4 => Some(Scancode::Q),
        0x5 => Some(Scancode::W),
        0x6 => Some(Scancode::E),
        0x7 => Some(Scancode::A),
        0x8 => Some(Scancode::S),
        0x9 => Some(Scancode::D),
        0xA => Some(Scancode::Z),
        0xB => Some(Scancode::C),
        0xC => Some(Scancode::Num4),
        0xD => Some(Scancode::R),
        0xE => Some(Scancode::F),
        0xF => Some(Scancode::V),
        _ => None,
    }
}

pub fn scancode_to_byte(scancode: Scancode) -> Option<u8> {
    match scancode {
        Scancode::X => Some(0x0),
        Scancode::Num1 => Some(0x1),
        Scancode::Num2 => Some(0x2),
        Scancode::Num3 => Some(0x3),
        Scancode::Q => Some(0x4),
        Scancode::W => Some(0x5),
        Scancode::E => Some(0x6),
        Scancode::A => Some(0x7),
        Scancode::S => Some(0x8),
        Scancode::D => Some(0x9),
        Scancode::Z => Some(0xA),
        Scancode::C => Some(0xB),
        Scancode::Num4 => Some(0xC),
        Scancode::R => Some(0xD),
        Scancode::F => Some(0xE),
        Scancode::V => Some(0xF),
        _ => None,
    }
}
