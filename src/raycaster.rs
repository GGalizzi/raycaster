use sdl2::{render::Canvas, video::Window};

use crate::game_plugin::{Position, Rotation};

const TILE_SIZE: i32 = 1;

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
    let angle_between_rays = fov.degrees() / projection_plane.0 as f32;

    // The starting angle is the viewing angle substracted half the fov, so once you add all the angles, you'd get back to your FOV.
    let mut ray_rotation = rotation.rotated(-half_fov.degrees());

    let tile_size = TILE_SIZE as f32;
    for x in 0..projection_plane.0 {
        // HORIZONTAL INTERSECTIONS
        let horizontal_distance = if ray_rotation.degrees() == 90.0
            || ray_rotation.degrees() == 180.0
            || ray_rotation.degrees() == 0.0
        {
            None
        } else {
            // Coordinate of first intersection
            // In unit and grid coordinates, this is the first intersection
            let to_next = if ray_rotation.is_facing_up() {
                -1.
            } else {
                tile_size
            };
            let mut intersection_point = {
                let intersection_point_y = (position.y / tile_size).trunc() * tile_size + to_next;
                let adj = position.y - intersection_point_y;
                let intersection_point_x = position.x + (adj * ray_rotation.tan());
                /*let intersection_point_x =
                position.x + (position.y - intersection_point_y) / ray_rotation.tan();*/
                IntersectionPoint::new(intersection_point_x, intersection_point_y, TILE_SIZE)
            };

            canvas.set_draw_color((0, 200, 0));
            canvas.draw_line(
                (position.x as i32, position.y as i32),
                (intersection_point.x as i32, intersection_point.y as i32),
            );

            // Distances to next x and y grid line (horizontal)
            // Y is always going to be the tile_size
            let y_dist = tile_size
                * if ray_rotation.is_facing_up() {
                    -1.0
                } else {
                    1.0
                };

            let x_dist = tile_size / ray_rotation.tan();

            let mut traveled = 0;
            let mut hit = false;

            while !hit {
                if traveled >= 5000
                    || intersection_point.out_of_bounds()
                    || map.out_of_bounds(intersection_point.as_grid().to_pair())
                {
                    break;
                }

                if map.is_blocking_at(intersection_point.as_grid().to_pair()) {
                    hit = true;
                    break;
                }

                intersection_point = IntersectionPoint::new(
                    intersection_point.x + x_dist,
                    intersection_point.y + y_dist,
                    TILE_SIZE,
                );

                traveled += 1;
            }

            if debug {
                println!(
                    "position {:?}\nintersection point {:?}\ndis {:?}\nrayrot {:?}, hit {:?}",
                    position,
                    intersection_point,
                    '?',
                    ray_rotation.degrees(),
                    hit
                );
            }
            if hit {
                Some((position.x - intersection_point.x).hypot(position.y - intersection_point.y))
            /*((position.x - intersection_point.x as f32).abs()
            / ray_rotation.radians().cos()) as i32*/
            } else {
                None
            }
        };

        // VERTICAL
        let vertical_distance = if ray_rotation.degrees() == 180.0 || ray_rotation.degrees() == 0.0
        {
            None
        } else {
            // Coordinate of first intersection
            let to_next = if ray_rotation.is_facing_left() {
                -1.0
            } else {
                tile_size
            };
            let mut intersection_point = {
                let intersection_point_x = (position.x / tile_size).trunc() * tile_size + to_next;
                let intersection_point_y =
                    position.y + (position.x - intersection_point_x) / ray_rotation.tan();
                IntersectionPoint::new(intersection_point_x, intersection_point_y, TILE_SIZE)
            };

            if intersection_point.y > 5000. || intersection_point.x > 5000. {
                println!(
                    "iy {:?} ix {:?}",
                    intersection_point.y, intersection_point.x
                );
                println!(
                    "rt {:?} ; tan(rt) {:?}, px {:?}",
                    ray_rotation.degrees(),
                    ray_rotation.tan(),
                    position.x
                );
            }

            canvas.set_draw_color((0, 200, 0));
            canvas.draw_line(
                (position.x as i32, position.y as i32),
                (intersection_point.x as i32, intersection_point.y as i32),
            );

            let x_dist = if ray_rotation.is_facing_left() {
                -tile_size
            } else {
                tile_size
            };
            let y_dist = tile_size / ray_rotation.tan();

            let mut traveled = 0;
            let mut hit = false;

            while !hit {
                if traveled >= 5000
                    || intersection_point.out_of_bounds()
                    || map.out_of_bounds(intersection_point.as_grid().to_pair())
                {
                    break;
                }

                if map.is_blocking_at(intersection_point.as_grid().to_pair()) {
                    canvas.set_draw_color((255, 255, 255));
                    canvas.draw_point((intersection_point.x as i32, intersection_point.y as i32));
                    hit = true;
                    break;
                }

                intersection_point = IntersectionPoint::new(
                    intersection_point.x + x_dist,
                    intersection_point.y + y_dist,
                    TILE_SIZE,
                );

                traveled += 1;
            }

            if hit {
                let dis =
                    (position.x - intersection_point.x).hypot(position.y - intersection_point.y);
                if debug {
                    println!(
                        "position {:?}\nintersection point {:?}\ndis {:?}\nrayrot {:?}",
                        position,
                        intersection_point,
                        dis,
                        ray_rotation.degrees()
                    );
                }
                Some(dis)
            } else {
                None
            }
        };

        if debug {
            debug = false;
        }

        let hit = horizontal_distance.is_some() || vertical_distance.is_some();

        if hit {
            let (side, distance_to_wall) = if horizontal_distance.is_none() {
                ('v', vertical_distance.unwrap())
            } else if vertical_distance.is_none() {
                ('h', horizontal_distance.unwrap())
            } else {
                let (horizontal_distance, vertical_distance) =
                    (horizontal_distance.unwrap(), vertical_distance.unwrap());
                if horizontal_distance < vertical_distance {
                    ('h', horizontal_distance)
                } else {
                    ('v', vertical_distance)
                }
            };
            /*let (side, distance_to_wall) = if horizontal_distance < vertical_distance {
                ('h', horizontal_distance.unwrap())
            } else {
                ('v', vertical_distance.unwrap())
            };*/

            let beta = Rotation::new(-half_fov.degrees()).rotated(angle_between_rays * x as f32);
            //println!("beta {:?}", beta.degrees());
            let distance_to_wall = distance_to_wall / (beta.radians() - rotation.radians()).cos();

            //println!("dtw {:?}",distance_to_wall);
            let projected_height = if distance_to_wall <= f32::EPSILON {
                0.0
            } else {
                (tile_size / distance_to_wall) * distance_to_plane
            };

            let plane_center = projection_plane.1 / 2;

            let top_of_wall = plane_center - projected_height as i32 / 2;
            let bottom_of_wall = plane_center + projected_height as i32 / 2;

            if side == 'h' {
                canvas.set_draw_color((98, 10, 10));
            } else {
                canvas.set_draw_color((108, 100, 20));
            }
            canvas
                .draw_line((x, top_of_wall), (x, bottom_of_wall))
                .unwrap();
        }

        if rotation.degrees() == 90.0 && (x == 0 || x == projection_plane.0 - 1) {
            println!("angleray {:?}", ray_rotation.degrees());
        }

        ray_rotation = ray_rotation.rotated(angle_between_rays);
    }
}

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
