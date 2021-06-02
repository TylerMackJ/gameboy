extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
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
            .window("GameBoy", 800, 600)
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

    pub fn event_loop(&mut self) {
        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
    
            self.canvas.clear();
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
            // The rest of the game loop goes here...
        }
    }
}
