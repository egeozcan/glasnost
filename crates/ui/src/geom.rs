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

/// The smallest logical interface size a scale may leave: a larger scale is
/// lowered until the interface still fits.
// nocite: our own client setting (the smallest scaled interface, the size the screens are laid out for)
pub const MIN_UI: (i32, i32) = (640, 480);

/// The interface's scale: the screens lay themselves out in logical
/// pixels, drawn at `percent` / 100 screen pixels each, with the logical
/// origin at screen pixel `at`. Screen edges are the floors of the scaled
/// logical edges, so neighbouring rectangles stay adjacent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    /// Screen pixels per 100 logical pixels (100 or more).
    pub percent: i32,
    /// The logical origin's screen pixel.
    pub at: Point,
    /// The logical size of the screen.
    pub ui: (i32, i32),
}

/// ⌈a / b⌉ for b > 0.
const fn ceil_div(a: i32, b: i32) -> i32 {
    (a + b - 1).div_euclid(b)
}

impl Scale {
    /// No scaling on a screen of `size`.
    pub const fn one(size: (i32, i32)) -> Self {
        Self {
            percent: 100,
            at: Point::new(0, 0),
            ui: size,
        }
    }

    /// The scale `percent` gives on a screen of `size`, lowered so that the
    /// logical screen keeps at least [`MIN_UI`] (and never below 100). The
    /// logical screen covers the whole screen with its right edge on the
    /// screen's right edge; part of a logical pixel may pass the left and
    /// bottom edges.
    pub fn fit(percent: i32, size: (i32, i32)) -> Self {
        let (w, h) = (size.0.max(1), size.1.max(1));
        let cap = (w * 100 / MIN_UI.0).min(h * 100 / MIN_UI.1);
        let p = percent.min(cap).max(100);
        let ui = (ceil_div(w * 100, p), ceil_div(h * 100, p));
        Self {
            percent: p,
            at: Point::new(w - (ui.0 * p).div_euclid(100), 0),
            ui,
        }
    }

    /// Whether this scale draws logical pixels as they are.
    pub fn is_one(&self) -> bool {
        self.percent == 100 && self.at == Point::new(0, 0)
    }

    fn up(&self, v: i32) -> i32 {
        (v * self.percent).div_euclid(100)
    }

    /// The screen column of logical column `x`'s left edge.
    pub fn x(&self, x: i32) -> i32 {
        self.at.x + self.up(x)
    }

    /// The screen row of logical row `y`'s top edge.
    pub fn y(&self, y: i32) -> i32 {
        self.at.y + self.up(y)
    }

    /// The screen rectangle a logical rectangle covers.
    pub fn rect(&self, r: Rect) -> Rect {
        let (x0, y0) = (self.x(r.x), self.y(r.y));
        Rect::new(x0, y0, self.x(r.right()) - x0, self.y(r.bottom()) - y0)
    }

    /// The logical pixel a screen pixel lies in: the largest logical pixel
    /// whose screen edge is at or before it. This is the exact inverse of
    /// [`Scale::rect`]: a screen pixel is inside `rect(r)` exactly when its
    /// logical pixel is inside `r`.
    pub fn to_ui(&self, p: Point) -> Point {
        let down = |v: i32| ceil_div(100 * (v + 1), self.percent) - 1;
        Point::new(down(p.x - self.at.x), down(p.y - self.at.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scaled_rectangle_holds_exactly_the_pixels_that_map_back_into_it() {
        for size in [
            (640, 480),
            (1024, 768),
            (1366, 768),
            (1920, 1080),
            (2560, 1440),
        ] {
            for pct in [100, 125, 150, 200, 300] {
                let s = Scale::fit(pct, size);
                assert!(s.ui.0 >= MIN_UI.0 && s.ui.1 >= MIN_UI.1, "{size:?} {pct}");
                // The logical screen covers the screen, its right edge on the screen's.
                assert_eq!(s.x(s.ui.0), size.0, "{size:?} {pct}");
                assert!(s.x(0) <= 0 && s.y(0) == 0 && s.y(s.ui.1) >= size.1);
                for r in [
                    Rect::new(3, 5, 7, 2),
                    Rect::new(0, 0, 1, 1),
                    Rect::new(97, 41, 33, 18),
                ] {
                    let sr = s.rect(r);
                    for py in sr.y - 3..sr.bottom() + 3 {
                        for px in sr.x - 3..sr.right() + 3 {
                            let p = Point::new(px, py);
                            assert_eq!(sr.contains(p), r.contains(s.to_ui(p)), "{pct} {r:?} {p:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_scale_too_large_for_the_screen_is_lowered() {
        assert_eq!(Scale::fit(300, (1024, 768)).percent, 160);
        assert_eq!(Scale::fit(200, (1280, 960)).percent, 200);
        assert_eq!(Scale::fit(200, (800, 600)).percent, 125);
        assert_eq!(Scale::fit(150, (320, 200)).percent, 100);
        assert!(Scale::fit(100, (800, 600)).is_one());
        assert_eq!(Scale::fit(100, (800, 600)), Scale::one((800, 600)));
    }

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
