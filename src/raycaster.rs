use sdl2::{render::Canvas, video::Window};

use crate::game_plugin::{Position, Rotation};

const TILE_SIZE: i32 = 64;

pub fn raycast(
    projection_plane: (i32, i32),
    fov: i32,
    position: &Position,
    rotation: &Rotation,
    canvas: &mut Canvas<Window>,
) {
    let map: Map = Map::new();
    // As by the Pythagorean theorem we can get the adjacent side length by
    // using the formula tan(angle) = opposite / adjacent
    // We know the angle, because that's FOV/2
    // We know opposite, because that's projection's plane width / 2
    let distance_to_plane = (projection_plane.0 / 2) / (((fov / 2) as f32).tan()) as i32;

    // The angle increment between rays is known by the fov. ie, how many steps would you need to fit the plane.
    let angle_between_rays = fov / projection_plane.0;

    // The starting angle is the viewing angle substracted half the fov, so once you add all the angles, you'd get back to your FOV.
    let mut ray_rotation = Rotation::new(rotation.degrees() - fov as f32 / 2.0);

    let tile_size = TILE_SIZE as f32;
    for x in 0..projection_plane.0 {
        let ray_direction = ray_rotation.direction();

        // HORIZONTAL INTERSECTIONS

        // Coordinate of first intersection
        let to_next = if ray_rotation.is_facing_up() { -1 } else { TILE_SIZE };

        // In unit and grid coordinates, this is the first intersection
        let mut intersection_point = {
            let intersection_point_y = ((position.y / tile_size) as i32 * TILE_SIZE) + to_next;
            let intersection_point_x = (position.x
                + (position.y - intersection_point_y as f32) / ray_rotation.degrees().tan())
                as i32;
            IntersectionPoint::new(intersection_point_x, intersection_point_y, TILE_SIZE)
        };

        // Distances to next x and y grid line (horizontal)
        // Y is always going to be the tile_size
        let y_dist = tile_size
            * if ray_rotation.is_facing_up() {
                -1.0
            } else {
                1.0
            };

        let x_dist = tile_size / ray_rotation.degrees().tan();

        let mut traveled = 0;
        let mut hit = false;

        while !hit {
            if traveled >= 500 || intersection_point.out_of_bounds() {
                break;
            }

            if map.is_blocking_at(intersection_point.as_grid().to_pair()) {
                hit = true;
                break;
            }

            intersection_point = IntersectionPoint::new(
                intersection_point.x + x_dist as i32,
                intersection_point.y + y_dist as i32,
                TILE_SIZE,
            );

            traveled += 1;
        }

        if hit {
            canvas.set_draw_color((255, 0, 0));
            canvas
                .draw_line((x, 0), (x, projection_plane.1))
                .unwrap();
        }

        ray_rotation = Rotation::new(ray_rotation.degrees() + angle_between_rays as f32);
    }
}

struct IntersectionPoint {
    pub x: i32,
    pub y: i32,
    grid_size: i32,
}

impl IntersectionPoint {
    pub fn new(x: i32, y: i32, grid_size: i32) -> IntersectionPoint {
        IntersectionPoint { x, y, grid_size }
    }

    pub fn as_grid(&self) -> IntersectionPoint {
        IntersectionPoint {
            x: self.x / self.grid_size,
            y: self.y / self.grid_size,
            grid_size: 1,
        }
    }

    pub fn to_pair(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    
    pub fn out_of_bounds(&self) -> bool {
        self.x < 0 || self.y < 0
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
                #######################
                #..........#####.....##
                #..............#.....##
                #..............##.....#
                #............###......#
                #.....................#
                #.....................#
                #######################
            "#
            .to_owned().chars().collect(),
            width: 23,
        }
    }

    pub fn is_blocking_at(&self, (x, y): (i32, i32)) -> bool {
        let given_idx = (self.width * y + x) as usize;
        self.tiles[given_idx] == '#'
    }
}
