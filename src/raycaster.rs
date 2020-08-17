use sdl2::{render::Canvas, video::Window};

use crate::game_plugin::{Position, Rotation};

const TILE_SIZE: i32 = 10;

pub fn raycast(
    projection_plane: (i32, i32),
    fov: i32,
    position: &Position,
    rotation: &Rotation,
    canvas: &mut Canvas<Window>,
    angle_mod: f32,
    mut debug: bool,
) {
    let map: Map = Map::new();
    let half_fov = Rotation::new(fov as f32 / 2.0);
    let fov = Rotation::new(fov as f32);

    // using the formula tan(angle) = opposite / adjacent
    // We know the angle, because that's FOV/2
    // We know opposite, because that's projection's plane width / 2
    let distance_to_plane = (projection_plane.0 / 2) as f32 / half_fov.tan() + angle_mod;

    // The angle increment between rays is known by the fov. ie, how many steps would you need to fit the plane.
    let degrees_per_iteration = fov.degrees() / projection_plane.0 as f32;

    // The starting angle is the viewing angle rotated minus half the fov.
    // \  |  /
    //  \ | /
    //  º\|/
    //  º p--------
    //  ºººº
    let mut ray_rotation = rotation.rotated( -half_fov.degrees());
    
    println!("player pos {:?}", position);

    let tile_size = TILE_SIZE as f64;
    for x in 0..projection_plane.0 {
        // This is the angle that forms from the viewing direction to the current ray
        // \  |  /
        //  \º|º/
        //   \|/
        //    p
        let relative_angle = Rotation::new(half_fov.degrees() + x as f32 * degrees_per_iteration);

        canvas.set_draw_color((120, 120, 120));
        look_for_horizontal(&ray_rotation, position, rotation, canvas);
        //look_for_vertical

        canvas.set_draw_color((20, 50, 20));
        let ray_dir = ray_rotation.direction() * 5.0;
        let some_distance_away = IntersectionPoint::new(position.x + ray_dir.x, position.y + ray_dir.y, TILE_SIZE);
        canvas.draw_line((position.x as i32, position.y as i32), (some_distance_away.x as i32, some_distance_away.y as i32));

        // Done, next angle
        ray_rotation.add(degrees_per_iteration);
    }
}

// Looks for horizontal grid lines
// ============= <-
// |  |  |  |  |
// ============= <- these
// |  |  |  |  |
// ============= <-
fn look_for_horizontal(
    ray_rotation: &Rotation,
    position: &Position,
    rotation: &Rotation,
    canvas: &mut Canvas<Window>,
) {
    let tile_size = TILE_SIZE as f32;
    // Define the first intersection
    let intersection = {
        // The Y of the first intersection is going to be player_position_y / tile_size. And we add one tile_size to that if looking down
        let mut first_y = (position.y / tile_size).trunc() * tile_size;
        if !rotation.is_facing_up() {
            first_y += tile_size;
        }

        let first_x = position.x + (position.y - first_y) / ray_rotation.tan();

        IntersectionPoint::new(first_x, first_y, TILE_SIZE)
    };

    let distance_to_next_y = if rotation.is_facing_up() {
        -tile_size
    } else {
        tile_size
    };

    let distance_to_next_x = distance_to_next_y * rotation.tan();

    canvas.draw_point((intersection.x as i32, intersection.y as i32));
    let intersection = IntersectionPoint::new(
        intersection.x + distance_to_next_x,
        intersection.y + distance_to_next_y,
        TILE_SIZE,
    );
    canvas.draw_point((intersection.x as i32, intersection.y as i32));
}

// Looks for vertical grid lines
// ‖--‖--‖--‖--‖
// ‖  ‖  ‖  ‖  ‖
// ‖--‖--‖--‖--‖
// ‖  ‖  ‖  ‖  ‖
// ‖--‖--‖--‖--‖
// ^  ^  ^  ^  ^
//       |
//     these

#[derive(Debug)]
struct IntersectionPoint {
    pub x: f32,
    pub y: f32,
    grid_size: f32,
}

impl IntersectionPoint {
    pub fn new(x: f32, y: f32, grid_size: i32) -> IntersectionPoint {
        IntersectionPoint {
            x,
            y,
            grid_size: grid_size as f32,
        }
    }

    pub fn as_grid(&self) -> IntersectionPoint {
        IntersectionPoint {
            x: self.x / self.grid_size,
            y: self.y / self.grid_size,
            grid_size: 1.0,
        }
    }

    pub fn to_pair(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub fn out_of_bounds(&self) -> bool {
        self.x < 0.0 || self.y < 0.0
    }
}

struct Map {
    tiles: Vec<char>,
    width: i32,
}

impl Map {
    pub fn new() -> Map {
        Map {
            tiles: r#"
                #................
                #................
                #................
                #................
                #................
                #................
                #................
                ################.
            "#
            .to_owned()
            .chars()
            .collect(),
            width: 17,
        }
    }

    pub fn is_blocking_at(&self, (x, y): (i32, i32)) -> bool {
        x == 0 || y == 0 // || (x == 4 && y == 4)
                         /*
                         let given_idx = (self.width * y + x) as usize;
                         if given_idx > 285 {
                             println!("{:?} {:?} {:?}", self.width, y, x);
                         }
                         self.tiles[given_idx] == '#'
                         */
    }

    pub fn out_of_bounds(&self, (x, y): (i32, i32)) -> bool {
        if y > 5000 || x > 5000 {
            println!("xy {:?} {:?}", x, y);
        }
        let given_idx = (self.width * y + x) as usize;
        x < 0 || y < 0 || given_idx >= self.tiles.len()
    }
}
