use bevy::prelude::*;
use sdl2::keyboard::Keycode;

use crate::Keypress;

pub struct GamePlugin;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
}

pub struct Player;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(spawn.system())
            .add_system(movement.system());
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((Position::new(5, 6), Player));
    println!("should spaned?");
}

fn movement(keypress: Res<Keypress>, mut position: Mut<Position>) {
    if keypress.is(Keycode::W) {
        position.x += 1;
    }
}
