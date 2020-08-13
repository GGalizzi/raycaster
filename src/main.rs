extern crate sdl2;

use bevy::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};

mod base_plugin;
mod game_plugin;

use self::base_plugin::BasePlugin;
use self::game_plugin::{GamePlugin, Player, Position};

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

    let keypress = Keypress::new();

    let mut event_pump = sdl_context.event_pump()?;

    let mut app = std::mem::replace(
        &mut App::build()
            .add_plugin(BasePlugin)
            .add_plugin(GamePlugin)
            .add_resource(keypress)
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
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'game;
                    }
                    Event::KeyDown { keycode, .. } => {
                        if let Some(kc) = keycode {
                            kp.set(kc);
                        }
                    }
                    _ => {}
                }
            }
        }

        canvas.set_draw_color((255, 0, 0));
        for (position, _) in app.world.query::<(&Position, &Player)>().iter() {
            canvas.draw_point((position.x, position.y))?;
        }
        canvas.present();

        app.update();
    }

    Ok(())
}
