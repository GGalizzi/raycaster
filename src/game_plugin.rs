use bevy::prelude::*;
use glam::vec2;
use sdl2::keyboard::Keycode;

use crate::{Keypress, MouseMotion};

pub struct GamePlugin;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
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

impl std::ops::Mul<f32> for Direction {
    type Output = Direction;

    fn mul(self, rhs: f32) -> Direction {
        Direction {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        Direction {
            x: -self.x,
            y: -self.y,
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

    pub fn move_towards(&self, dir: Direction, dt: f32) -> Position {
        // TODO: Would be a component, or stat or something
        let speed = 320.0;
        self + dir * (dt * speed)
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
    commands.spawn((Position::new(50., 50.), Player, Rotation::default()));
    println!("should spaned?");
}

fn movement(
    keypress: Res<Keypress>,
    time: Res<Time>,
    mut position: Mut<Position>,
    rotation: &Rotation,
) {
    let direction = rotation.direction();
    if keypress.is(Keycode::W) {
        *position = position.move_towards(direction, time.delta_seconds);
    }

    if keypress.is(Keycode::S) {
        *position = position.move_towards(-direction, time.delta_seconds);
    }

    if keypress.is(Keycode::A) {
        *position = position.move_towards(rotation.rotated(-90.).direction(), time.delta_seconds);
    }

    if keypress.is(Keycode::D) {
        *position = position.move_towards(rotation.rotated(90.).direction(), time.delta_seconds);
    }
}

const TO_RAD: f64 = std::f64::consts::PI / 180.;

#[derive(Default, Clone)]
pub struct Rotation {
    pub degrees: f32,
}

impl Rotation {
    pub fn new(degrees: f32) -> Rotation {
        Rotation {
            degrees,
        }
    }
    
    pub fn radians(&self) -> f32 {
        self.degrees * TO_RAD as f32
    }
    
    pub fn rotated(&self, degrees: f32) -> Rotation {
        let mut rotation = self.clone();
        rotation.add(degrees);
        rotation
    }

    pub fn add(&mut self, degrees: f32) {
        self.degrees += degrees;
        if self.degrees >= 360.0 {
            self.degrees = self.degrees - 360.0;
        }

        if self.degrees < 0.0 {
            self.degrees += 360.0;
        }
    }

    pub fn direction(&self) -> Direction {
        let v = Vec2::new(self.radians().cos(), self.radians().sin()).normalize();
        Direction::new(v.x(), v.y())
    }
}

fn move_camera(mouse_motion: Res<MouseMotion>, mut rotation: Mut<Rotation>) {
    if mouse_motion.x == 0 {
        return;
    }
    rotation.add(mouse_motion.x as f32);
}
