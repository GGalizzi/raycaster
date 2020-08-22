use sdl2::{render::Canvas, video::Window};

use crate::game_plugin::{Position, Rotation};

use crate::TILE_SIZE;

pub fn raycast(
    projection_plane: (i32, i32),
    fov: i32,
    position: &Position,
    rotation: &Rotation,
    canvas: &mut Canvas<Window>,
    angle_mod: f32,
    mut debug: bool,
) -> Result<(), String> {
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
    let mut ray_rotation = rotation.rotated(-half_fov.degrees());

    let tile_size = TILE_SIZE as f32;
    for x in 0..projection_plane.0 {

        let horizontal_distance = if ray_rotation.is_straight_horizontal() {
            (IntersectionPoint::default(), f32::MAX)
        } else {
            look_for_horizontal(&ray_rotation, position, &map, canvas)?
        };
        let vertical_distance = if ray_rotation.is_straight_vertical() {
            (IntersectionPoint::default(), f32::MAX)
        } else {
            look_for_vertical(&ray_rotation, position, &map, canvas)?
        };

        // Drawing some debug lines for the rays
        canvas.set_draw_color((20, 50, 20));
        let ray_dir = ray_rotation.direction() * 5.0;
        let some_distance_away =
            IntersectionPoint::new(position.x + ray_dir.x, position.y + ray_dir.y, TILE_SIZE);

        canvas.draw_line(
            (position.x.floor() as i32, position.y.floor() as i32),
            (some_distance_away.x.floor() as i32, some_distance_away.y.floor() as i32),
        )?;

        // Kay, draw the walls now if we hit something
        let ((intersection, closest_hit), side) = if horizontal_distance.1 < vertical_distance.1 {
            (horizontal_distance, 'h')
        } else {
            (vertical_distance, 'v')
        };

        if closest_hit != f32::MAX {
            let distance_to_wall = closest_hit * (ray_rotation.radians() - rotation.radians()).cos();
            let projected_height = (tile_size / distance_to_wall * distance_to_plane).floor() as i32;

            let mid_point = projection_plane.1 / 2;
            
            canvas.set_draw_color((100, 155, if side == 'v' { 155 } else { 255 }));
            canvas.draw_line(
                (x, mid_point - projected_height / 2),
                (x, mid_point + projected_height / 2),
            )?;
            canvas.set_draw_color((220, if side == 'v' { 15 } else { 255 }, 55));
            canvas.draw_point((intersection.x as i32, intersection.y as i32))?;
        }

        // Done, next angle
        ray_rotation.add(degrees_per_iteration);
    }

    Ok(())
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
    map: &Map,
    canvas: &mut Canvas<Window>,
) -> Result<(IntersectionPoint, f32), String> {
    let tile_size = TILE_SIZE as f32;
    let mut minusone = false;
    // Define the first intersection
    let mut intersection = {
        // The Y of the first intersection is going to be player_position_y / tile_size. And we add one tile_size to that if looking down
        let mut first_y = (position.y / tile_size).floor() * tile_size;
        if !ray_rotation.is_facing_up() {
            first_y += tile_size;
        } else {
            minusone = true;
        }

        let first_x = position.x + (position.y - first_y) / -ray_rotation.tan();

        IntersectionPoint::new(first_x, first_y, TILE_SIZE)
    };

    let distance_to_next_y = if ray_rotation.is_facing_up() {
        -tile_size
    } else {
        tile_size
    };
    let distance_to_next_x = distance_to_next_y / ray_rotation.tan();

    //canvas.draw_point((intersection.x as i32, intersection.y as i32))?;

    Ok(step_ray(
        position,
        &mut intersection,
        distance_to_next_x,
        distance_to_next_y,
        'h',
        minusone,
        map,
        0,
        canvas,
    ))
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
fn look_for_vertical(
    ray_rotation: &Rotation,
    position: &Position,
    map: &Map,
    canvas: &mut Canvas<Window>,
) -> Result<(IntersectionPoint, f32), String> {
    let tile_size = TILE_SIZE as f32;

    let mut minusone = false;
    // Define the first intersection
    let mut intersection = {
        // We know the first_x that will be hit because it's
        // the next (or previous) grid line from player position
        let mut first_x = (position.x / tile_size).floor() * tile_size;
        // And if the ray is going right, then it's the next grid line
        if !ray_rotation.is_facing_left() {
            first_x += tile_size;
        } else {
            minusone = true;
        }

        // tan(θ) = opposite/adjacent
        let first_y = position.y + (position.x - first_x) * -ray_rotation.tan();

        IntersectionPoint::new(first_x, first_y, TILE_SIZE)
    };

    let distance_to_next_x = if ray_rotation.is_facing_left() {
        -tile_size
    } else {
        tile_size
    };
    let distance_to_next_y = distance_to_next_x * ray_rotation.tan();

    //canvas.draw_point((intersection.x as i32, intersection.y as i32))?;

    Ok(step_ray(
        position,
        &mut intersection,
        distance_to_next_x,
        distance_to_next_y,
        'v',
        minusone,
        map,
        0,
        canvas,
    ))
}

fn step_ray(
    position: &Position,
    intersection: &mut IntersectionPoint,
    distance_to_next_x: f32,
    distance_to_next_y: f32,
    side: char,
    minusone: bool,
    map: &Map,
    n: i32,
    canvas: &mut Canvas<Window>,
) -> (IntersectionPoint, f32) {
    let mut grid = intersection.as_grid().to_pair();
    if minusone && side == 'v' {
        grid.0 -= 1;
    }
    if minusone && side == 'h' {
        grid.1 -= 1;
    }
    if map.is_blocking_at(grid) {

        return (
            *intersection,
            (position.y - intersection.y).hypot(position.x - intersection.x),
        );
    }

    if n > 500 {
        return (*intersection, f32::MAX);
    }


    let nextx = intersection.x + distance_to_next_x;
    let nexty = intersection.y + distance_to_next_y;
       step_ray(
        position,
        &mut IntersectionPoint::new(
            nextx,
            nexty,
            TILE_SIZE,
        ),
        distance_to_next_x,
        distance_to_next_y,
        side,
        minusone,
        map,
        n + 1,
        canvas,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            x: (self.x / self.grid_size).floor(),
            y: (self.y / self.grid_size).floor(),
            grid_size: 1.0,
        }
    }

    pub fn to_pair(&self) -> (i32, i32) {
        (self.x.floor() as i32, self.y.floor() as i32)
    }
}

impl Default for IntersectionPoint {
    fn default() -> IntersectionPoint {
        IntersectionPoint {
            x: 0.0,
            y: 0.0,
            grid_size: 0.0,
        }
    }
}

struct Map {
    tiles: Vec<char>,
    width: i32,
    height: i32,
}

impl Map {
    pub fn new() -> Map {
        Map {
            tiles: r#"
                #################
                #................
                #...#............
                #................
                #................
                #................
                #................
                ################.
            "#
            .to_owned()
            .replace('\n', "")
            .replace(' ', "")
            .chars()
            .collect(),
            width: 17,
            height: 8,
        }
    }

    pub fn is_blocking_at(&self, (x, y): (i32, i32)) -> bool {
        /*x == 1 || y == 1*/// || (x == 4 && y == 4)
        let given_idx = (self.width * y + x) as usize;
        if x >= self.height || y >= self.width || given_idx >= self.tiles.len() {
            return false;
        }
        self.tiles[given_idx] == '#'
    }
}
