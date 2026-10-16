//! Terrain surface height (S-0106 claims 1 to 3, F1).

use glasnost_map::CellXY;

use crate::world::World;

/// (a, b, c, d) of S-0106 table C2 for shapes 1 … 20.
// spec: S-0106 Constants table C2 (rise of each shape)
const RISE: [(i32, i32, i32, i32); 20] = [
    (1, 0, 0, 104),
    (0, 1, 0, 104),
    (-1, 0, 104, 104),
    (0, -1, 104, 104),
    (1, 1, -104, 104),
    (-1, 1, 0, 104),
    (-1, -1, 104, 104),
    (1, -1, 0, 104),
    (1, 1, 0, 104),
    (-1, 1, 104, 104),
    (-1, -1, 208, 104),
    (1, -1, 104, 104),
    (1, 1, 0, 208),
    (-1, 1, 104, 208),
    (-1, -1, 208, 208),
    (1, -1, 104, 208),
    (0, 0, 52, 52),
    (0, 0, 52, 52),
    (0, 0, 52, 52),
    (0, 0, 52, 52),
];

/// The surface height H of a cell at level `level` with slope shape `s` at
/// in-cell position (u, v) (S-0106 F1). `raised` is
/// `quirks.negative_level_floor_raised`.
pub fn surface_height(level: i32, s: u8, u: i32, v: i32, raised: bool) -> i32 {
    // spec: S-0106 F1 (B(ℓ) = T(104ℓ + 0.5): 104ℓ for ℓ ≥ 0, 104ℓ + 1 below 0)
    let base = 104 * level + i32::from(level < 0 && raised);
    // spec: S-0106 F1 (h = min(max(13(a u + b v)/32 + c, 0), d); H = T(B + h)), in 32nds
    let h32 = match s {
        1..=20 => {
            let (a, b, c, d) = RISE[usize::from(s) - 1];
            (13 * (a * u + b * v) + 32 * c).clamp(0, 32 * d)
        }
        _ => 0,
    };
    (32 * base + h32) / 32 // spec: S-0106 F1 (h in 32nds)
}

impl World {
    /// The surface height at a position on the ground (S-0106 claim 2).
    pub fn surface_at(&self, x: i32, y: i32) -> i32 {
        let c = CellXY::new(x / 256, y / 256); // spec: S-0204 claim 2 (the containing cell)
        let raised = self.rules().quirks.negative_level_floor_raised;
        match self.cells.cell(c) {
            Some(cell) => surface_height(
                i32::from(cell.level),
                cell.slope,
                x.rem_euclid(256), // spec: S-0204 claim 2 (the position within the cell)
                y.rem_euclid(256), // spec: S-0204 claim 2 (the position within the cell)
                raised,
            ),
            None => 0,
        }
    }

    /// Whether a cell carries a high bridge with a deck (S-0211 claim 15).
    pub fn high_bridge(&self, c: CellXY) -> bool {
        self.cells.idx(c).is_some_and(|i| self.cells.deck[i])
    }

    /// The height of the surface an object is on or over (glossary *height
    /// above the surface*): the bridge deck, 416 leptons above the terrain,
    /// for an object on a deck; otherwise the terrain at its position.
    pub fn surface_for(&self, id: crate::world::ObjectId) -> i32 {
        let Some(o) = self.obj(id) else { return 0 };
        let t = self.surface_at(o.pos.x, o.pos.y);
        // spec: S-0501 claim 43 (an aircraft with its own deck mark is measured from the deck, S-0105 claim 40's b)
        let marked = o.air.as_ref().is_some_and(|a| a.deck);
        if o.layer == crate::world::Layer::Deck || marked {
            t + 416 // spec: S-0307 Constants (bridge deck height 416 leptons above the terrain)
        } else {
            t
        }
    }

    /// An object's height above the surface (glossary).
    pub fn height_above_surface(&self, id: crate::world::ObjectId) -> i32 {
        self.obj(id).map_or(0, |o| o.pos.z - self.surface_for(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0106 claim 2 (shape 1 rises from 0 at u = 0 to 103 at u = 255; centre heights)
    #[test]
    fn rises() {
        assert_eq!(surface_height(0, 1, 0, 128, true), 0);
        assert_eq!(surface_height(0, 1, 255, 128, true), 103);
        // Centre heights: edge slopes +52, corners +0, valleys and steep +104, saddles +52.
        assert_eq!(surface_height(2, 1, 128, 128, true), 208 + 52);
        assert_eq!(surface_height(2, 5, 128, 128, true), 208);
        assert_eq!(surface_height(2, 9, 128, 128, true), 208 + 104);
        assert_eq!(surface_height(2, 13, 128, 128, true), 208 + 104);
        assert_eq!(surface_height(2, 17, 10, 200, true), 208 + 52);
        assert_eq!(surface_height(2, 0, 10, 200, true), 208);
    }

    // spec: S-0106 claim 3 (level −1 flat at −103, −2 at −207; switch off: 104ℓ)
    #[test]
    fn negative_levels() {
        assert_eq!(surface_height(-1, 0, 0, 0, true), -103);
        assert_eq!(surface_height(-2, 0, 0, 0, true), -207);
        assert_eq!(surface_height(-1, 0, 0, 0, false), -104);
    }
}
