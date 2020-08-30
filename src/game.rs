use pixels::{
    wgpu::{PowerPreference, RequestAdapterOptions},
    Pixels, PixelsBuilder, SurfaceTexture,
};
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::EventPump;
use std::time::Instant;

use crate::font::Font;
use crate::State;

pub struct Game {
    pixels: Pixels<Window>,
    event_pump: EventPump,
    font_manager: Font,
    _window: Window,
}

impl Game {
    pub fn new(window_title: &str, resolution_x: u32, resolution_y: u32) -> Result<Game, String> {
        let sdl_context = sdl2::init()?;
        let video_sub = sdl_context.video()?;
        sdl_context.mouse().capture(true);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let font_manager = Font::new(ttf_context);

        let window = video_sub
            .window(window_title, resolution_x, resolution_y)
            .position_centered()
            .vulkan()
            .build()
            .map_err(|e| e.to_string())?;

        let surface_texture = SurfaceTexture::new(resolution_x, resolution_y, &window);
        //let pixels = Pixels::new(320, 200, surface_texture).map_err(|e| e.to_string())?;
        let pixels = PixelsBuilder::new(320, 200, surface_texture)
            .request_adapter_options(RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .enable_vsync(false)
            .build()
            .map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        Ok(Game {
            pixels,
            event_pump,
            font_manager,
            _window: window,
        })
    }

    pub fn run<S, F>(&mut self, init: F) -> Result<(), String>
    where
        S: State,
        F: FnOnce() -> Result<S, String>,
    {
        let mut state = init()?;
        // let font = self.ttf_context.load_font("assets/font.ttf", 18)?;
        let font = self.font_manager.build("assets/font.ttf", 18)?;

        let mut last = Instant::now();
        'game: loop {
            let dt = Instant::now().duration_since(last).as_secs_f64();
            let fps = 1.0 / dt;
            last = Instant::now();
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

            // Clear
            self.pixels
                .get_frame()
                .copy_from_slice(&[0, 0, 0, 0xff].repeat(320 * 200));

            state.draw(self.pixels.get_frame())?;

            // font.draw(&format!("{:.0}", fps), self.pixels.get_frame())?;

            self.pixels.render().map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
