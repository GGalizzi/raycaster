use sdl2::ttf;
use sdl2::ttf::Sdl2TtfContext;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Font {
    context: Sdl2TtfContext,
}

impl Font {
    pub fn new(context: Sdl2TtfContext) -> Font {
        Font { context }
    }

    pub fn build(&self, path: &str, size: u16) -> Result<Text, String> {
        let font = self.context.load_font(path, size)?;

        Ok(Text { font })
    }
}

pub struct Text<'ttf> {
    font: ttf::Font<'ttf, 'static>,
}

impl<'ttf> Text<'ttf> {
    pub fn draw(&self, string: &str, buf: &mut [u8]) -> Result<(), String> {
        let font_surface = self
            .font
            .render(string)
            .blended((0, 0, 255, 255))
            .map_err(|e| e.to_string())?;

        font_surface.with_lock(|data| {
            for x in 0..font_surface.width() {
                for y in 0..font_surface.height() {
                    let dst_idx = (((320 * y) + x) * 4) as usize;
                    let dst = buf.get_mut(dst_idx..dst_idx + 4);

                    let src_idx = (((font_surface.width() * y) + x) * 4) as usize;
                    let src = data.get(src_idx..src_idx + 4);

                    if let Some(dst) = dst {
                        if let Some(src) = src {
                            if src[1] == 0 && src[2] == 0 && src[3] == 0 {
                                continue;
                            }
                            dst.copy_from_slice(&[src[1], src[2], src[3], src[0]]);
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
