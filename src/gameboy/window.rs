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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
        return true
    }

    pub fn display_loop(&mut self, memory: &[u8; 0x10000]) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        let points = [Point::new(25, 25), Point::new(25, 26)];

        self.canvas.draw_points(&points[..]);
        self.canvas.present();
    }
}
