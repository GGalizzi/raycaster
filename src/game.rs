use pixels::{Pixels, SurfaceTexture};
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::EventPump;

use crate::State;

pub struct Game {
    pixels: Pixels<Window>,
    event_pump: EventPump,
    window: Window,
}

impl Game {
    pub fn new(window_title: &str, resolution_x: i32, resolution_y: i32) -> Result<Game, String> {
        let sdl_context = sdl2::init()?;
        let video_sub = sdl_context.video()?;
        sdl_context.mouse().capture(true);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let window = video_sub
            .window(window_title, 320, 200)
            .position_centered()
            .vulkan()
            .build()
            .map_err(|e| e.to_string())?;

        let surface_texture = SurfaceTexture::new(320, 200, &window);
        let pixels = Pixels::new(320, 200, surface_texture).map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        Ok(Game {
            pixels,
            event_pump,
            window,
        })
    }

    pub fn run<S, F>(&mut self, init: F) -> Result<(), String>
    where
        S: State,
        F: FnOnce() -> Result<S, String>,
    {
        let mut state = init()?;

        'game: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'game;
                    }
                    _ => {
                        state.event(event)?;
                    }
                }
            }

            state.update()?;

            self.pixels.get_frame().copy_from_slice(&[0x00; 320*200*4]);
            state.draw(self.pixels.get_frame())?;
            self.pixels.render().map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
