use std::path::Path;
use image::{DynamicImage, GenericImageView};
use std::simd::{u8x4, f32x4};

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
    pub fn sample_quad(&self, u: f32x4, v: f32x4) -> (u8x4, u8x4, u8x4, u8x4) {
        // For now, run lane-by-lane — later you could SIMD-optimize texture fetches
        let mut r = [0; 4];
        let mut g = [0; 4];
        let mut b = [0; 4];
        let mut a = [0; 4];
        for lane in 0..4 {
            let (rr, gg, bb, aa) = self.sample(u[lane], v[lane]);
            r[lane] = rr;
            g[lane] = gg;
            b[lane] = bb;
            a[lane] = aa;
        }
        (u8x4::from_array(r), u8x4::from_array(g), u8x4::from_array(b), u8x4::from_array(a))
    }
}
