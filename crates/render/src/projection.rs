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

/// Quirk switches of S-0101 implemented here. Each defaults to the
/// original's behaviour (on).
///
/// Not here (listed in `docs/development.md`, *Named switches of specs
/// whose behaviour is not implemented yet*):
/// `quirks.screen_band_tests_position_only` (our renderer does none of
/// claim 6's on-screen tests) and `quirks.screen_to_ground_not_exact_inverse`
/// (no F6 lepton position is used; the cell under a pixel is F7's pixel
/// formula, the same in both settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionQuirks {
    /// `quirks.projection_truncates_toward_zero` (claim 2): both divisions
    /// by 256 of F1 truncate toward zero. Off: floor division.
    pub projection_truncates_toward_zero: bool,
    /// `quirks.height_lift_float_scale` (claims 3 and 4, F2): the object
    /// height lift is F2. Off: the exact scale ⌊15·z/104 + ½⌋.
    pub height_lift_float_scale: bool,
    /// `quirks.oversized_view_clamp` (claim 8): on an axis where the view
    /// is larger than the local area, F5 gives the minimum below it and the
    /// maximum otherwise. Off: the local area is centred in the view.
    pub oversized_view_clamp: bool,
    /// `quirks.cursor_probe_depth_180` (claim 10, F8): the cursor probes
    /// 180 rows. Off: at least 15 × the map's highest height level.
    pub cursor_probe_depth_180: bool,
}

impl Default for ProjectionQuirks {
    fn default() -> Self {
        // spec: S-0101 Edge cases and quirks (every switch defaults on)
        Self {
            projection_truncates_toward_zero: true,
            height_lift_float_scale: true,
            oversized_view_clamp: true,
            cursor_probe_depth_180: true,
        }
    }
}

impl ProjectionQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 4] {
        [
            (
                "projection_truncates_toward_zero",
                self.projection_truncates_toward_zero,
            ),
            ("height_lift_float_scale", self.height_lift_float_scale),
            ("oversized_view_clamp", self.oversized_view_clamp),
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
/// leptons, with the default switches. Rounding to nearest at the three
/// inner steps gives the same result as the original's rounding toward
/// zero at every height a map can produce (F2), so plain double arithmetic
/// is used; the final conversion truncates toward zero.
pub fn height_lift(z: i32) -> i32 {
    height_lift_with(z, &ProjectionQuirks::default())
}

/// *L(z)* under the switches `q`.
pub fn height_lift_with(z: i32, q: &ProjectionQuirks) -> i32 {
    if !q.height_lift_float_scale {
        // spec: S-0101 claim 3, Edge cases (quirks.height_lift_float_scale off: ⌊15·z/104 + ½⌋)
        return (30 * i64::from(z) + 104).div_euclid(208) as i32;
    }
    // spec: S-0101 F2 (L(z) = trunc₀((z·k + [z ≥ 728]) + 0.5))
    let corr = if z >= LIFT_THRESHOLD { 1.0 } else { 0.0 };
    ((f64::from(z) * LIFT_SCALE + corr) + 0.5) as i32
}

/// The world pixel of a world point at height `z` (F1 with *L(z)*, claim
/// 3), with the default switches.
pub fn project_lifted(x: i32, y: i32, z: i32) -> (i32, i32) {
    project_lifted_with(x, y, z, &ProjectionQuirks::default())
}

/// F1 with *L(z)* under the switches `q`.
pub fn project_lifted_with(x: i32, y: i32, z: i32, q: &ProjectionQuirks) -> (i32, i32) {
    let (qx, qy) = project_with(x, y, q);
    (qx, qy - height_lift_with(z, q))
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
/// `size`.
pub fn limit_centre(
    requested: (i32, i32),
    size: (i32, i32),
    map_width: i32,
    local: [i32; 4],
    q: &ProjectionQuirks,
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
        let q = ProjectionQuirks::default();
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

    /// spec: S-0101 claims 3 and 4, Edge cases (quirks.height_lift_float_scale)
    #[test]
    fn height_lift_switch() {
        let on = ProjectionQuirks::default();
        let off = ProjectionQuirks {
            height_lift_float_scale: false,
            ..on
        };
        assert_eq!(height_lift_with(2080, &on), 299);
        assert_eq!(height_lift_with(2080, &off), 300);
        assert_eq!(height_lift_with(-104, &on), -14);
        assert_eq!(height_lift_with(-104, &off), -15);
        assert_eq!(height_lift_with(3500, &off), 505); // ⌊504.8 + ½⌋
        // Off: objects on whole levels stay aligned with the terrain at every level.
        for level in -128..=127 {
            assert_eq!(
                height_lift_with(104 * level, &off),
                15 * level,
                "level {level}"
            );
        }
        assert_eq!(project_lifted_with(0, 0, 2080, &off), (0, -300));
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
