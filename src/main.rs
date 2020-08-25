extern crate tetra;

use bevy::prelude::*;

use tetra::{
    time,
    graphics,
    graphics::{
        scaling::{ScalingMode, ScreenScaler},
        Canvas, DrawParams, Texture,
    },
    input::Key,
    Context, ContextBuilder, Event, Result, State,
};

mod base_plugin;
mod game_plugin;
mod raycaster;

use base_plugin::BasePlugin;
use game_plugin::{GamePlugin, Player, Position};
use raycaster::raycast;

pub const TILE_SIZE: i32 = 12;

const resulting_resolution: (i32, i32) = (320, 200);
const actual_resolution: (i32, i32) = (1080, 768);

const scale: (f32, f32) = (
    actual_resolution.0 as f32 / resulting_resolution.0 as f32,
    actual_resolution.1 as f32 / resulting_resolution.1 as f32,
);

#[derive(Debug)]
pub struct MouseMotion {
    x: i32,
    y: i32,
}

impl MouseMotion {
    pub fn new() -> MouseMotion {
        MouseMotion { x: 0, y: 0 }
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x as i32;
        self.y = y as i32;
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

pub struct Keypress {
    val: Option<Key>,
}

impl Keypress {
    pub fn new() -> Keypress {
        Keypress { val: None }
    }

    fn set(&mut self, ch: Key) {
        self.val = Some(ch);
    }

    fn is(&self, ch: Key) -> bool {
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
    scaler: ScreenScaler,
}

impl GameState {
    pub fn new(context: &mut Context) -> Result<GameState> {
        time::set_timestep(context, time::Timestep::Variable);
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

        let wall_texture = Texture::new(context, "assets/stone_wall.png")?;
        let floor_texture = Texture::new(context, "assets/stone_floor.png")?;
        //let canvas = Canvas::new(context, resulting_resolution.0, resulting_resolution.1).unwrap();

        let scaler = ScreenScaler::with_window_size(
            context,
            resulting_resolution.0,
            resulting_resolution.1,
            ScalingMode::ShowAll,
        )?;

        Ok(GameState {
            bevy,
            wall_texture,
            floor_texture,
            scaler,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> Result {
        let _fps = time::get_fps(ctx);
        

        self.bevy.update();

        {
            let mut mm = self.bevy.resources.get_mut::<MouseMotion>().unwrap();
            mm.clear();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
        let fov = 66;

        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, graphics::Color::rgb(0.2, 0., 0.5));

        for (position, _, rotation) in self
            .bevy
            .world
            .query::<(&Position, &Player, &game_plugin::Rotation)>()
            .iter()
        {
            raycast(
                resulting_resolution,
                fov,
                position,
                rotation,
                ctx,
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
        graphics::reset_canvas(ctx);
        graphics::draw(ctx, &self.scaler, DrawParams::new());
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> Result {
        match event {
            Event::KeyPressed { key } => {
                let mut kp = self.bevy.resources.get_mut::<Keypress>().unwrap();
                kp.set(key);
            }
            Event::KeyReleased { .. } => {
                let mut kp = self.bevy.resources.get_mut::<Keypress>().unwrap();
                kp.clear();
            }
            Event::MouseMoved {
                relative_position: tetra::math::Vec2 { x, y },
                ..
            } => {
                let mut mm = self.bevy.resources.get_mut::<MouseMotion>().unwrap();
                mm.set(x, y);
            }
            _ => {}
        };

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("tetra + bevy", actual_resolution.0, actual_resolution.1)
        .grab_mouse(true)
        .relative_mouse(true)
        .vsync(false)
        .build()?
        .run(GameState::new)?;

    //let mut texture = texture_creator.load_texture("assets/stone_wall.png")?;
    //let mut floor_texture = texture_creator.load_texture("assets/stone_floor.png")?;
    Ok(())
}
