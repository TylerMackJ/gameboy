extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::Duration;

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
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        for background_tile_x in 0..5 {
            for background_tile_y in 0..5 {
                let background_tile: *const u8 = &memory[0x9800 + background_tile_x + (background_tile_y * 32)];
                for bit in (0..128).step_by(2) {
                    let byte = bit / 8;
                    let pixel: u8 = unsafe { (*background_tile.offset(byte) >> (6 - (bit % 8)) & 0x03) };
                    let row: i32 = bit as i32 / 16;
                    let col: i32 = bit as i32 % 16;
                    //println!("{} ({}, {})", pixel, row, col);
                    self.canvas.draw_point(Point::new(col, row));
                }
            }
        }
        self.canvas.present();
    }
}
