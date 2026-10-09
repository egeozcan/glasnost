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

/// Object lift scale *k* (S-0101 F2), pixels per lepton.
pub const LIFT_SCALE: f64 = 0.143_503_600_826_600_85; // spec: S-0101 F2 (k)

/// Lift correction threshold (S-0101 F2).
pub const LIFT_THRESHOLD: i32 = 728; // spec: S-0101 Constants (lift correction threshold)

/// Cursor probe depth (S-0101 claim 10).
pub const PROBE_DEPTH: i32 = 180; // spec: S-0101 Constants (cursor probe depth)

/// F1: the world pixel of world position (X, Y) at height 0 (both
/// divisions truncate toward zero, claim 2).
pub fn project(x: i32, y: i32) -> (i32, i32) {
    let (x, y) = (i64::from(x), i64::from(y));
    // spec: S-0101 F1 (q_x = trunc₀((30X − 30Y) / 256), q_y = trunc₀((15X + 15Y) / 256))
    let qx = (30 * x - 30 * y) / 256;
    let qy = (15 * x + 15 * y) / 256;
    (qx as i32, qy as i32)
}

/// F2: the object height lift *L(z)* in pixels for a world height of `z`
/// leptons. Rounding to nearest at the three inner steps gives the same
/// result as the original's rounding toward zero at every height a map can
/// produce (F2), so plain double arithmetic is used; the final conversion
/// truncates toward zero.
pub fn height_lift(z: i32) -> i32 {
    // spec: S-0101 F2 (L(z) = trunc₀((z·k + [z ≥ 728]) + 0.5))
    let corr = if z >= LIFT_THRESHOLD { 1.0 } else { 0.0 };
    ((f64::from(z) * LIFT_SCALE + corr) + 0.5) as i32
}

/// The world pixel of a world point at height `z` (F1 with *L(z)*, claim 3).
pub fn project_lifted(x: i32, y: i32, z: i32) -> (i32, i32) {
    let (qx, qy) = project(x, y);
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

/// Scroll-limit switches of S-0101.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollQuirks {
    /// `quirks.oversized_view_clamp`.
    pub oversized_view_clamp: bool,
}

impl Default for ScrollQuirks {
    fn default() -> Self {
        // spec: S-0101 Edge cases (default: on)
        Self {
            oversized_view_clamp: true,
        }
    }
}

/// F5: limits a requested view centre for a map of width `map_width`
/// (M_w) and local area `local` (V_x, V_y, V_w, V_h), with a view of
/// `size`.
pub fn limit_centre(
    requested: (i32, i32),
    size: (i32, i32),
    map_width: i32,
    local: [i32; 4],
    q: &ScrollQuirks,
) -> (i32, i32) {
    let (w, h) = size;
    let [vx, vy, vw, vh] = local;
    // spec: S-0101 F5 (x_min, x_max, y_min, y_max)
    let x_min = w / 2 + 30 * (2 * vx - map_width);
    let x_max = x_min + 60 * vw - w;
    let y_min = 15 * (map_width - 5 + 2 * vy) + h / 2;
    let y_max = y_min + 30 * vh + 135 - h;
    let limit = |c: i32, m: i32, big: i32| {
        if big < m && !q.oversized_view_clamp {
            // spec: S-0101 Edge cases (quirk off: centre the local area in the view)
            return (m + big) / 2;
        }
        // spec: S-0101 F5 (m for c < m, min(c, M) otherwise)
        if c < m { m } else { c.min(big) }
    };
    (
        limit(requested.0, x_min, x_max),
        limit(requested.1, y_min, y_max),
    )
}

/// F7: the flat-ground cell under world pixel (q_x, q_y).
pub fn flat_cell(qx: i32, qy: i32) -> (i32, i32) {
    // spec: S-0101 F7 (trunc₀((q_x + 2q_y) / 60), trunc₀((2q_y − q_x) / 60))
    ((qx + 2 * qy) / 60, (2 * qy - qx) / 60)
}

/// F8: the cell under the cursor at world pixel (q_x, q_y), given each
/// cell's height level.
pub fn cursor_cell(qx: i32, qy: i32, level: impl Fn(i32, i32) -> i32) -> (i32, i32) {
    // spec: S-0101 F8 (the farthest of the 180 probe rows whose lifted cell reaches the cursor row)
    for r in (qy + 1..=qy + PROBE_DEPTH).rev() {
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

    /// spec: S-0101 F2 (check values)
    #[test]
    fn lift_check_values() {
        for (z, l) in [
            (0, 0),
            (3, 0),
            (4, 1),
            (103, 15),
            (104, 15),
            (727, 104),
            (728, 105),
            (1456, 210),
            (1500, 216),
            (2000, 288),
            (2080, 299),
            (3000, 432),
            (3500, 503),
            (-1, 0),
            (-3, 0),
            (-4, 0),
            (-104, -14),
            (400, 57),
            (416, 60),
            (1000, 145),
        ] {
            assert_eq!(height_lift(z), l, "L({z})");
        }
        for level in 0..20 {
            assert_eq!(height_lift(104 * level), 15 * level);
        }
    }

    /// spec: S-0101 F5 (ordinary clamp and the oversized-view rule)
    #[test]
    fn scroll_limits() {
        let q = ScrollQuirks::default();
        let local = [2, 4, 26, 22];
        let (w, h) = (640, 400);
        let (x_min, y_min) = (320 + 30 * (4 - 30), 15 * (30 - 5 + 8) + 200);
        assert_eq!(
            limit_centre((-99999, -99999), (w, h), 30, local, &q),
            (x_min, y_min)
        );
        let x_max = x_min + 60 * 26 - w;
        assert_eq!(limit_centre((99999, y_min), (w, h), 30, local, &q).0, x_max);
        // Oversized: every request at or above the minimum gives the maximum.
        let wide = (3000, h);
        let m = 1500 + 30 * (4 - 30);
        let big = m + 60 * 26 - 3000;
        assert_eq!(limit_centre((m - 1, y_min), wide, 30, local, &q).0, m);
        assert_eq!(limit_centre((m, y_min), wide, 30, local, &q).0, big);
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
                assert_eq!(cursor_cell(0, r, level), (cell, cell), "row {r}");
            }
        }
    }
}
