use png;
use std::fs::File;

pub struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn new(path: &str) -> Texture {
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        println!("{:?}", info);
        Texture {
            data: buf,
            width: info.width,
            height: info.height,
        }
    }

    pub fn color_at(&self, x: i32, y: i32) -> (u8, u8, u8) {
        let idx = ((self.width as i32 * y + x) * 3) as usize;
        let d = &self.data;
        if idx >= d.len() {
            return (0, 0, 0);
        }
        (d[idx], d[idx + 1], d[idx + 2])
    }

    pub fn width(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> i32 {
        self.height as i32
    }

    pub fn draw_strip_at_ex(
        &self,
        x: i32,
        tex_x: i32,
        top: i32,
        bottom: i32,
        buf: &mut [u8],
        mult: Option<&[f32; 3]>,
    ) {
        let height = bottom - top;

        // TODO: Replace fixed 320 and 200s by a width being passed
        for y in 0..height {
            let tex_y = (y as f64 / height as f64 * self.height as f64).round() as usize;

            let screen_y = (top + y as i32) as usize;

            if screen_y > 200 {
                continue;
            }
            self.copy_to_ex(tex_x, tex_y as i32, x, screen_y as i32, buf, mult)
        }
    }

    pub fn draw_strip_at(&self, x: i32, tex_x: i32, top: i32, bottom: i32, buf: &mut [u8]) {
        self.draw_strip_at_ex(x, tex_x, top, bottom, buf, None)
    }
}

impl Drawable for Texture {
    fn copy_to_ex(
        &self,
        tex_x: i32,
        tex_y: i32,
        x: i32,
        y: i32,
        buf: &mut [u8],
        mult: Option<&[f32; 3]>,
    ) {
        let (r, g, b) = self.color_at(tex_x, tex_y);

        let idx = ((320 * y + x) * 4) as usize;
        if idx >= buf.len() {
            return;
        }

        let (r, g, b) = if let Some(&[mr, mg, mb]) = mult {
            (
                (r as f32 * mr) as u8,
                (g as f32 * mg) as u8,
                (b as f32 * mb) as u8,
            )
        } else {
            (r, g, b)
        };

        buf[idx..idx + 4].copy_from_slice(&[r, g, b, 0xff]);
    }
}

pub trait Drawable {
    fn copy_to_ex(
        &self,
        tex_x: i32,
        tex_y: i32,
        x: i32,
        y: i32,
        buf: &mut [u8],
        mult: Option<&[f32; 3]>,
    );

    fn copy_to(&self, tex_x: i32, tex_y: i32, x: i32, y: i32, buf: &mut [u8]) {
        self.copy_to_ex(tex_x, tex_y, x, y, buf, None)
    }
}
