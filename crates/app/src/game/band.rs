//! The band box's input state and geometry (S-0214 claims 18 and 19, F6
//! and State and ordering): a left-button press arms a drag, a motion
//! farther than the threshold from the press pixel makes it a box anchored
//! there, and the release selects. The frame's screen candidates follow
//! claims 20, 24 and 25 and F7. Input only, never game state.

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
    /// The moving endpoint as last updated by a motion of the active box,
    /// limited to the battlefield view (the press pixel before that).
    pub end: Point,
}

impl Drag {
    /// A left-button press in the battlefield arms a drag.
    pub fn armed(press: Point) -> Self {
        Self {
            press,
            active: false,
            end: press,
        }
    }

    /// A pointer motion to `p` (the limited pointer) with the battlefield
    /// view `view`: [`Drag::motion`], then an active box's endpoint follows
    /// `p` limited to the view.
    pub fn motion_in(&mut self, p: Point, mode: Rounding, view: Rect) {
        self.motion(p, mode);
        if self.active {
            // spec: S-0214 claim 18 and claim 34 (b) (while the box is active its moving endpoint follows the limited pointer, further limited to the battlefield view)
            self.end = limit(p, view);
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

/// The factor applied to each host drag metric for a right-button drag.
// spec: S-0214 claim 27, F8 and Constants (right-drag metric factor 2, applied separately to each axis)
pub const RIGHT_DRAG_FACTOR: i32 = 2;

/// Our engine's drag metric on each axis, pixels, in place of the host's.
// spec: S-0214 claim 27, F8 and Constants (our engine fixes both host drag metrics at 4 pixels, lead decision)
pub const RIGHT_DRAG_METRIC: i32 = 4;

/// A right-button press in the battlefield (S-0214 claims 26 to 28 and
/// F8): it anchors a possible scroll drag at the press pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RightDrag {
    /// The press pixel, the anchor.
    pub press: Point,
    /// A sampled motion has crossed F8: the gesture is a scroll drag until
    /// release.
    pub scrolling: bool,
}

impl RightDrag {
    /// An accepted right-button press anchors a possible scroll drag.
    pub fn armed(press: Point) -> Self {
        Self {
            press,
            scrolling: false,
        }
    }

    /// A sampled pointer motion while the button is held.
    pub fn motion(&mut self, p: Point) {
        let limit = RIGHT_DRAG_FACTOR * RIGHT_DRAG_METRIC;
        // spec: S-0214 F8 (the scroll drag starts when |Δx| > 2hₓ or |Δy| > 2hᵧ, exact integers, no radial distance; equality on both axes stays a click)
        // spec: S-0214 claim 27 (once started the classification stays until release, even back at the anchor)
        if (p.x - self.press.x).abs() > limit || (p.y - self.press.y).abs() > limit {
            self.scrolling = true;
        }
    }
}

/// The moving endpoint, limited to the battlefield view's last pixel on
/// each axis.
pub fn limit(p: Point, view: Rect) -> Point {
    // spec: S-0214 claim 18 (once dragging, the moving endpoint is limited to the battlefield view's last pixel on each axis)
    Point::new(p.x.min(view.right() - 1), p.y.min(view.bottom() - 1))
}

/// The limited pointer of S-0214 claim 34 (a): screen pixel `p` with each
/// coordinate limited to the client area of `screen` pixels.
pub fn limited(p: Point, screen: (i32, i32)) -> Point {
    // spec: S-0214 claim 34 (a) (x limited to 0 … B − 1 and y to 0 … C − 1 of the client area)
    Point::new(
        p.x.clamp(0, (screen.0 - 1).max(0)),
        p.y.clamp(0, (screen.1 - 1).max(0)),
    )
}

/// Whether screen pixel `p` lies outside the client area of `screen`
/// pixels (S-0214 claim 34).
pub fn outside(p: Point, screen: (i32, i32)) -> bool {
    p.x < 0 || p.y < 0 || p.x >= screen.0 || p.y >= screen.1
}

/// What a left click released outside the window acts on (S-0214 claim 34
/// (c)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffView {
    /// The cell under the release pixel, projected beyond the view's edge.
    Projected,
    /// The cell at map cell coordinates (0, 0).
    OriginCell,
    /// Nothing: no order, and the selection stays.
    Nothing,
}

/// Where a left click released at screen pixel `p` outside the window acts,
/// for the battlefield view `view`, with `quirks.offview_release_origin_cell`
/// as `origin_cell`.
pub fn release_outside(p: Point, view: Rect, origin_cell: bool) -> OffView {
    // spec: S-0214 claim 34 (c) (a release pixel to the left of or above the view on either axis, beyond its upper-right and lower-left corners included)
    if p.x < view.x || p.y < view.y {
        if origin_cell {
            // spec: S-0214 claim 34 (c) and Edge cases (quirks.offview_release_origin_cell on, the original: the cell at map cell coordinates (0, 0))
            OffView::OriginCell
        } else {
            // spec: S-0214 Edge cases (quirks.offview_release_origin_cell off: no order, and the selection does not change)
            OffView::Nothing
        }
    } else {
        // spec: S-0214 claim 34 (c) (otherwise the cell under the release pixel, projected beyond the view's edge as a pixel inside it)
        OffView::Projected
    }
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

/// How far outside the battlefield view a mobile candidate's point may lie,
/// pixels on each axis.
// spec: S-0214 F7 and Constants (mobile candidate margin 32 pixels on each axis, independent of image dimensions)
pub const MOBILE_MARGIN: i32 = 32;

/// Whether a mobile object's box point `p`, relative to the battlefield
/// view's top-left corner, passes F7 for a view of `w` × `h` pixels.
pub fn mobile_point_in(p: Point, w: i32, h: i32) -> bool {
    // spec: S-0214 F7 (−32 ≤ x ≤ W + 32 and −32 ≤ y ≤ H + 32, inclusive integer comparisons; the upper bound is W + 32, not W − 1 + 32)
    (-MOBILE_MARGIN..=w + MOBILE_MARGIN).contains(&p.x)
        && (-MOBILE_MARGIN..=h + MOBILE_MARGIN).contains(&p.y)
}

/// Whether a building's box point `p`, relative to the battlefield view's
/// top-left corner, passes F7 for a view of `w` × `h` pixels.
pub fn building_point_in(p: Point, w: i32, h: i32) -> bool {
    // spec: S-0214 F7 (0 ≤ x ≤ W and 0 ≤ y ≤ H, the unexpanded view including its right and bottom boundary coordinates)
    (0..=w).contains(&p.x) && (0..=h).contains(&p.y)
}

/// The screen candidates admitted for a frame (S-0214 claims 20, 24 and
/// 25): every qualifying building, then every qualifying mobile object,
/// cut at [`SCREEN_CANDIDATES`]. Every candidate consumes a place whether
/// or not a box could take it; places are not replenished. `buildings`
/// come in creation order and `mobile` in drawing order (the callers').
pub fn admit<T>(
    buildings: impl IntoIterator<Item = T>,
    mobile: impl IntoIterator<Item = T>,
) -> Vec<T> {
    // spec: S-0214 claims 20 and 25 (qualifying buildings precede every qualifying mobile object; the first 500 in that order are admitted; ineligible objects still consume places)
    buildings
        .into_iter()
        .chain(mobile)
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

    // spec: S-0214 claim 34 (a) (the limited pointer: each coordinate limited to the client area)
    #[test]
    fn claim_34_a_limited_pointer() {
        let screen = (800, 600);
        assert_eq!(limited(Point::new(-30, 700), screen), Point::new(0, 599));
        assert_eq!(limited(Point::new(900, -5), screen), Point::new(799, 0));
        assert_eq!(limited(Point::new(12, 34), screen), Point::new(12, 34));
        assert!(outside(Point::new(-1, 0), screen));
        assert!(outside(Point::new(0, 600), screen));
        assert!(!outside(Point::new(799, 599), screen));
    }

    // spec: S-0214 claim 34 (b) (the endpoint as last updated by a motion of the active box, limited to the view, not the release pixel)
    #[test]
    fn claim_34_b_the_endpoint_as_last_updated() {
        let mode = Rounding::default();
        let view = Rect::new(0, 0, 640, 480);
        let mut d = Drag::armed(Point::new(100, 100));
        d.motion_in(Point::new(102, 100), mode, view);
        assert_eq!(d.end, Point::new(100, 100), "a click so far");
        d.motion_in(Point::new(20, 30), mode, view);
        assert_eq!(d.end, Point::new(20, 30));
        d.motion_in(limited(Point::new(-50, 700), (800, 600)), mode, view);
        assert_eq!(d.end, Point::new(0, 479), "on the view's boundary");
    }

    // spec: S-0214 claim 34 (c) and Edge cases (left of or above the view on either axis: the origin cell with the quirk on, nothing with it off; otherwise the projected cell)
    #[test]
    fn claim_34_c_release_outside() {
        let view = Rect::new(0, 0, 640, 600);
        for p in [
            Point::new(-1, 300),
            Point::new(300, -1),
            Point::new(900, -20),
            Point::new(-20, 700),
        ] {
            assert_eq!(release_outside(p, view, true), OffView::OriginCell, "{p:?}");
            assert_eq!(release_outside(p, view, false), OffView::Nothing, "{p:?}");
        }
        for p in [Point::new(900, 300), Point::new(300, 700)] {
            assert_eq!(release_outside(p, view, true), OffView::Projected, "{p:?}");
            assert_eq!(release_outside(p, view, false), OffView::Projected, "{p:?}");
        }
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

    fn right_moved(dx: i32, dy: i32) -> bool {
        let mut d = RightDrag::armed(Point::new(100, 100));
        d.motion(Point::new(100 + dx, 100 + dy));
        d.scrolling
    }

    // spec: S-0214 claim 27 and F8 (with metrics of 4: moves of (8, 0), (0, 8) and (8, 8) stay a click; (9, 0), (0, −9) and (9, 1) start the scroll drag; the test is per axis, not radial)
    #[test]
    fn claim_27_right_drag_threshold() {
        assert!(!right_moved(8, 0));
        assert!(!right_moved(0, 8));
        assert!(!right_moved(8, 8));
        assert!(!right_moved(-8, -8));
        assert!(right_moved(9, 0));
        assert!(right_moved(0, -9));
        assert!(right_moved(9, 1));
    }

    // spec: S-0214 claim 27 (a started scroll drag stays one when the pointer returns to its anchor)
    #[test]
    fn claim_27_right_drag_latches() {
        let mut d = RightDrag::armed(Point::new(10, 10));
        d.motion(Point::new(30, 10));
        d.motion(Point::new(10, 10));
        assert!(d.scrolling);
        assert_eq!(d.press, Point::new(10, 10));
    }

    // spec: S-0214 claims 20 and 25 and Scenario T15 part B (20 qualifying buildings and 490 mobile objects: all 20 buildings and the first 480 mobile objects; 500 mobile objects and one vehicle-like building: the building and 499 mobile objects)
    #[test]
    fn claims_20_25_buildings_first() {
        let buildings: Vec<u32> = (1000..1020).collect();
        let mobile: Vec<u32> = (0..490).collect();
        let a = admit(buildings.clone(), mobile.clone());
        assert_eq!(a.len(), 500);
        assert_eq!(&a[..20], &buildings[..]);
        assert_eq!(&a[20..], &mobile[..480]);
        let crowd: Vec<u32> = (0..500).collect();
        let a = admit([2000], crowd.clone());
        assert_eq!(a.len(), 500);
        assert_eq!(a[0], 2000);
        assert_eq!(&a[1..], &crowd[..499]);
        // Few candidates: all admitted, buildings first.
        assert_eq!(admit([9], [1, 2]), [9, 1, 2]);
    }

    // spec: S-0214 F7 (mobile points pass from −32 through the view dimension + 32 inclusive on each axis; −33 and dimension + 33 fail)
    #[test]
    fn f7_mobile_bounds() {
        let (w, h) = (800, 600);
        for x in [-32, -1, 0, w, w + 32] {
            assert!(mobile_point_in(Point::new(x, 300), w, h), "{x}");
        }
        for x in [-33, w + 33] {
            assert!(!mobile_point_in(Point::new(x, 300), w, h), "{x}");
        }
        for y in [-32, h + 32] {
            assert!(mobile_point_in(Point::new(400, y), w, h), "{y}");
        }
        for y in [-33, h + 33] {
            assert!(!mobile_point_in(Point::new(400, y), w, h), "{y}");
        }
    }

    // spec: S-0214 F7 (building points pass from 0 through the view dimension inclusive; −1 and dimension + 1 fail)
    #[test]
    fn f7_building_bounds() {
        let (w, h) = (800, 600);
        for x in [0, w] {
            assert!(building_point_in(Point::new(x, 300), w, h), "{x}");
        }
        for x in [-1, w + 1] {
            assert!(!building_point_in(Point::new(x, 300), w, h), "{x}");
        }
        for y in [0, h] {
            assert!(building_point_in(Point::new(400, y), w, h), "{y}");
        }
        for y in [-1, h + 1] {
            assert!(!building_point_in(Point::new(400, y), w, h), "{y}");
        }
    }
}
