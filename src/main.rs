extern crate sdl2;

use bevy::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};

mod base_plugin;
mod game_plugin;

use self::base_plugin::BasePlugin;
use self::game_plugin::{GamePlugin, Player, Position};

#[derive(Debug)]
pub struct MouseMotion {
    x: i32,
    y: i32,
}

impl MouseMotion {
    pub fn new() -> MouseMotion {
        MouseMotion { x: 0, y: 0 }
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

pub struct Keypress {
    val: Option<Keycode>,
}

impl Keypress {
    pub fn new() -> Keypress {
        Keypress { val: None }
    }

    fn set(&mut self, ch: Keycode) {
        self.val = Some(ch);
    }

    fn is(&self, ch: Keycode) -> bool {
        if let Some(cur_ch) = &self.val {
            return *cur_ch == ch;
        }
        false
    }

    fn clear(&mut self) {
        self.val = None;
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_sub = sdl_context.video()?;
    let window = video_sub
        .window("sdl2+bevy demo", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color((0, 0, 0));
    canvas.clear();
    canvas.present();
    
    sdl_context.mouse().capture(true);

    let keypress = Keypress::new();
    let mouse_motion = MouseMotion::new();

    let mut event_pump = sdl_context.event_pump()?;

    let mut app = std::mem::replace(
        &mut App::build()
            .add_plugin(BasePlugin)
            .add_plugin(GamePlugin)
            .add_resource(keypress)
            .add_resource(mouse_motion)
            .app,
        App::default(),
    );

    app.startup_schedule.initialize(&mut app.resources);
    app.startup_executor.run(
        &mut app.startup_schedule,
        &mut app.world,
        &mut app.resources,
    );

    'game: loop {
        {
            let mut kp = app.resources.get_mut::<Keypress>().unwrap();
            kp.clear();
        }

        {
            let mut mm = app.resources.get_mut::<MouseMotion>().unwrap();
            mm.clear();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game;
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    let mut mm = app.resources.get_mut::<MouseMotion>().unwrap();
                    mm.set(xrel, yrel);
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(kc) = keycode {
                        {
                            let mut kp = app.resources.get_mut::<Keypress>().unwrap();
                            kp.set(kc);
                        }
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color((255, 0, 0));
        for (position, _) in app.world.query::<(&Position, &Player)>().iter() {
            canvas.draw_point((position.x as i32, position.y as i32))?;
        }
        canvas.present();

        app.update();
    }

    Ok(())
}
