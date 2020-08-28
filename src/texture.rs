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

    pub fn draw_strip_at(&self, x: i32, mut top: i32, mut bottom: i32, buf: &mut [u8]) {
        //self.data.chunks_exact(3)

        if top < 0 { top = 0; }
        if bottom > 200 { bottom = 200; }

        // TODO: Replace 320 by a width being passed
        for (i, pixel) in buf
            .chunks_exact_mut(4)
            .enumerate()
            .skip(320 * top as usize)
            .skip(x as usize)
            .step_by(320)
        {
            if (i / 320) > bottom as usize { break; }
            pixel.copy_from_slice(&[0xff, 0x00, 0x00, 0x2f]);
        }
    }
}
