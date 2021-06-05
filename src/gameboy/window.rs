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

        for background_tile_x in 0..5 {
            for background_tile_y in 0..5 {
                let background_tile_address: u8 = memory[0x9800 + background_tile_x + (background_tile_y * 32)];
                for background_tile_offset in 0..16 {
                    for background_tile_byte_offset in (0..8).step_by(2) {
                        let pixel_color = memory[(0x8000 as usize + background_tile_address as usize + background_tile_offset) as usize];
                        if pixel_color != 0x00 {
                            panic!("Colored pixel found");
                        }
                        //println!("{}", pixel_color);
                    }
                }
            }
        }
        self.canvas.present();
    }
}
