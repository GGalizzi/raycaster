use glam::vec2;
use bevy::prelude::*;
use sdl2::keyboard::Keycode;

use crate::{Keypress, MouseMotion};

pub struct GamePlugin;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Direction {
    pub x: f32,
    pub y: f32,
}

impl Direction {
    pub fn new(x: f32, y: f32) -> Direction {
        Direction { x, y }
    }
}

impl std::ops::Mul<f32> for &Direction {
    type Output = Direction;

    fn mul(self, rhs: f32) -> Direction {
        Direction {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Add<Direction> for &Position {
    type Output = Position;

    fn add(self, rhs: Direction) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position { x, y }
    }

    pub fn move_towards(&self, dir: &Direction) -> Position {
        self + dir * 0.25
    }
}

pub struct Player;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(spawn.system())
            .add_system(move_camera.system())
            .add_system(movement.system());
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((Position::new(400., 250.), Player, Direction::new(1., 0.5)));
    println!("should spaned?");
}

fn movement(keypress: Res<Keypress>, mut position: Mut<Position>, direction: &Direction) {
    if keypress.is(Keycode::W) {
        *position = position.move_towards(direction);
        
        println!("position {:?}", *position);
        println!("direction {:?}", direction);
    }
}

#[derive(Default)]
struct Rotation {
    radians: f32,
    degrees: f32,
}

fn move_camera(mouse_motion: Res<MouseMotion>, mut rotation: Local<Rotation>, mut direction: Mut<Direction>) {
    if mouse_motion.x == 0  {
        return;
    }
    let to_rad = std::f64::consts::PI / 180.;
    rotation.degrees += mouse_motion.x as f32;
    rotation.radians = (rotation.degrees as f64 * to_rad) as f32;
    
    direction.x = rotation.radians.cos();
    direction.y = rotation.radians.sin();

    let v = Vec2::new(direction.x, direction.y).normalize();
    direction.x = v.x();
    direction.y = v.y();
}
