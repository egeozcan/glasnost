//! Map footprint, legal and usable cells, visiting order and cell numbers
//! (S-0009 claims 1 to 4, 15 and 16).

use serde::{Deserialize, Serialize};

/// Highest supported cell coordinate (S-0009 claim 2: coordinates 0 … 511;
/// our engine treats any cell beyond as non-legal, a departure).
pub const MAX_COORDINATE: i32 = 511; // spec: S-0009 Constants (supported coordinate range per axis)

/// Cell-number base: n = x + 1000·y (S-0009 Formulas).
pub const CELL_NUMBER_BASE: i32 = 1000; // spec: S-0009 Constants (cell number base)

/// Legacy cell-tag base below format version 4 (S-0009 Formulas).
pub const LEGACY_CELL_TAG_BASE: i32 = 128; // spec: S-0009 Constants (legacy cell-tag base)

/// Format version from which cell tags decode 1000-based (S-0009 claim 15).
pub const CELL_TAG_BASE_SWITCH_VERSION: i32 = 4; // spec: S-0009 Constants (format version that switches cell-tag decoding)

/// A cell position (S-0009 Terms: x to the lower right, y to the lower left).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellXY {
    /// x.
    pub x: i32,
    /// y.
    pub y: i32,
}

impl CellXY {
    /// The cell (x, y).
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Screen row s = x + y (S-0009 Terms), in 64 bits so that no input
    /// overflows.
    pub fn s(self) -> i64 {
        i64::from(self.x) + i64::from(self.y)
    }

    /// Screen column d = x − y (S-0009 Terms).
    pub fn d(self) -> i64 {
        i64::from(self.x) - i64::from(self.y)
    }
}

/// Decodes a cell number with base `base`: x = n rem base, y = n quo base,
/// both truncating toward zero (S-0009 Formulas).
pub fn decode_cell_number(n: i32, base: i32) -> (i32, i32) {
    // spec: S-0009 Formulas (quo truncates toward zero, rem takes the sign of n)
    (n % base, n / base)
}

/// Reduces a decoded waypoint coordinate modulo 65536 into −32768 … 32767
/// (S-0009 claim 15).
pub fn wrap_coordinate(v: i32) -> i16 {
    // spec: S-0009 claim 15 (each decoded coordinate is reduced modulo 65536 into −32768 … 32767)
    v as i16
}

/// The map size W × H and the local area (S-0009 claims 1 and 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    /// Map width W (cell widths).
    pub width: i32,
    /// Map height H (cell heights).
    pub height: i32,
    /// Local area (Lx, Ly, Lw, Lh).
    pub local: [i32; 4], // spec: S-0009 Terms (local area: four numbers)
}

impl Footprint {
    /// A footprint of W × H with the local area `local`. Negative sizes
    /// count as 0 (no legal cells).
    // spec: S-0009 Terms (local area: four numbers)
    pub fn new(width: i32, height: i32, local: [i32; 4]) -> Self {
        Self {
            width: width.max(0),
            height: height.max(0),
            local,
        }
    }

    /// Whether (x, y) is a legal cell (S-0009 claims 1 and 2):
    /// W < s ≤ W + 2H and −W < d < W, and both coordinates within 0 … 511.
    pub fn is_legal(&self, c: CellXY) -> bool {
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (s, d) = (c.s(), c.d());
        // spec: S-0009 claim 1 (W < s ≤ W + 2H and −W < d < W)
        let inside = w < s && s <= w + 2 * h && -w < d && d < w;
        // spec: S-0009 claim 2 (our engine: coordinates above 511 are not legal)
        inside && (0..=MAX_COORDINATE).contains(&c.x) && (0..=MAX_COORDINATE).contains(&c.y)
    }

    /// Number of legal cells, H·(2W − 1) for W ≥ 1 (S-0009 claim 1),
    /// ignoring the coordinate limit of claim 2.
    pub fn legal_count(&self) -> i64 {
        if self.width < 1 {
            return 0;
        }
        // spec: S-0009 claim 1 (H·(2W − 1) legal cells)
        i64::from(self.height) * (2 * i64::from(self.width) - 1)
    }

    /// Side of the coordinate square 0 … W + H − 1 that holds every legal
    /// cell, capped at the supported range (S-0009 claim 2).
    pub fn coordinate_side(&self) -> u16 {
        let side = (i64::from(self.width) + i64::from(self.height))
            .clamp(0, i64::from(MAX_COORDINATE) + 1);
        u16::try_from(side).unwrap_or(0)
    }

    /// Every legal cell in the visiting order of S-0009 claim 3: screen
    /// rows from s = W + 1 down, each row in increasing x. With W = 1 and
    /// `narrow_quirk` on (`quirks.narrow_map_partial_pass`) only (1, 1) is
    /// visited; off, every legal cell is.
    pub fn visiting_order(&self, narrow_quirk: bool) -> Vec<CellXY> {
        let (w, h) = (self.width, self.height);
        let mut out = Vec::new();
        if w < 1 || h < 1 {
            return out;
        }
        // spec: S-0009 claim 3 (W = 1: the pass visits only (1, 1), quirks.narrow_map_partial_pass)
        if w == 1 && narrow_quirk {
            let c = CellXY::new(1, 1);
            if self.is_legal(c) {
                out.push(c);
            }
            return out;
        }
        let (w, h) = (i64::from(w), i64::from(h));
        let max = i64::from(MAX_COORDINATE);
        // spec: S-0009 claim 3 (rows s = W + 1 … W + 2H top to bottom, increasing x within a row);
        // claim 2 (our engine: no cell beyond coordinate 511 exists, so no row beyond 1022 holds one)
        for s in (w + 1)..=(w + 2 * h).min(2 * max) {
            for x in (s - max).max(0)..=s.min(max) {
                let c = CellXY::new(x as i32, (s - x) as i32);
                if self.is_legal(c) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// The usable-cell test of S-0009 claim 4 with height term `h` (0 for
    /// the plain test).
    pub fn is_usable_with(&self, c: CellXY, h: i32) -> bool {
        let w = i64::from(self.width);
        let [lx, ly, lw, lh] = self.local.map(i64::from);
        let h = i64::from(h);
        let (s, d) = (c.s(), c.d());
        // spec: S-0009 claim 4, Formulas (usable cell inequalities)
        self.is_legal(c)
            && w + 2 * ly + h < s
            && s <= w + 2 + 2 * (ly + lh) + h
            && 2 * lx - w < d
            && d < 2 * (lx + lw) - w
    }

    /// The height-aware usable test of S-0009 claim 4: h = level, plus one
    /// for a slope when s < W + 4 + 2·Ly + level.
    pub fn is_usable(&self, c: CellXY, level: i32, slope: bool) -> bool {
        let w = i64::from(self.width);
        let ly = i64::from(self.local[1]);
        // spec: S-0009 Formulas (δ = 1 if the cell is a slope and s < W + 4 + 2·Ly + ℓ)
        let delta = i32::from(slope && c.s() < w + 4 + 2 * ly + i64::from(level));
        self.is_usable_with(c, level + delta)
    }

    /// The home waypoint's default cell (c, c), c = ⌊(W + H) / 2⌋ (S-0009
    /// claim 16).
    pub fn home_default(&self) -> CellXY {
        // spec: S-0009 Formulas (home waypoint default cell)
        let c = (i64::from(self.width) + i64::from(self.height)).max(0) / 2;
        let c = i32::try_from(c).unwrap_or(i32::MAX);
        CellXY::new(c, c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0009 claim 1 (scenario M1's map: 10 × 6, 114 legal cells)
    #[test]
    fn legal_cells_of_m1() {
        let f = Footprint::new(10, 6, [0, 0, 10, 6]);
        assert!(f.is_legal(CellXY::new(1, 10)));
        assert!(!f.is_legal(CellXY::new(1, 9)));
        assert_eq!(f.legal_count(), 114);
        assert_eq!(f.visiting_order(true).len(), 114);
        // spec: S-0009 claim 3 (the first cell visited is (1, W))
        assert_eq!(f.visiting_order(true)[0], CellXY::new(1, 10));
    }

    /// spec: S-0009 claim 3 (W = 1 edge case)
    #[test]
    fn narrow_map() {
        let f = Footprint::new(1, 5, [0, 0, 1, 5]);
        assert_eq!(f.visiting_order(true), vec![CellXY::new(1, 1)]);
        assert_eq!(f.visiting_order(false).len(), 5);
    }

    /// spec: S-0009 claim 2 (coordinates above 511 are not legal in our engine)
    #[test]
    fn coordinate_limit() {
        let f = Footprint::new(400, 400, [0, 0, 400, 400]);
        assert!(!f.is_legal(CellXY::new(512, 400)));
        assert!(f.is_legal(CellXY::new(400, 400)));
        assert_eq!(f.coordinate_side(), 512);
    }

    /// spec: S-0009 claim 4 (height moves the top edge down one half-row per level)
    #[test]
    fn usable_cells() {
        let f = Footprint::new(30, 30, [2, 4, 26, 22]);
        // Top edge: s > W + 2·Ly = 38.
        assert!(!f.is_usable(CellXY::new(20, 18), 0, false));
        assert!(f.is_usable(CellXY::new(20, 19), 0, false));
        assert!(!f.is_usable(CellXY::new(20, 19), 4, false));
        assert!(f.is_usable(CellXY::new(22, 21), 4, false));
    }

    /// spec: S-0009 Formulas (cell number decoding, truncation toward zero)
    #[test]
    fn cell_numbers() {
        assert_eq!(decode_cell_number(10015, CELL_NUMBER_BASE), (15, 10));
        assert_eq!(decode_cell_number(-1005, CELL_NUMBER_BASE), (-5, -1));
        assert_eq!(decode_cell_number(1290, LEGACY_CELL_TAG_BASE), (10, 10));
        assert_eq!(wrap_coordinate(32768), -32768);
    }

    /// spec: S-0009 claim 16 (home default (c, c))
    #[test]
    fn home_default() {
        assert_eq!(
            Footprint::new(50, 51, [0; 4]).home_default(),
            CellXY::new(50, 50)
        );
    }
}
