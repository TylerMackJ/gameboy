extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
//use std::time::Duration;

pub struct SdlWindow {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
}

impl SdlWindow {
    pub fn new() -> Result<SdlWindow, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
    
        let window = video_subsystem
            .window("Gameboy", 160, 144)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
    
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump()?;

        Ok(SdlWindow {
            canvas: canvas,
            event_pump: event_pump,
        })
    }

    pub fn event_loop(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,
                _ => {}
            }
        }
        return true
    }

    pub fn display_loop(&mut self, memory: &[u8; 0x10000]) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();


        for tile_x in 0..32u32 {
            for tile_y in 0..32u32 {
                //let tile_number: u32 = tile_x + (tile_y * 32);
                let tile_number: u32 = memory[(0x9800 + tile_x + (tile_y * 32)) as usize] as u32;
                for byte_offset in (0..16u32).step_by(2) {
                    let lsb: u8 = memory[(0x8000 + tile_number + byte_offset) as usize];
                    let msb: u8 = memory[(0x8000 + tile_number + byte_offset + 1) as usize];

                    for bit_offset in 0..8u32 {
                        let color_data: u8 = ((lsb >> (7 - bit_offset)) & 0x01) + (((msb >> (7 - bit_offset)) & 0x01) << 1);

                        match color_data {
                            0b00 => self.canvas.set_draw_color(Color::RGB(0, 0, 0)),
                            0b01 => self.canvas.set_draw_color(Color::RGB(86, 86, 86)),
                            0b10 => self.canvas.set_draw_color(Color::RGB(172, 172, 172)),
                            0b11 => self.canvas.set_draw_color(Color::RGB(255, 255, 255)),
                            _ => panic!("Color data incorrect {:08b}", color_data),
                        }

                        self.canvas.draw_point(Point::new(((tile_x * 8u32) + bit_offset) as i32, ((tile_y * 8u32) + (byte_offset / 2u32)) as i32));
                    }
                }
            }
        }
        self.canvas.present();
    }
}
