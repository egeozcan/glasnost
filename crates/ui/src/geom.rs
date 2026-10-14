//! Integer screen geometry in pixels, top-left origin.

/// A point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// Column.
    pub x: i32,
    /// Row.
    pub y: i32,
}

impl Point {
    /// A point.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A rectangle: `x .. x + w` by `y .. y + h`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    /// Left column.
    pub x: i32,
    /// Top row.
    pub y: i32,
    /// Width (never negative).
    pub w: i32,
    /// Height (never negative).
    pub h: i32,
}

impl Rect {
    /// A rectangle; negative sizes become 0.
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x,
            y,
            w: if w < 0 { 0 } else { w },
            h: if h < 0 { 0 } else { h },
        }
    }

    /// One past the right column.
    pub const fn right(&self) -> i32 {
        self.x + self.w
    }

    /// One past the bottom row.
    pub const fn bottom(&self) -> i32 {
        self.y + self.h
    }

    /// Whether `p` lies inside.
    pub const fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.right() && p.y < self.bottom()
    }

    /// Whether the rectangle covers no pixel.
    pub const fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// The overlap of two rectangles (empty when they do not meet).
    pub fn intersect(&self, o: &Rect) -> Rect {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        Rect::new(
            x,
            y,
            self.right().min(o.right()) - x,
            self.bottom().min(o.bottom()) - y,
        )
    }

    /// The rectangle shrunk by `d` on every side.
    pub fn inset(&self, d: i32) -> Rect {
        Rect::new(self.x + d, self.y + d, self.w - 2 * d, self.h - 2 * d)
    }

    /// The centre (rounded toward the top left).
    pub const fn centre(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    /// The smallest rectangle holding two corner points (both included).
    pub fn spanning(a: Point, b: Point) -> Rect {
        let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
        let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
        Rect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_is_half_open() {
        let r = Rect::new(10, 20, 5, 4);
        assert!(r.contains(Point::new(10, 20)));
        assert!(r.contains(Point::new(14, 23)));
        assert!(!r.contains(Point::new(15, 23)));
        assert!(!r.contains(Point::new(14, 24)));
        assert!(!r.contains(Point::new(9, 20)));
    }

    #[test]
    fn intersect_inset_and_span() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 6, 10, 10);
        assert_eq!(a.intersect(&b), Rect::new(5, 6, 5, 4));
        assert!(a.intersect(&Rect::new(20, 20, 3, 3)).is_empty());
        assert_eq!(a.inset(2), Rect::new(2, 2, 6, 6));
        assert_eq!(a.inset(6), Rect::new(6, 6, 0, 0));
        assert_eq!(
            Rect::spanning(Point::new(5, 9), Point::new(2, 3)),
            Rect::new(2, 3, 4, 7)
        );
    }
}
