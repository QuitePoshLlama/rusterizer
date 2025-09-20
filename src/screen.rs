use bytemuck::cast_slice_mut;
use std::fs::File;
use std::io::{Write, BufWriter};
use anyhow::Result;
use std::simd::f32x4;
use std::simd::Mask;
use std::simd::u8x4;

use crate::rectangle::Rect;
pub struct ScreenSpace {
    pub rect: Rect,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub depth: Vec<f32>,
}

impl ScreenSpace {
    pub fn new(width: u32, height: u32) -> Self {
        let size_calc = (width * height) as usize;
        Self {
            rect: Rect { min_x: 0, min_y: 0, max_x: 0, max_y: 0 },
            width,
            height,
            rgba: vec![0; size_calc * 4],
            depth: vec![f32::INFINITY; size_calc],
        }
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8, alpha: u8) {
        if x >= self.width || y >= self.height { return; }
        let i = ((y * self.width + x) * 4) as usize;
        self.rgba[i] = red;
        self.rgba[i + 1] = green;
        self.rgba[i + 2] = blue;
        self.rgba[i + 3] = alpha;
    }
    pub fn set_pixel_quad(
        &mut self,
        x: u32,
        y: u32,
        r: u8x4,
        g: u8x4,
        b: u8x4,
        a: u8x4,
        mask: Mask<i32, 4>,
    ) {
        let base = (y * self.width + x) as usize;
        let base_row_down = ((y+1) * self.width + x) as usize;
        let rr = r.to_array();
        let gg = g.to_array();
        let bb = b.to_array();
        let aa = a.to_array();

        for lane in 0..2 {
            if mask.test(lane) {
                let idx = (base + lane) * 4;
                self.rgba[idx + 0] = rr[lane];
                self.rgba[idx + 1] = gg[lane];
                self.rgba[idx + 2] = bb[lane];
                self.rgba[idx + 3] = aa[lane];
            }
        }
        for lane in 2..4 {
            if mask.test(lane) {
                let idx = (base_row_down + lane - 2) * 4;
                self.rgba[idx + 0] = rr[lane];
                self.rgba[idx + 1] = gg[lane];
                self.rgba[idx + 2] = bb[lane];
                self.rgba[idx + 3] = aa[lane];
            }
        }
    }
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height { return None }
        let i = ((y * self.width + x) * 4) as usize;
        Some((self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]))
    }
    pub fn set_depth(&mut self, x: u32, y: u32, value: f32) {
        let i = (y * self.width + x) as usize;
        self.depth[i] = value;
    }
    pub fn set_depth_quad(&mut self, x: u32, y: u32, depth: f32x4, mask: Mask<i32, 4>) {
        let base = (y * self.width + x) as usize;
        let base_row_down = ((y+1) * self.width + x) as usize;
        let arr = depth.to_array();
        for lane in 0..2 {
            if mask.test(lane) {
                self.depth[base + lane] = arr[lane];
            }
        }
        for lane in 2..4 {
            if mask.test(lane) {
                self.depth[base_row_down + lane - 2] = arr[lane];
            }
        }
    }
    pub fn get_depth(&self, x: u32, y: u32) -> f32 {
        let i = (y * self.width + x) as usize;
        self.depth[i]
    }
    pub fn get_depth_quad(&self, x: u32, y: u32) -> f32x4 {
        // load 4 contiguous depths in a row
        let base = (y * self.width + x) as usize;
        let base_row_down = ((y+1) * self.width + x) as usize;
        f32x4::from_array([
            self.depth[base],
            self.depth[base + 1],
            self.depth[base_row_down],
            self.depth[base_row_down + 1],
        ])
    }
    pub fn clear(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let color: u32 = u32::from_le_bytes([r, g, b, a]);
        let buf_as_u32: &mut [u32] = cast_slice_mut(&mut self.rgba);
        buf_as_u32.fill(color);
        self.depth.fill(f32::INFINITY);
    }
    pub fn write_bmp(&self, path: &str) -> Result<()> {
        let width = self.width;
        let height = self.height;
        let row_stride = (3 * width + 3) & !3;
        let pixel_array_size = row_stride * height;
        let file_size = 54 + pixel_array_size;
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(b"BM")?;
        file.write_all(&(file_size as u32).to_le_bytes())?;
        file.write_all(&[0u8; 4])?;
        file.write_all(&54u32.to_le_bytes())?;
        file.write_all(&[40u8, 0, 0, 0])?;
        file.write_all(&(width as i32).to_le_bytes())?;
        file.write_all(&(height as i32).to_le_bytes())?;
        file.write_all(&[1, 0])?;
        file.write_all(&[24, 0])?;
        file.write_all(&[0u8; 4])?;
        file.write_all(&(pixel_array_size as u32).to_le_bytes())?;
        file.write_all(&[0u8; 4])?;
        file.write_all(&[0u8; 4])?;
        file.write_all(&[0u8; 4])?;
        file.write_all(&[0u8; 4])?;
        let padding = vec![0u8; (row_stride - width * 3) as usize];
        for y in (0..height).rev() {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                let r = self.rgba[i];
                let g = self.rgba[i + 1];
                let b = self.rgba[i + 2];
                file.write_all(&[b, g, r])?;
            }
            file.write_all(&padding)?;
        }
        Ok(())
    }
}
