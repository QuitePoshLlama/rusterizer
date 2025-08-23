use std::path::Path;
use image::{DynamicImage, GenericImageView};

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(path: P) -> image::ImageResult<Self> {
        let img: DynamicImage = image::open(path)?;
        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in (0..height).rev() {
            let row_start = (y * width * 4) as usize;
            let row_end = row_start + (width * 4) as usize;
            rgba.extend_from_slice(&rgba_img.as_raw()[row_start..row_end]);
        }
        Ok(Self { width, height, rgba })
    }
    pub fn sample(&self, u: f32, v: f32) -> (u8, u8, u8, u8) {
        let u = u.fract();
        let v = v.fract();
        let x = (u * (self.width as f32 - 1.0)).round() as u32;
        let y = (v * (self.height as f32 - 1.0)).round() as u32;
        let idx = ((y * self.width + x) * 4) as usize;
        (
            self.rgba[idx],
            self.rgba[idx + 1],
            self.rgba[idx + 2],
            self.rgba[idx + 3],
        )
    }
}
