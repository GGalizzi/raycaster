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
        (d[idx], d[idx + 1], d[idx + 2])
    }

    pub fn width(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> i32 {
        self.height as i32
    }

    pub fn draw_strip_at(&self, x: i32, tex_x: i32, top: i32, bottom: i32, buf: &mut [u8]) {
        //self.data.chunks_exact(3)

        let height = bottom - top;

        // TODO: Replace fixed 320 and 200s by a width being passed
        for y in 0..height {
            let tex_y = (y as f64 / height as f64 * self.height as f64).round() as usize;

            let screen_y = (top + y as i32) as usize;

            if screen_y > 200 {
                continue;
            }
            if let Some(pixel) = self
                .data
                .chunks_exact(3)
                .skip(tex_y * self.width as usize)
                .skip(tex_x as usize)
                .next()
            {
                if let Some(screen_slice) = buf
                    .chunks_exact_mut(4)
                    .skip(screen_y * 320)
                    .skip(x as usize)
                    .next()
                {
                    screen_slice.copy_from_slice(&[pixel[0], pixel[1], pixel[2], 0xff]);
                }
            }
        }
    }
}
