use png;
use std::fs::File;
use tetra::{
    graphics,
    graphics::{DrawParams, Drawable},
    Context,
};

pub struct Texture {
    drawable: graphics::Texture,
    data: Vec<u8>,
    width: u32,
}

impl Texture {
    pub fn new(ctx: &mut Context, path: &str) -> Texture {
        let drawable = graphics::Texture::new(ctx, path).unwrap();
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        println!("{:?}", info);
        Texture {
            drawable,
            data: buf,
            width: info.width,
        }
    }

    pub fn color_at(&self, x: i32, y: i32) -> (u8, u8, u8) {
        let idx = ((self.width as i32 * y + x) * 3) as usize;
        let d = &self.data;
        (d[idx], d[idx + 1], d[idx + 2])
    }
    
    pub fn width(&self) -> i32 {
        self.drawable.width()
    }
    
    pub fn height(&self) -> i32 {
        self.drawable.height()
    }

    pub fn draw(&self, ctx: &mut Context, params: DrawParams) {
        self.drawable.draw(ctx, params);
    }
}
