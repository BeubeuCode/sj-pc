#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectPx {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl RectPx {
    pub fn contains(&self, point_x: i32, point_y: i32) -> bool {
        let inside_x = point_x >= self.x && point_x < self.x + self.width as i32;
        let inside_y = point_y >= self.y && point_y < self.y + self.height as i32;
        inside_x && inside_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: RectPx = RectPx {
        x: 10,
        y: 20,
        width: 30,
        height: 40,
    };

    #[test]
    fn contains_top_left_corner() {
        assert!(RECT.contains(10, 20));
    }

    #[test]
    fn excludes_right_and_bottom_edges() {
        assert!(!RECT.contains(40, 20));
        assert!(!RECT.contains(10, 60));
    }
}
