/// A point in physical screen coordinates.
///
/// Coordinates may be negative on multi-monitor desktops where a display is
/// positioned left of or above the primary monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl ScreenPoint {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A rectangle in physical screen coordinates.
///
/// The rectangle uses half-open bounds: `x <= point.x < right` and
/// `y <= point.y < bottom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ScreenRect {
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[must_use]
    pub fn right(self) -> i64 {
        i64::from(self.x) + i64::from(self.width)
    }

    #[must_use]
    pub fn bottom(self) -> i64 {
        i64::from(self.y) + i64::from(self.height)
    }

    #[must_use]
    pub fn contains(self, point: ScreenPoint) -> bool {
        let x = i64::from(point.x);
        let y = i64::from(point.y);

        !self.is_empty()
            && x >= i64::from(self.x)
            && x < self.right()
            && y >= i64::from(self.y)
            && y < self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::{ScreenPoint, ScreenRect};

    #[test]
    fn contains_uses_half_open_bounds() {
        let rect = ScreenRect::new(10, 20, 30, 40);

        assert!(rect.contains(ScreenPoint::new(10, 20)));
        assert!(rect.contains(ScreenPoint::new(39, 59)));
        assert!(!rect.contains(ScreenPoint::new(40, 59)));
        assert!(!rect.contains(ScreenPoint::new(39, 60)));
    }

    #[test]
    fn empty_rect_contains_no_points() {
        let rect = ScreenRect::new(0, 0, 0, 10);

        assert!(!rect.contains(ScreenPoint::new(0, 0)));
    }
}
