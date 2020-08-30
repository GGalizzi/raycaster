pub fn get_line((mut x0, mut y0): (i32, i32), (mut x1, mut y1): (i32, i32)) -> Vec<(i32, i32)> {
    let mut result = Vec::new();

    let steep = (y1 - y0).abs() > (x1 - x0).abs();

    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let deltax = x1 - x0;
    let deltay = (y1 - y0).abs();

    let mut error = 0;

    let ystep;

    let mut y = y0;

    if y0 < y1 {
        ystep = 1;
    } else {
        ystep = -1;
    }

    for x in x0..x1 {
        result.push(if steep { (y, x) } else { (x, y) });

        error += deltay;

        if 2 * error >= deltax {
            y += ystep;
            error -= deltax;
        }
    }

    result
}

pub fn raycast<F>(point_a: (i32, i32), point_b: (i32, i32), is_valid: F) -> Option<(i32, i32)>
where
    F: Fn((i32, i32)) -> bool,
{
    for point in get_line(point_a, point_b) {
        if is_valid(point) {
            return Some(point);
        }
    }

    None
}
