//! The band box's input state and geometry (S-0214 claims 18 and 19, F6
//! and State and ordering): a primary-button press arms a drag, a motion
//! farther than the threshold from the press pixel makes it a box anchored
//! there, and the release selects. Input only, never game state.

use glasnost_num::Rounding;
use glasnost_num::geom::root_floor;
use glasnost_ui::geom::{Point, Rect};

/// The drag threshold, pixels of truncated table-root distance.
// spec: S-0214 claim 18, F6 and Constants (drag begins exactly when d > 4)
pub const DRAG_THRESHOLD: i64 = 4;

/// F6's distance of `p` from the press pixel: ⌊r(Δx² + Δy²)⌋ with the table
/// square root.
pub fn distance(press: Point, p: Point, mode: Rounding) -> i64 {
    let (dx, dy) = (i64::from(p.x - press.x), i64::from(p.y - press.y));
    // spec: S-0214 F6 (d = ⌊r(Δx² + Δy²)⌋ pixels, the table square root of S-0204 F1, truncated toward zero)
    root_floor((dx * dx + dy * dy) as u64, true, mode) as i64
}

/// An armed drag (table *State and ordering*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drag {
    /// The press pixel, the box's anchor.
    pub press: Point,
    /// The box is active: a motion has crossed the threshold.
    pub active: bool,
}

impl Drag {
    /// A primary-button press in the battlefield arms a drag.
    pub fn armed(press: Point) -> Self {
        Self {
            press,
            active: false,
        }
    }

    /// A pointer motion: the box becomes active once F6's distance from the
    /// press pixel exceeds the threshold, and stays active when the pointer
    /// comes back closer.
    pub fn motion(&mut self, p: Point, mode: Rounding) {
        // spec: S-0214 claim 18 (the distance is from the press pixel, not the previous motion; returning closer does not turn it back into a click)
        if !self.active && distance(self.press, p, mode) > DRAG_THRESHOLD {
            self.active = true;
        }
    }
}

/// The moving endpoint, limited to the battlefield view's last pixel on
/// each axis.
pub fn limit(p: Point, view: Rect) -> Point {
    // spec: S-0214 claim 18 (once dragging, the moving endpoint is limited to the battlefield view's last pixel on each axis)
    Point::new(p.x.min(view.right() - 1), p.y.min(view.bottom() - 1))
}

/// Whether box point `q` is inside the box from `press` to `end`: all four
/// edges included.
pub fn inside(press: Point, end: Point, q: Point) -> bool {
    // spec: S-0214 F6 (min(a, c) ≤ x ≤ max(a, c) and min(b, e) ≤ y ≤ max(b, e), exact integers)
    (press.x.min(end.x)..=press.x.max(end.x)).contains(&q.x)
        && (press.y.min(end.y)..=press.y.max(end.y)).contains(&q.y)
}

/// The most screen candidates admitted in one drawn frame.
// spec: S-0214 claim 20 and Constants (500 screen candidates per drawn frame; not a selection limit)
pub const SCREEN_CANDIDATES: usize = 500;

/// The screen candidates admitted for a frame (S-0214 claim 20): mobile
/// candidates first, then buildings, cut at [`SCREEN_CANDIDATES`]. Every
/// candidate consumes a place whether or not a box could take it; places
/// are not replenished. The order inside each group is the callers'.
pub fn admit<T>(
    mobile: impl IntoIterator<Item = T>,
    buildings: impl IntoIterator<Item = T>,
) -> Vec<T> {
    // spec: S-0214 claim 20 (mobile drawing candidates consume places before the later building candidates; ineligible objects still consume places)
    mobile
        .into_iter()
        .chain(buildings)
        .take(SCREEN_CANDIDATES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moved(dx: i32, dy: i32) -> bool {
        let mut d = Drag::armed(Point::new(100, 100));
        d.motion(Point::new(100 + dx, 100 + dy), Rounding::default());
        d.active
    }

    // spec: S-0214 claim 18 and F6 (moves of (4, 0) and (3, 3) remain clicks; (5, 0) and (4, 4) begin boxes)
    #[test]
    fn claim_18_threshold() {
        assert!(!moved(4, 0));
        assert!(!moved(3, 3));
        assert!(moved(5, 0));
        assert!(moved(4, 4));
        assert!(moved(-4, -4));
    }

    // spec: S-0214 claim 18 (returning inside the threshold after crossing it keeps the box, anchored at the press pixel)
    #[test]
    fn claim_18_return_inside_stays_a_box() {
        let mode = Rounding::default();
        let mut d = Drag::armed(Point::new(10, 10));
        d.motion(Point::new(13, 13), mode);
        assert!(!d.active);
        d.motion(Point::new(20, 10), mode);
        d.motion(Point::new(11, 10), mode);
        assert!(d.active);
        assert_eq!(d.press, Point::new(10, 10));
    }

    // spec: S-0214 claim 18 (the endpoint is limited to the view's last pixel on each axis)
    #[test]
    fn claim_18_endpoint_limit() {
        let view = Rect::new(0, 0, 640, 480);
        assert_eq!(limit(Point::new(700, 500), view), Point::new(639, 479));
        assert_eq!(limit(Point::new(30, 40), view), Point::new(30, 40));
    }

    // spec: S-0214 F6 (the box includes its four edges, whichever corner the drag ends at)
    #[test]
    fn f6_inclusive_edges() {
        let (a, c) = (Point::new(10, 20), Point::new(4, 30));
        assert!(inside(a, c, Point::new(4, 20)));
        assert!(inside(a, c, Point::new(10, 30)));
        assert!(!inside(a, c, Point::new(11, 25)));
        assert!(!inside(a, c, Point::new(7, 19)));
    }

    // spec: S-0214 claim 20 (at most 500 screen candidates, mobile before buildings; ineligible candidates consume places without replenishment)
    #[test]
    fn claim_20_candidate_capacity() {
        let mobile: Vec<u32> = (0..490).collect();
        let buildings: Vec<u32> = (1000..1020).collect();
        let a = admit(mobile.clone(), buildings.clone());
        assert_eq!(a.len(), 500);
        assert_eq!(&a[..490], &mobile[..]);
        assert_eq!(&a[490..], &buildings[..10]);
        let crowd: Vec<u32> = (0..510).collect();
        let a = admit(crowd, buildings);
        assert_eq!(a.len(), 500);
        assert!(a.iter().all(|&x| x < 1000), "no building gets a place");
        // Few candidates: all admitted, buildings after mobiles.
        assert_eq!(admit([1, 2], [9]), [1, 2, 9]);
    }
}
