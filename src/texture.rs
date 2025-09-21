use std::{path::Path, simd::num::SimdFloat};
use image::{DynamicImage, GenericImageView};
use std::simd::{Simd, StdFloat, u8x4, usizex4, f32x4};
use std::simd::num::SimdUint;

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

    pub fn sample_quad_test(&self, u: f32x4, v: f32x4) -> (f32x4, f32x4, f32x4, f32x4) {
        let mut r = [0.0f32; 4];
        let mut g = [0.0f32; 4];
        let mut b = [0.0f32; 4];
        let mut a = [0.0f32; 4];

        for lane in 0..4 {
            let (sr, sg, sb, sa) = self.sample(u[lane], v[lane]);
            r[lane] = sr as f32;
            g[lane] = sg as f32;
            b[lane] = sb as f32;
            a[lane] = sa as f32;
        }

        (f32x4::from_array(r), f32x4::from_array(g), f32x4::from_array(b), f32x4::from_array(a))
    }

    pub fn sample_quad(&self, u: f32x4, v: f32x4) -> (f32x4, f32x4, f32x4, f32x4) {
        let width  = self.width as f32;
        let height = self.height as f32;

        // Convert UV to pixel coords
        let x = (u * f32x4::splat(width  - 1.0)).cast::<usize>();
        let y = (v * f32x4::splat(height - 1.0)).cast::<usize>();

        // Index into texel (RGBA = 4 bytes)
        let idx: usizex4 = (y * usizex4::splat(self.width as usize) + x) * Simd::splat(4);

        let gathered_simd_r: f32x4 = Simd::gather_or_default(&self.rgba, idx).cast::<f32>();
        let gathered_simd_g: f32x4 = Simd::gather_or_default(&self.rgba, idx+Simd::splat(1)).cast::<f32>();
        let gathered_simd_b: f32x4 = Simd::gather_or_default(&self.rgba, idx+Simd::splat(2)).cast::<f32>();
        let gathered_simd_a: f32x4 = Simd::gather_or_default(&self.rgba, idx+Simd::splat(3)).cast::<f32>();
        //println!("{gathered_simd_r:?},{gathered_simd_g:?},{gathered_simd_b:?},{gathered_simd_a:?}");
        (gathered_simd_r, gathered_simd_g, gathered_simd_b, gathered_simd_a)
    }




} 
