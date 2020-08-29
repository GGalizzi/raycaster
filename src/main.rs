use bevy::prelude::App;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod base_plugin;
mod font;
mod game;
mod game_plugin;
mod raycaster;
mod texture;

use base_plugin::BasePlugin;
use game::Game;
use game_plugin::{GamePlugin, Player, Position};
use raycaster::raycast;
use texture::Texture;

pub const TILE_SIZE: i32 = 12;

const resulting_resolution: (i32, i32) = (320, 200);
const actual_resolution: (u32, u32) = (640, 400);

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

struct GameState {
    bevy: App,
    wall_texture: Texture,
    floor_texture: Texture,
    fps: f64,
}

impl GameState {
    pub fn new() -> Result<GameState, String> {
        //time::set_timestep(context, time::Timestep::Variable);
        let keypress = Keypress::new();
        let mouse_motion = MouseMotion::new();
        let mut bevy = std::mem::replace(
            &mut App::build()
                .add_plugin(BasePlugin)
                .add_plugin(GamePlugin)
                .add_resource(keypress)
                .add_resource(mouse_motion)
                .app,
            App::default(),
        );

        bevy.startup_schedule.initialize(&mut bevy.resources);
        bevy.startup_executor.run(
            &mut bevy.startup_schedule,
            &mut bevy.world,
            &mut bevy.resources,
        );

        let wall_texture = Texture::new("assets/stone_wall_b.png");
        let floor_texture = Texture::new("assets/stone_floor_c.png");
        //let canvas = Canvas::new(context, resulting_resolution.0, resulting_resolution.1).unwrap();

        Ok(GameState {
            bevy,
            wall_texture,
            floor_texture,
            fps: 0.0,
        })
    }
}

pub trait State {
    fn update(&mut self) -> Result<(), String>;
    fn draw(&mut self, buf: &mut [u8]) -> Result<(), String>;
    fn event(&mut self, event: Event) -> Result<(), String>;
}

impl State for GameState {
    fn update(&mut self) -> Result<(), String> {
        self.bevy.update();

        {
            let mut mm = self.bevy.resources.get_mut::<MouseMotion>().unwrap();
            mm.clear();
        }
        Ok(())
    }

    fn draw(&mut self, buf: &mut [u8]) -> Result<(), String> {
        let fov = 66;

        /*
        let fps = graphics::text::Text::new(
            format!("{}", self.fps),
            graphics::text::Font::vector(ctx, "assets/font.ttf", 8.0)?,
        );

        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, graphics::Color::rgb(0.1568, 0.1746, 0.1568));
        */

        for (position, _, rotation) in self
            .bevy
            .world
            .query::<(&Position, &Player, &game_plugin::Rotation)>()
            .iter()
        {
            buf.chunks_exact_mut(4)
                .nth(320 * position.y as usize + position.x as usize)
                .unwrap()
                .copy_from_slice(&[0x5e, 0x48, 0xe8, 0xff]);

            raycast(
                resulting_resolution,
                fov,
                position,
                rotation,
                buf,
                &self.wall_texture,
                &self.floor_texture,
            )
            .expect("Failed raycasting");

            /*
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
            )?;*/
        }

        /*
        graphics::reset_canvas(ctx);
        graphics::draw(ctx, &self.scaler, DrawParams::new());
        graphics::draw(ctx, &fps, Vec2::new(5.0, 50.0));
        */
        Ok(())
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::KeyDown { keycode, .. } => {
                if let Some(kc) = keycode {
                    let mut kp = self.bevy.resources.get_mut::<Keypress>().unwrap();
                    kp.set(kc);
                }
            }
            Event::KeyUp { keycode, .. } => {
                if let Some(kc) = keycode {
                    let mut kp = self.bevy.resources.get_mut::<Keypress>().unwrap();
                    kp.clear();
                }
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                let mut mm = self.bevy.resources.get_mut::<MouseMotion>().unwrap();
                mm.set(xrel, yrel);
            }
            _ => {}
        };

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut game = Game::new("tetra + bevy", actual_resolution.0, actual_resolution.1)?;

    game.run(GameState::new)?;

    //let mut texture = texture_creator.load_texture("assets/stone_wall.png")?;
    //let mut floor_texture = texture_creator.load_texture("assets/stone_floor.png")?;
    Ok(())
}
