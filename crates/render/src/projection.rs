//! Isometric projection, height lift, view position and scroll limits, and
//! screen-to-cell picking (S-0101).

/// Leptons per cell (S-0101 Constants).
pub const LEPTONS_PER_CELL: i32 = 256; // spec: S-0101 Constants (leptons per cell)

/// Cell-centre displacement (S-0101 Constants).
pub const CELL_CENTRE: i32 = 128; // spec: S-0101 Constants (cell centre displacement)

/// Terrain lift per height level, pixels (S-0101 claim 4).
pub const TERRAIN_LIFT: i32 = 15; // spec: S-0101 Constants (terrain lift per level)

/// Leptons per height level (S-0101 Constants).
pub const LEVEL_LEPTONS: i32 = 104; // spec: S-0101 Constants (leptons per height level)

/// Cursor probe depth (S-0101 claim 10).
pub const PROBE_DEPTH: i32 = 180; // spec: S-0101 Constants (cursor probe depth)

/// Quirk switches of S-0101 implemented here. Each defaults to the
/// original's behaviour (on).
///
/// Claims 3, 6 and 8 are departures with no switch (S-0101 Edge cases,
/// *Drawing and camera departures*): the exact height lift
/// ([`height_lift`]), the drawn-extent test (the map pass skips the items
/// whose bounds miss the view, `map_gpu`) and the centred oversized view
/// ([`limit_centre`]).
///
/// Not here (listed in `docs/development.md`, *Named switches of specs
/// whose behaviour is not implemented yet*):
/// `quirks.screen_to_ground_not_exact_inverse` (the off state's exact
/// inverse is not implemented; the cell under a pixel is F7's pixel
/// formula, the same in both settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionQuirks {
    /// `quirks.projection_truncates_toward_zero` (claim 2): both divisions
    /// by 256 of F1 truncate toward zero. Off: floor division.
    pub projection_truncates_toward_zero: bool,
    /// `quirks.cursor_probe_depth_180` (claim 10, F8): the cursor probes
    /// 180 rows. Off: at least 15 × the map's highest height level.
    pub cursor_probe_depth_180: bool,
}

impl Default for ProjectionQuirks {
    fn default() -> Self {
        // spec: S-0101 Edge cases and quirks (every switch defaults on)
        Self {
            projection_truncates_toward_zero: true,
            cursor_probe_depth_180: true,
        }
    }
}

impl ProjectionQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 2] {
        [
            (
                "projection_truncates_toward_zero",
                self.projection_truncates_toward_zero,
            ),
            ("cursor_probe_depth_180", self.cursor_probe_depth_180),
        ]
    }
}

/// F1: the world pixel of world position (X, Y) at height 0 (both
/// divisions truncate toward zero, claim 2), with the default switches.
pub fn project(x: i32, y: i32) -> (i32, i32) {
    project_with(x, y, &ProjectionQuirks::default())
}

/// F1 under the switches `q`.
pub fn project_with(x: i32, y: i32, q: &ProjectionQuirks) -> (i32, i32) {
    let (x, y) = (i64::from(x), i64::from(y));
    let (nx, ny) = (30 * x - 30 * y, 15 * x + 15 * y);
    let (qx, qy) = if q.projection_truncates_toward_zero {
        // spec: S-0101 F1 (q_x = trunc₀((30X − 30Y) / 256), q_y = trunc₀((15X + 15Y) / 256))
        (nx / 256, ny / 256)
    } else {
        // spec: S-0101 claim 2, Edge cases (quirks.projection_truncates_toward_zero off: floor division)
        (nx.div_euclid(256), ny.div_euclid(256))
    };
    (qx as i32, qy as i32)
}

/// F2: the object height lift *L(z)* in pixels for a world height of `z`
/// leptons, in exact integer arithmetic: the nearest integer to 15·z/104,
/// ties toward positive infinity, negative heights included (claim 3, a
/// departure with no switch).
pub fn height_lift(z: i32) -> i32 {
    // spec: S-0101 F2 (L(z) = ⌊15·z/104 + ½⌋ = ⌊(30·z + 104)/208⌋, exact rational arithmetic)
    (30 * i64::from(z) + 104).div_euclid(208) as i32
}

/// The world pixel of a world point at height `z` (F1 with *L(z)*, claim
/// 3), with the default switches.
pub fn project_lifted(x: i32, y: i32, z: i32) -> (i32, i32) {
    project_lifted_with(x, y, z, &ProjectionQuirks::default())
}

/// F1 with *L(z)* under the switches `q`.
pub fn project_lifted_with(x: i32, y: i32, z: i32, q: &ProjectionQuirks) -> (i32, i32) {
    let (qx, qy) = project_with(x, y, q);
    (qx, qy - height_lift(z))
}

/// The top corner of cell (x, y) at height level 0 (S-0101 F1:
/// (30(x − y), 15(x + y))).
pub fn cell_top(x: i32, y: i32) -> (i32, i32) {
    // spec: S-0101 F1 (a cell's top corner projects to (30(x − y), 15(x + y)))
    (30 * (x - y), 15 * (x + y))
}

/// The centre of cell (x, y) in leptons (glossary, S-0101 Constants).
pub fn cell_centre(x: i32, y: i32) -> (i32, i32) {
    (
        LEPTONS_PER_CELL * x + CELL_CENTRE,
        LEPTONS_PER_CELL * y + CELL_CENTRE,
    )
}

/// F3: the view origin of a view centre for a view of `size`.
pub fn view_origin(centre: (i32, i32), size: (i32, i32)) -> (i32, i32) {
    // spec: S-0101 F3 (O = centre − trunc₀(size / 2))
    (centre.0 - size.0 / 2, centre.1 - size.1 / 2)
}

/// F5: limits a requested view centre for a map of width `map_width`
/// (M_w) and local area `local` (V_x, V_y, V_w, V_h), with a view of
/// `size`. An axis on which the view is oversized takes the local area's
/// geometric centre, whatever the request (claim 8, a departure with no
/// switch).
pub fn limit_centre(
    requested: (i32, i32),
    size: (i32, i32),
    map_width: i32,
    local: [i32; 4],
) -> (i32, i32) {
    let (w, h) = size;
    let [vx, vy, vw, vh] = local;
    // spec: S-0101 F5 (x_min, x_max, y_min, y_max)
    let x_min = w / 2 + 30 * (2 * vx - map_width);
    let x_max = x_min + 60 * vw - w;
    let y_min = 15 * (map_width - 5 + 2 * vy) + h / 2;
    let y_max = y_min + 30 * vh + 135 - h;
    let limit = |c: i32, m: i32, big: i32, centre: i32| {
        if big < m {
            // spec: S-0101 claim 8, F5 (an oversized axis, M < m, ignores the request and takes the
            // local area's geometric centre)
            return centre;
        }
        // spec: S-0101 F5 (m for c < m, min(c, M) otherwise)
        if c < m { m } else { c.min(big) }
    };
    (
        // spec: S-0101 F5 (horizontal centre = 30·(2·V_x − M_w + V_w))
        limit(requested.0, x_min, x_max, 30 * (2 * vx - map_width + vw)),
        // spec: S-0101 F5 (vertical centre = 15·(M_w + 2·V_y + V_h))
        limit(requested.1, y_min, y_max, 15 * (map_width + 2 * vy + vh)),
    )
}

/// The local area's rectangle in world pixels, (x, y, width, height):
/// the part of the battlefield that F5's limits let a view show, for a
/// map of width `map_width` (M_w) and local area `local` (V_x, V_y, V_w,
/// V_h). A view that is not oversized lies inside it.
pub fn local_area_rect(map_width: i32, local: [i32; 4]) -> (i32, i32, i32, i32) {
    let [vx, vy, vw, vh] = local;
    // spec: S-0101 F5 (x_min − w/2 = 30·(2·V_x − M_w), x_max + w/2 − (x_min − w/2) = 60·V_w; y_min − h/2 = 15·(M_w − 5 + 2·V_y), y_max + h/2 − (y_min − h/2) = 30·V_h + 135)
    (
        30 * (2 * vx - map_width),
        15 * (map_width - 5 + 2 * vy),
        60 * vw,
        30 * vh + 135,
    )
}

/// F6: the ground position (X, Y) in leptons under world pixel
/// (q_x, q_y), in exact integer arithmetic (equal to the original's value
/// with the round-toward-zero mode on).
pub fn screen_to_ground(qx: i32, qy: i32) -> (i32, i32) {
    // spec: S-0101 F6 (X = trunc₀(a·(q_x + 2q_y)), Y = trunc₀(a·(2q_y − q_x)), a = 4 473 959 / 1 048 576)
    let a = |u: i64| (4_473_959 * u / 1_048_576) as i32;
    let (qx, qy) = (i64::from(qx), i64::from(qy));
    (a(qx + 2 * qy), a(2 * qy - qx))
}

/// F7: the flat-ground cell under world pixel (q_x, q_y).
pub fn flat_cell(qx: i32, qy: i32) -> (i32, i32) {
    // spec: S-0101 F7 (trunc₀((q_x + 2q_y) / 60), trunc₀((2q_y − q_x) / 60))
    ((qx + 2 * qy) / 60, (2 * qy - qx) / 60)
}

/// F8: the cell under the cursor at world pixel (q_x, q_y), given each
/// cell's height level and the highest height level on the map
/// (`max_level`, read only with `quirks.cursor_probe_depth_180` off).
pub fn cursor_cell(
    qx: i32,
    qy: i32,
    level: impl Fn(i32, i32) -> i32,
    max_level: i32,
    q: &ProjectionQuirks,
) -> (i32, i32) {
    let depth = if q.cursor_probe_depth_180 {
        // spec: S-0101 F8, Constants (cursor probe depth 180)
        PROBE_DEPTH
    } else {
        // spec: S-0101 claim 10, Edge cases (quirks.cursor_probe_depth_180 off: at least 15 × the highest level)
        PROBE_DEPTH.max(TERRAIN_LIFT * max_level)
    };
    // spec: S-0101 F8 (the farthest of the probe rows whose lifted cell reaches the cursor row)
    for r in (qy + 1..=qy + depth).rev() {
        let c = flat_cell(qx, r);
        if r - TERRAIN_LIFT * level(c.0, c.1) <= qy {
            return c;
        }
    }
    flat_cell(qx, qy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0101 F1 (cell (10, 4) centre projects to (180, 225))
    #[test]
    fn projection_example() {
        assert_eq!(project(2688, 1152), (180, 225));
        assert_eq!(cell_top(10, 4), (180, 210));
        // spec: S-0101 claim 2 (truncation toward zero around the diagonal)
        assert_eq!(project(100, 108).0, 0);
        assert_eq!(project(108, 100).0, 0);
    }

    /// spec: S-0101 F2 (check values of the exact lift)
    #[test]
    fn lift_check_values() {
        for (z, l) in [
            (-104, -15),
            (-4, -1),
            (0, 0),
            (400, 58),
            (416, 60),
            (1000, 144),
            (2080, 300),
            (3500, 505),
        ] {
            assert_eq!(height_lift(z), l, "L({z})");
        }
    }

    /// spec: S-0101 claim 3, F2, scenario M2 (f) (nearest integer, ties toward positive
    /// infinity, negative heights included)
    #[test]
    fn lift_rounding() {
        for (z, l) in [
            (-53, -8),
            (-52, -7),
            (-51, -7),
            (-4, -1),
            (0, 0),
            (51, 7),
            (52, 8),
            (53, 8),
        ] {
            assert_eq!(height_lift(z), l, "L({z})");
        }
        // The lifted point moves up by L(z).
        assert_eq!(project_lifted(0, 0, 2080), (0, -300));
    }

    /// spec: S-0101 claims 3 and 4, F2 (objects at whole levels lift exactly like terrain, at
    /// every level a map can hold, negative levels included)
    #[test]
    fn lift_whole_levels_match_terrain() {
        for level in -128..=127 {
            assert_eq!(
                height_lift(LEVEL_LEPTONS * level),
                TERRAIN_LIFT * level,
                "level {level}"
            );
        }
    }

    /// spec: S-0101 F5 (ordinary clamp)
    #[test]
    fn scroll_limits() {
        let local = [2, 4, 26, 22];
        let (w, h) = (640, 400);
        let (x_min, y_min) = (320 + 30 * (4 - 30), 15 * (30 - 5 + 8) + 200);
        assert_eq!(
            limit_centre((-99999, -99999), (w, h), 30, local),
            (x_min, y_min)
        );
        let x_max = x_min + 60 * 26 - w;
        let y_max = y_min + 30 * 22 + 135 - h;
        assert_eq!(
            limit_centre((99999, 99999), (w, h), 30, local),
            (x_max, y_max)
        );
        assert_eq!(
            limit_centre((x_min + 7, y_min + 9), (w, h), 30, local),
            (x_min + 7, y_min + 9)
        );
    }

    /// spec: S-0101 claim 8, F5 (an oversized axis takes the local area's geometric centre,
    /// whatever the request; the other axis keeps its ordinary limits)
    #[test]
    fn oversized_view_is_centred() {
        let local = [2, 4, 26, 22];
        // Horizontal centre 30·(2·2 − 30 + 26) = 0, vertical centre 15·(30 + 8 + 22) = 900.
        let (cx, cy) = (0, 900);
        // Too wide (60·26 = 1560 < 3000), ordinary height.
        let wide = (3000, 400);
        let y_min = 15 * (30 - 5 + 8) + 200;
        for rx in [-99999, -1, 0, 1, 99999] {
            assert_eq!(limit_centre((rx, -99999), wide, 30, local), (cx, y_min));
        }
        // Too tall (30·22 + 135 = 795 < 800), ordinary width.
        let tall = (640, 800);
        let x_min = 320 + 30 * (4 - 30);
        for ry in [-99999, 0, 900, 99999] {
            assert_eq!(limit_centre((-99999, ry), tall, 30, local), (x_min, cy));
        }
        // At 795 rows the vertical axis is not oversized (M = m).
        let exact = (640, 795);
        let m = 15 * (30 - 5 + 8) + 397;
        assert_eq!(limit_centre((0, 99999), exact, 30, local).1, m);
        // Oversized on both axes: both fixed.
        for r in [(-99999, -99999), (5, 5), (99999, 99999)] {
            assert_eq!(limit_centre(r, (3000, 800), 30, local), (cx, cy));
        }
        // The centred view shows the local area's middle: its left edge lies as far left of
        // the area's left edge as its right edge lies right of the area's right edge.
        let left = 60 * 2 - 30 * 30;
        let right = left + 60 * 26;
        let ox = view_origin((cx, cy), wide).0;
        assert_eq!(left - ox, ox + wide.0 - right);
    }

    // spec: S-0101 F5 (every limited view that is not oversized lies inside the local area's pixel rectangle, touching its edges at the limits) and claim 8 (an oversized axis is centred on it)
    #[test]
    fn limited_views_lie_inside_the_local_area() {
        let (mw, local) = (30, [2, 4, 26, 22]);
        let (lx, ly, lw, lh) = local_area_rect(mw, local);
        assert_eq!((lx, ly, lw, lh), (-780, 495, 1560, 795));
        for size in [(640, 400), (1560, 795), (800, 600)] {
            for r in [(-99999, -99999), (0, 900), (99999, 99999), (-99999, 99999)] {
                let o = view_origin(limit_centre(r, size, mw, local), size);
                assert!(o.0 >= lx && o.0 + size.0 <= lx + lw, "{size:?} {r:?}");
                assert!(o.1 >= ly && o.1 + size.1 <= ly + lh, "{size:?} {r:?}");
            }
        }
        let o = view_origin(limit_centre((-99999, 0), (640, 400), mw, local), (640, 400));
        assert_eq!(o.0, lx);
        let size = (3000, 1000);
        let o = view_origin(limit_centre((0, 0), size, mw, local), size);
        assert_eq!(lx - o.0, o.0 + size.0 - (lx + lw));
        // Vertically F5's centre 15·(M_w + 2·V_y + V_h) lies 7.5 pixels above the rectangle's middle.
        assert_eq!(o.1 + size.1 / 2, 15 * (mw + 2 * 4 + 22));
        assert!(o.1 < ly && o.1 + size.1 > ly + lh);
    }

    /// spec: S-0101 F6 (exact rational arithmetic, truncation toward zero)
    #[test]
    fn screen_to_ground_truncates_toward_zero() {
        assert_eq!(screen_to_ground(0, 0), (0, 0));
        // u = 60: 4 473 959 · 60 / 1 048 576 = 256.002, truncated to 256
        assert_eq!(screen_to_ground(0, 30), (256, 256));
        // u = ±30: ±128.001, both toward zero
        assert_eq!(screen_to_ground(30, 0), (128, -128));
        assert_eq!(screen_to_ground(-30, 0), (-128, 128));
        // u = 1 and −1: ±4.27, toward zero
        assert_eq!(screen_to_ground(1, 0), (4, -4));
        assert_eq!(screen_to_ground(30, 15), (256, 0));
    }

    /// spec: S-0101 F7 (the top corner and upper edges belong to the cell)
    #[test]
    fn flat_cells() {
        let (qx, qy) = cell_top(5, 3);
        assert_eq!(flat_cell(qx, qy), (5, 3));
        assert_eq!(flat_cell(qx, qy + 29), (5, 3));
        assert_eq!(flat_cell(qx, qy + 30), (6, 4));
    }

    /// spec: S-0101 Scenario M6 worked example (p = 10, plateau at level 4)
    #[test]
    fn cursor_on_a_cliff() {
        let level = |x: i32, y: i32| if x + y <= 20 { 4 } else { 0 };
        for (rows, cell) in [(180..210, 8), (210..240, 9), (240..330, 10), (330..360, 11)] {
            for r in rows {
                assert_eq!(
                    cursor_cell(0, r, level, 4, &ProjectionQuirks::default()),
                    (cell, cell),
                    "row {r}"
                );
            }
        }
    }

    /// spec: S-0101 claim 2, Edge cases (quirks.projection_truncates_toward_zero)
    #[test]
    fn projection_truncation_switch() {
        let on = ProjectionQuirks::default();
        let off = ProjectionQuirks {
            projection_truncates_toward_zero: false,
            ..on
        };
        // X < Y with a fractional column: one pixel further left with the switch off.
        assert_eq!(project_with(100, 108, &on), (0, 12));
        assert_eq!(project_with(100, 108, &off), (-1, 12));
        // X > Y: truncation and floor agree.
        assert_eq!(project_with(108, 100, &off), project_with(108, 100, &on));
        // Cell corners and centres have no fractional part: terrain does not move.
        for (x, y) in [(3, 5), (5, 3), (0, 9)] {
            assert_eq!(project_with(256 * x, 256 * y, &off), cell_top(x, y));
            let (cx, cy) = cell_centre(x, y);
            assert_eq!(project_with(cx, cy, &off), project_with(cx, cy, &on));
        }
    }

    /// spec: S-0101 claim 10, Edge cases (quirks.cursor_probe_depth_180)
    #[test]
    fn cursor_probe_depth_switch() {
        let on = ProjectionQuirks::default();
        let off = ProjectionQuirks {
            cursor_probe_depth_180: false,
            ..on
        };
        // A plateau at level 14 (lift 210) in front of level-0 ground: the
        // front plateau cell (10, 10) is drawn on rows 90 to 119.
        let level = |x: i32, y: i32| if x + y <= 20 { 14 } else { 0 };
        for y in 90..120 {
            // On: only 180 rows are probed, so the top rows of the drawn
            // diamond select the cell behind it.
            assert_eq!(cursor_cell(0, y, level, 14, &on), (9, 9), "row {y}");
            // Off: the pick follows the drawn lifted diamond.
            assert_eq!(cursor_cell(0, y, level, 14, &off), (10, 10), "row {y}");
        }
        // At level 12 or below both settings agree.
        let low = |x: i32, y: i32| if x + y <= 20 { 12 } else { 0 };
        for y in 0..400 {
            assert_eq!(
                cursor_cell(0, y, low, 12, &on),
                cursor_cell(0, y, low, 12, &off)
            );
        }
    }
}
