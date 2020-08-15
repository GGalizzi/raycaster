extern crate sdl2;

use bevy::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};

mod base_plugin;
mod game_plugin;
mod raycaster;

use base_plugin::BasePlugin;
use game_plugin::{GamePlugin, Player, Position};
use raycaster::raycast;

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

    let resulting_resolution = (320, 200);
    let actual_resolution = (960, 600);
    let scale = (
        actual_resolution.0 as f32 / resulting_resolution.0 as f32,
        actual_resolution.1 as f32 / resulting_resolution.1 as f32,
    );
    let plane: (f32, f32) = (0., 0.66);
    let window = video_sub
        .window("sdl2+bevy demo", actual_resolution.0, actual_resolution.1)
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
    canvas.set_scale(scale.0, scale.1)?;

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

        let mut angle_mod = 0.0;
    'game: loop {
        {
            let mut kp = app.resources.get_mut::<Keypress>().unwrap();
            kp.clear();
        }

        {
            let mut mm = app.resources.get_mut::<MouseMotion>().unwrap();
            mm.clear();
        }
        let mut fov = 66;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game;
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    let mut mm = app.resources.get_mut::<MouseMotion>().unwrap();
                    mm.set(xrel, yrel);
                }
                Event::KeyDown { keycode, .. } if keycode.unwrap() == Keycode::Q => {
                    angle_mod -= 0.00025;
                }
                Event::KeyDown { keycode, .. } if keycode.unwrap() == Keycode::E => {
                    angle_mod += 0.00025;
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

        canvas.set_draw_color((15, 15, 25));
        canvas.clear();

        /*
        let distance_to_plane = (resulting_resolution.0 / 2) / (fov / 2.0).tan() as i32;
        let angle_between_rays = fov / resulting_resolution.0 as f32;
        */
        for (position, _, rotation) in app
            .world
            .query::<(&Position, &Player, &game_plugin::Rotation)>()
            .iter()
        {
            raycast(resulting_resolution, fov, position, rotation, &mut canvas, angle_mod);

            canvas.set_draw_color((185, 66, 66));
            canvas.draw_point((position.x as i32, position.y as i32))?;
            canvas.set_draw_color((66, 76, 222));
            let direction = rotation.direction();
            let dv = Vec2::new(direction.x, direction.y) * 2.5;
            let view_point_end = (
                position.x as i32 + dv.x() as i32,
                position.y as i32 + dv.y() as i32,
            );
            canvas.draw_line(
                (position.x as i32, position.y as i32),
                (view_point_end.0, view_point_end.1),
            )?;
        }
        canvas.present();

        app.update();
    }

    Ok(())
}
