#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

// helper methods
impl Rect {
    pub fn width(&self) -> u32 {
        self.max_x - self.min_x
    }
    pub fn height(&self) -> u32 {
        self.max_y - self.min_y
    }
}

// Compute minimum depth to get at least n rectangles for # of CPU cores
pub fn compute_subdivisions(n: usize) -> u32 {
    let mut depth = 0;
    let mut count = 1;
    while count < n {
        depth += 1;
        count *= 2;
    }
    depth
}
