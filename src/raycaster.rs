use crate::game_plugin::{Position, Rotation};
use crate::texture::{Drawable, Texture};

use crate::TILE_SIZE;

pub fn raycast(
    projection_plane: (i32, i32),
    fov: i32,
    position: &Position,
    rotation: &Rotation,
    pixels: &mut [u8],
    wall_texture: &Texture,
    floor_texture: &Texture,
) -> Result<(), String> {
    let map: Map = Map::new();
    let half_fov = Rotation::new(fov as f32 / 2.0);
    let fov = Rotation::new(fov as f32);

    // using the formula tan(angle) = opposite / adjacent
    // We know the angle, because that's FOV/2
    // We know opposite, because that's projection's plane width / 2
    let distance_to_plane = (projection_plane.0 / 2) as f32 / half_fov.tan();

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
            look_for_horizontal(&ray_rotation, position, &map)?
        };
        let vertical_distance = if ray_rotation.is_straight_vertical() {
            (IntersectionPoint::default(), f32::MAX)
        } else {
            look_for_vertical(&ray_rotation, position, &map)?
        };

        // Drawing some debug lines for the rays
        /*
        canvas.set_draw_color((20, 50, 20));
        let ray_dir = ray_rotation.direction() * 5.0;
        let some_distance_away = (position.x + ray_dir.x, position.y + ray_dir.y);

        canvas.draw_line(
            (position.x.floor() as i32, position.y.floor() as i32),
            (
                some_distance_away.0.floor() as i32,
                some_distance_away.1.floor() as i32,
            ),
        )?;
        */

        // Kay, draw the walls now if we hit something
        let ((intersection, closest_hit), side) = if horizontal_distance.1 < vertical_distance.1 {
            (horizontal_distance, 'h')
        } else {
            (vertical_distance, 'v')
        };

        if closest_hit != f32::MAX {
            let distance_to_wall =
                closest_hit * (ray_rotation.radians() - rotation.radians()).cos();
            let projected_height =
                (tile_size / distance_to_wall * distance_to_plane).floor() as i32;

            let mid_point = projection_plane.1 / 2;

            let wall_bottom = mid_point + projected_height / 2;
            let wall_top = mid_point - projected_height / 2;

            // Draw fill color of walls
            /*
            let color =
                (if side == 'v' { 750.0 } else { 450.0 } * (1.0 / distance_to_wall.sqrt())) as u8;
            canvas.set_draw_color((color, color, color));
            canvas.draw_line((x, wall_top), (x, wall_bottom - 2))?;
            */

            // Draw wall texture
            let wall_x = if side == 'h' {
                intersection.x
            } else {
                intersection.y
            };
            let tex_x = ((wall_x / tile_size).fract() * wall_texture.width() as f32) as i32;

            let dst_to_light = map.distance_to_light(
                (intersection.x / tile_size) as i32,
                (intersection.y / tile_size) as i32,
            );

            let mult = 1. / distance_to_wall + 1. / dst_to_light;

            // So dark we don't need to copy anything
            if mult > 0.008 {
                wall_texture.draw_strip_at_ex(
                    x,
                    tex_x,
                    wall_top,
                    wall_bottom,
                    pixels,
                    Some(&[mult, mult, mult]),
                );
            }

            let angle = rotation.rotated(-ray_rotation.degrees());

            floorcast(
                x,
                wall_bottom..projection_plane.1,
                &position,
                &ray_rotation,
                angle.clone(),
                distance_to_plane,
                projection_plane,
                pixels,
                floor_texture,
                'f',
                &map,
            )?;

            floorcast(
                x,
                0..wall_top,
                &position,
                &ray_rotation,
                angle,
                distance_to_plane,
                projection_plane,
                pixels,
                floor_texture,
                'c',
                &map,
            )?;
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
) -> Result<(IntersectionPoint, f32), String> {
    let tile_size = TILE_SIZE as f32;
    // Define the first intersection
    let mut intersection = {
        // The Y of the first intersection is going to be player_position_y / tile_size. And we add one tile_size to that if looking down
        let mut first_y = (position.y / tile_size).floor() * tile_size;
        let mut mod_y = 0;
        if !ray_rotation.is_facing_up() {
            first_y += tile_size;
        } else {
            mod_y -= 1;
        }

        let first_x = position.x + (position.y - first_y) / -ray_rotation.tan();

        IntersectionPoint::new(first_x, first_y, 0, mod_y, TILE_SIZE)
    };

    let distance_to_next_y = if ray_rotation.is_facing_up() {
        -tile_size
    } else {
        tile_size
    };
    let distance_to_next_x = distance_to_next_y / ray_rotation.tan();

    Ok(step_ray(
        position,
        &mut intersection,
        distance_to_next_x,
        distance_to_next_y,
        'h',
        map,
        0,
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
) -> Result<(IntersectionPoint, f32), String> {
    let tile_size = TILE_SIZE as f32;

    // Define the first intersection
    let mut intersection = {
        // We know the first_x that will be hit because it's
        // the next (or previous) grid line from player position
        let mut first_x = (position.x / tile_size).floor() * tile_size;
        let mut mod_x = 0;
        if !ray_rotation.is_facing_left() {
            // And if the ray is going right, then it's the next grid line
            first_x += tile_size;
        } else {
            // Otherwise it's in the same position but it needs to check the grid to the left
            mod_x -= 1;
        }

        // tan(θ) = opposite/adjacent
        let first_y = position.y + (position.x - first_x) * -ray_rotation.tan();

        IntersectionPoint::new(first_x, first_y, mod_x, 0, TILE_SIZE)
    };

    let distance_to_next_x = if ray_rotation.is_facing_left() {
        -tile_size
    } else {
        tile_size
    };
    let distance_to_next_y = distance_to_next_x * ray_rotation.tan();

    Ok(step_ray(
        position,
        &mut intersection,
        distance_to_next_x,
        distance_to_next_y,
        'v',
        map,
        0,
    ))
}

fn step_ray(
    position: &Position,
    intersection: &mut IntersectionPoint,
    distance_to_next_x: f32,
    distance_to_next_y: f32,
    side: char,
    map: &Map,
    n: i32,
) -> (IntersectionPoint, f32) {
    if map.is_blocking_at(intersection.as_grid_pair()) {
        return (
            *intersection,
            (position.y - intersection.y).hypot(position.x - intersection.x),
        );
    }

    if n > 250 {
        return (*intersection, f32::MAX);
    }

    let nextx = intersection.x + distance_to_next_x;
    let nexty = intersection.y + distance_to_next_y;
    step_ray(
        position,
        &mut IntersectionPoint::new(
            nextx,
            nexty,
            intersection.mod_x,
            intersection.mod_y,
            TILE_SIZE,
        ),
        distance_to_next_x,
        distance_to_next_y,
        side,
        map,
        n + 1,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct IntersectionPoint {
    pub x: f32,
    pub y: f32,
    pub mod_x: i32, // Which grid does this point belong to.
    pub mod_y: i32,
    grid_size: f32,
}

impl IntersectionPoint {
    pub fn new(x: f32, y: f32, mod_x: i32, mod_y: i32, grid_size: i32) -> IntersectionPoint {
        IntersectionPoint {
            x,
            y,
            mod_x,
            mod_y,
            grid_size: grid_size as f32,
        }
    }

    pub fn as_grid_pair(&self) -> (i32, i32) {
        (
            (self.x / self.grid_size).floor() as i32 + self.mod_x,
            (self.y / self.grid_size).floor() as i32 + self.mod_y,
        )
    }
}

impl Default for IntersectionPoint {
    fn default() -> IntersectionPoint {
        IntersectionPoint {
            x: 0.0,
            y: 0.0,
            mod_x: 0,
            mod_y: 0,
            grid_size: 0.0,
        }
    }
}

struct Map {
    tiles: Vec<char>,
    width: i32,
    height: i32,
    lights: Vec<(i32, i32)>,
}

impl Map {
    pub fn new() -> Map {
        let mut map = Map {
            tiles: r#"
                ##################
                #.............####
                #..............###
                #.............####
                #.............####
                #.............####
                #..............###
                #..............###
                #..............###
                #........#.....###
                #........#.....###
                #...######.....###
                #...#..........###
                #...#.....#....###
                #...#.....#....###
                #...####..#....###
                #.........#...l###
                ##################
            "#
            .to_owned()
            .replace('\n', "")
            .replace(' ', "")
            .chars()
            .collect(),
            width: 18,
            height: 18,
            lights: Vec::new(),
        };
        
        map.bake_lights();
        map
    }
    
    fn bake_lights(&mut self) {
        self.lights.clear();
        for (i, t) in self.tiles.iter().enumerate() {
            if *t == 'l' {
                self.lights.push((i as i32 % self.width, i as i32 / self.width))
            }
        }
    }

    pub fn is_blocking_at(&self, (x, y): (i32, i32)) -> bool {
        let given_idx = (self.width * y + x) as usize;
        if y > self.height || x > self.width || given_idx >= self.tiles.len() {
            return false;
        }
        self.tiles[given_idx] == '#'
    }

    pub fn distance_to_light(&self, x: i32, y: i32) -> f32 {
        let mut closest = None;
        for (lx, ly) in &self.lights {
            let dst = ((x - lx) as f32).abs().hypot(((y - ly) as f32).abs());
            
            if let Some(c) = closest {
                if dst < c { closest = Some(dst); }
            } else {
                closest = Some(dst);
            }
        }
        
        if let Some(closest) = closest {
            closest
        } else {
            f32::MAX
        }
    }
}

const PLAYER_HEIGHT: i32 = TILE_SIZE / 2;
fn floorcast(
    x: i32,
    range: std::ops::Range<i32>,
    player: &Position,
    ray: &Rotation,
    angle: Rotation,
    distance_to_plane: f32,
    projection_plane: (i32, i32),
    pixels: &mut [u8],
    floor_texture: &Texture,
    side: char,
    map: &Map,
) -> Result<(), String> {
    let projection_center = projection_plane.1 / 2;
    let tile_size = TILE_SIZE as f32;

    for row in range {
        let bheight = if side == 'f' {
            row - projection_center
        } else {
            projection_center - row
        };
        let straight_distance =
            (PLAYER_HEIGHT as f32 / (bheight) as f32) * distance_to_plane as f32;

        let distance_to_point = straight_distance / angle.cos();

        let ends = (
            distance_to_point * ray.cos() + player.x,
            distance_to_point * ray.sin() + player.y,
        );

        let tex_x = ((ends.0 / tile_size).fract() * floor_texture.width() as f32) as i32;
        let tex_y = ((ends.1 / tile_size).fract() * floor_texture.height() as f32) as i32;
        
        let mut light_mult = 1.0 / map.distance_to_light((ends.0 / tile_size) as i32, (ends.1 / tile_size) as i32);
        
        if light_mult < 0.08 { light_mult = 0.; }
        
        let mult = 1. / distance_to_point;

        // So dark we don't need to copy anything
        if mult < 0.005 {
            continue;
        }

        floor_texture.copy_to_ex(tex_x, tex_y, x, row, pixels, Some(&[(mult + light_mult).min(1.0), mult, mult]));
    }

    Ok(())
}
