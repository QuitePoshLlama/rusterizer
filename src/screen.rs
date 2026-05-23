use bytemuck::cast_slice_mut;
use std::fs::File;
use std::io::{Write, BufWriter};
use anyhow::Result;
use std::cell::UnsafeCell;

use crate::rectangle::Rect;
pub struct ScreenSpace {
    pub rect: Rect,
    pub width: u32,
    pub height: u32,
    pub rgba: UnsafeCell<Vec<u8>>,
    pub depth: UnsafeCell<Vec<f32>>,
}

unsafe impl Send for ScreenSpace {}
unsafe impl Sync for ScreenSpace {}

impl ScreenSpace {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            rect: Rect { min_x: 0, min_y: 0, max_x: 0, max_y: 0 },
            width,
            height,
            rgba: UnsafeCell::new(vec![0; size * 4]),
            depth: UnsafeCell::new(vec![f32::INFINITY; size]),
        }
    }

    #[inline]
    pub fn unsafe_set_pixel(&self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height { return; }
        let i = ((y * self.width + x) * 4) as usize;
        unsafe {
            let rgba = &mut *self.rgba.get();
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = a;
        }
    }

    #[inline]
    pub fn unsafe_set_depth(&self, x: u32, y: u32, value: f32) {
        if x >= self.width || y >= self.height { return; }
        let i = (y * self.width + x) as usize;
        unsafe {
            (&mut *self.depth.get())[i] = value;
        }
    }

    #[inline]
    pub fn get_depth(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return f32::INFINITY;
        }
        let i = (y * self.width + x) as usize;
        unsafe {
            (&*self.depth.get())[i]
        }
    }

    pub fn clear(&self, r: u8, g: u8, b: u8, a: u8) {
        let color = u32::from_le_bytes([r, g, b, a]);
        let rgba = unsafe { &mut *self.rgba.get() };
        let buf_as_u32: &mut [u32] = cast_slice_mut(rgba);
        buf_as_u32.fill(color);

        let depth = unsafe { &mut *self.depth.get() };
        depth.fill(f32::INFINITY);
    }

    pub fn write_bmp(&self, path: &str) -> Result<()> {
        let width = self.width;
        let height = self.height;
        let rgba = unsafe { &*self.rgba.get() };
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
                let r = rgba[i];
                let g = rgba[i + 1];
                let b = rgba[i + 2];
                file.write_all(&[b, g, r])?;
            }
            file.write_all(&padding)?;
        }
        Ok(())
    }
}
