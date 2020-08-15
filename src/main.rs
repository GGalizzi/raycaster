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

    let resulting_resolution = (640, 400);
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

    let mut fov: f32 = 60.0;

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
                Event::KeyDown { keycode, .. } if keycode.unwrap() == Keycode::Q => {
                    fov -= 5.0;
                }
                Event::KeyDown { keycode, .. } if keycode.unwrap() == Keycode::E => {
                    fov += 5.0;
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
            let mut ray_rotation = game_plugin::Rotation::new(rotation.degrees - fov / 2.0);
            for x in 0..resulting_resolution.0 {
                let camera_x = 2.0 * x as f32 / (resulting_resolution.0 as f32) - 1.0;
                let direction = rotation.direction() * (fov / 100.0);
                let plane = game_plugin::Rotation::new(rotation.degrees + 90.0).direction();
                let ray_direction = game_plugin::Direction::new(
                    direction.x + plane.x * camera_x,
                    direction.y + plane.y * camera_x,
                );

                /*
                let ray_direction = ray_rotation.direction();
                ray_rotation.degrees += angle_between_rays;
                */

                let mut grid_position = (position.x as i32, position.y as i32);

                // Length of ray from current position to next x/y side
                let mut side_distance = Vec2::new(0.0, 0.0);

                // Length of ray from x/y to next x/y side
                let delta_distance =
                    Vec2::new((1.0 / ray_direction.x).abs(), (1.0 / ray_direction.y).abs());

                // Should we step negative, or positive? (-1,+1)
                let mut step = (0, 0);

                let mut hit = false;

                // Which side was hit
                let mut side_hit = 0;

                if ray_direction.x < 0.0 {
                    step.0 = -1;
                    side_distance.set_x((position.x - grid_position.0 as f32) * delta_distance.x());
                } else {
                    step.0 = 1;
                    side_distance
                        .set_x((grid_position.0 as f32 + 1.0 - position.x) * delta_distance.x());
                }

                if ray_direction.y < 0.0 {
                    step.1 = -1;
                    side_distance.set_y((position.y - grid_position.1 as f32) * delta_distance.y());
                } else {
                    step.1 = 1;
                    side_distance
                        .set_y((grid_position.1 as f32 + 1.0 - position.y) * delta_distance.y());
                }

                let mut traveled = 0;
                // Cast the ray
                while !hit && traveled < 1000 {
                    if side_distance.x() < side_distance.y() {
                        side_distance.set_x(side_distance.x() + delta_distance.x());
                        grid_position.0 += step.0;
                        side_hit = 0;
                    } else {
                        side_distance.set_y(side_distance.y() + delta_distance.y());
                        grid_position.1 += step.1;
                        side_hit = 1;
                    }

                    // TODO: Check with an actual map
                    if grid_position.0 == 0
                        || grid_position.1 == 0
                        || (grid_position.0 == 20 && grid_position.1 == 20)
                    {
                        hit = true;
                    }
                    traveled += 1;
                }

                if !hit {
                    continue;
                }

                let distance_to_wall = if side_hit == 0 {
                    (grid_position.0 as f32 - position.x + (1. - step.0 as f32) / 2.0)
                        / ray_direction.x
                } else {
                    (grid_position.1 as f32 - position.y + (1. - step.1 as f32) / 2.0)
                        / ray_direction.y
                };

                let line_height = (resulting_resolution.1 as f32 / distance_to_wall) as i32;

                let draw_start = -line_height / 2 + resulting_resolution.1 / 2;
                let draw_end = line_height / 2 + resulting_resolution.1 / 2;

                let mut mult = -(0.1 * std::cmp::min(distance_to_wall as i32, 1000) as f32) + 2.5;
                if mult <= 0.8 {
                    mult = 0.8;
                }
                canvas.set_draw_color(if side_hit == 0 {
                    ((65. * mult) as u8, (65. * mult) as u8, (65. * mult) as u8)
                } else {
                    let mult = mult * 1.05;
                    ((44. * mult) as u8, (44. * mult) as u8, (44. * mult) as u8)
                });
                canvas.draw_line((x, draw_start), (x, draw_end))?;
            }

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
