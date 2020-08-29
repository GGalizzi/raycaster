use pixels::{Pixels, SurfaceTexture};
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::EventPump;
use std::time::{Duration, Instant};

use crate::State;

pub struct Game {
    pixels: Pixels<Window>,
    event_pump: EventPump,
    ttf_context: sdl2::ttf::Sdl2TtfContext,
    window: Window,
}

impl Game {
    pub fn new(window_title: &str, resolution_x: i32, resolution_y: i32) -> Result<Game, String> {
        let sdl_context = sdl2::init()?;
        let video_sub = sdl_context.video()?;
        sdl_context.mouse().capture(true);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

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
            ttf_context,
            window,
        })
    }

    pub fn run<S, F>(&mut self, init: F) -> Result<(), String>
    where
        S: State,
        F: FnOnce() -> Result<S, String>,
    {
        let mut state = init()?;
        let font = self.ttf_context.load_font("assets/font.ttf", 68)?;

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

            self.pixels
                .get_frame()
                .copy_from_slice(&[65,70,67,0xff].repeat(320 * 200));
                //.copy_from_slice(&[0x00; 320 * 200 * 4]);
            state.draw(self.pixels.get_frame())?;

            {
                let buf = self.pixels.get_frame();
                let font_surface = font
                    .render(&format!("{:.0}", fps))
                    .solid((0, 0, 255, 255))
                    .map_err(|e| e.to_string())?;
                let pitch = font_surface.pitch();
                font_surface.with_lock(|data| {
                    for x in 0..font_surface.width() {
                        for y in 0..font_surface.height() {
                            let dst = buf
                                .chunks_exact_mut(4)
                                .skip(y as usize * 320)
                                .skip(x as usize)
                                .next();
                            let src = data
                                .chunks_exact(4)
                                .skip(y as usize * font_surface.width() as usize)
                                .skip(x as usize)
                                .next();

                            if let Some(dst) = dst {
                                if let Some(src) = src {
                                    if src[1] == 0 && src[2] == 0 && src[3] == 0 { continue; }
                                    dst.copy_from_slice(&[src[1], src[2], src[3], src[0]]);
                                }
                            }
                        }
                    }
                });
            }
            self.pixels.render().map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
