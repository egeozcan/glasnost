//! Entering a cell by class, the passability category and the zone table
//! (S-0211 claims 10 to 12, 14).

use glasnost_map::terrain::land;
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::{F32, F64};
use glasnost_rules::sim::{LocomotionClass, MovementZone};

use crate::rules::{Kind, SimRules};
use crate::world::World;

/// Passability categories (S-0211 Terms).
pub mod category {
    /// Open.
    pub const OPEN: u8 = 0; // spec: S-0211 Terms (category 0 open)
    /// Crushable.
    pub const CRUSHABLE: u8 = 1; // spec: S-0211 Terms (category 1 crushable)
    /// Wall-like.
    pub const WALL_LIKE: u8 = 2; // spec: S-0211 Terms (category 2 wall-like)
    /// Beach.
    pub const BEACH: u8 = 3; // spec: S-0211 Terms (category 3 beach)
    /// Water.
    pub const WATER: u8 = 4; // spec: S-0211 Terms (category 4 water)
    /// Partial obstacle.
    pub const PARTIAL: u8 = 5; // spec: S-0211 Terms (category 5 partial obstacle)
    /// Blocked.
    pub const BLOCKED: u8 = 6; // spec: S-0211 Terms (category 6 blocked)
    /// Off-map.
    pub const OFF_MAP: u8 = 7; // spec: S-0211 Terms (category 7 off-map)
}

/// A zone-table answer (S-0211 claim 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The zone may cross the category.
    Yes,
    /// It may not.
    No,
    /// The stronger no of the off-map column.
    Never,
}

/// The zone table of S-0211 claim 12.
pub fn zone_table(zone: MovementZone, cat: u8) -> Answer {
    use MovementZone as Z;
    if cat >= category::OFF_MAP {
        // spec: S-0211 claim 12 (column 7: never for every zone)
        return Answer::Never;
    }
    // spec: S-0211 claim 12 (the zone table, columns 0 open … 6 blocked)
    let row: [bool; 7] = match zone {
        Z::Normal => [true, false, false, false, false, false, false],
        Z::Crusher => [true, true, false, false, false, false, false],
        Z::Destroyer => [true, true, true, false, false, false, false],
        Z::AmphibiousDestroyer => [true, true, true, true, true, true, false],
        Z::AmphibiousCrusher => [true, true, false, true, true, false, false],
        Z::Amphibious => [true, false, false, true, true, false, false],
        Z::Subterranean => [true, true, true, false, false, false, true],
        Z::Infantry => [true, false, false, false, false, true, false],
        Z::InfantryDestroyer => [true, true, true, false, false, true, false],
        Z::Fly => [true, true, true, true, true, true, true],
        Z::Water => [false, false, false, false, true, false, false],
        Z::WaterBeach => [false, false, false, true, true, false, false],
        Z::CrusherAll => [true, true, true, false, false, false, false],
    };
    if row[usize::from(cat)] {
        Answer::Yes
    } else {
        Answer::No
    }
}

fn nonzero(m: F32) -> bool {
    // spec: S-0211 Formulas (zero tests are exact on the single-precision value)
    !m.to_f64().eq_num(F64::ZERO)
}

/// The passability category of a cell (S-0211 claim 11), from its land type,
/// overlay and the deciding objects listed in its ground layer.
pub fn compute_category(w: &World, c: CellXY) -> u8 {
    let r = w.rules();
    let Some(cell) = w.cells.cell(c) else {
        return category::OFF_MAP;
    };
    // spec: S-0211 claim 11 (rule 1: outside the usable area)
    if !w.cells.usable(c) {
        return category::OFF_MAP;
    }
    let wheel = LocomotionClass::Wheel;
    // spec: S-0211 claim 11 (rule 2: the overlay's flags and land type)
    if cell.overlay != NO_OVERLAY
        && let Some(o) = r.overlays.get(usize::from(cell.overlay))
    {
        if o.crushable {
            return category::CRUSHABLE;
        }
        if o.wall {
            return category::WALL_LIKE;
        }
        if !nonzero(r.multiplier(o.land, wheel)) {
            return category::BLOCKED;
        }
        if o.rock {
            return category::BLOCKED;
        }
        if o.rubble {
            return category::OPEN;
        }
    }
    // spec: S-0211 claim 11 (rule 3: water, beach, M(ℓ, wheel) ≤ 0.01)
    if cell.land == land::WATER {
        return category::WATER;
    }
    if cell.land == land::BEACH {
        return category::BEACH;
    }
    let blocked_by_land = if r.quirks.passability_wheel_column {
        low(r.multiplier(cell.land, wheel))
    } else {
        // spec: S-0211 claim 11 (`quirks.passability_wheel_column` off: every ground class)
        GROUND_CLASSES
            .iter()
            .all(|&c| low(r.multiplier(cell.land, c)))
    };
    if blocked_by_land {
        return category::BLOCKED;
    }
    // spec: S-0211 claim 11 (rule 4: deciding objects; the first listed decides)
    for id in w.cells.list(c, crate::world::Layer::Ground) {
        if let Some(o) = w.obj(*id)
            && o.kind == Kind::Terrain
        {
            let t = &r.terrain[o.type_index as usize];
            let pattern = if w.theater_snow() {
                t.occupation.1
            } else {
                t.occupation.0
            };
            // spec: S-0211 claim 11 (pattern 7 gives 2, any other pattern 5)
            return if pattern == 7 {
                category::WALL_LIKE
            } else {
                category::PARTIAL
            };
        }
    }
    // spec: S-0211 claim 11 (rule 5)
    category::OPEN
}

const GROUND_CLASSES: [LocomotionClass; 7] = [ // spec: S-0211 claim 2 (the seven classes that move on the ground)
    LocomotionClass::Foot,
    LocomotionClass::Track,
    LocomotionClass::Wheel,
    LocomotionClass::Hover,
    LocomotionClass::Float,
    LocomotionClass::Amphibious,
    LocomotionClass::FloatBeach,
];

/// M ≤ the double nearest to 0.01 (S-0211 Formulas, rule 3 threshold).
fn low(m: F32) -> bool {
    // spec: S-0211 Formulas (category rule 3 threshold: the double nearest 0.01, compared exactly)
    m.to_f64().le(crate::rules::nearest_double("0.01"))
}

/// Whether an object of locomotion class `class` and movement zone `zone`
/// may enter cell `c` by its land and overlay (S-0211 claim 10). `deck`: it
/// will stand on the cell's bridge deck.
pub fn enterable_by_class(
    r: &SimRules,
    w: &World,
    c: CellXY,
    class: Option<LocomotionClass>,
    zone: MovementZone,
    deck: bool,
) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    // spec: S-0211 claim 3 (departure: a type with no class reads as if the key were absent; no class enters nothing)
    let Some(class) = class else {
        return false;
    };
    // spec: S-0211 claim 10 (winged: always; on a bridge deck: always)
    if class == LocomotionClass::Winged || deck {
        return true;
    }
    if cell.overlay != NO_OVERLAY
        && let Some(o) = r.overlays.get(usize::from(cell.overlay))
        && o.wall
    {
        use MovementZone as Z;
        // spec: S-0211 claim 10 (a wall cell: destroyer zones and crusher-all, crushers for crushable walls; as clear)
        let may = matches!(
            zone,
            Z::Destroyer | Z::AmphibiousDestroyer | Z::InfantryDestroyer | Z::CrusherAll
        ) || (matches!(zone, Z::Crusher | Z::AmphibiousCrusher) && o.crushable);
        return may && nonzero(r.multiplier(land::CLEAR, class));
    }
    // spec: S-0211 claim 10 (otherwise: M(ℓ, c) ≠ 0)
    nonzero(r.multiplier(cell.land, class))
}

/// The restricted-land rule of S-0211 claim 14.
pub fn restricted_allows(w: &World, c: CellXY, restricted: Option<u8>, on_deck: bool) -> bool {
    let Some(want) = restricted else {
        return true;
    };
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    // spec: S-0211 claim 14 (the restricted land type itself)
    if cell.land == want {
        return true;
    }
    if cell.land == land::TUNNEL {
        // spec: S-0211 claim 14 (tunnel cells; on 3×4/3×5 tiles only sub-tile 6, on 4×3/5×3 only sub-tile 2)
        let size = w.tile_size(cell.tile);
        return match size {
            Some((3, 4) | (3, 5)) => cell.sub_tile == 6, // spec: S-0211 claim 14 (3×4 and 3×5 tunnel tiles: sub-tile 6)
            Some((4, 3) | (5, 3)) => cell.sub_tile == 2, // spec: S-0211 claim 14 (4×3 and 5×3 tunnel tiles: sub-tile 2)
            _ => true,
        };
    }
    // spec: S-0211 claim 14 (overlay registration indices 237 and 238 under a deck; quirk off: any high-bridge cell)
    on_deck
        && ((w.rules().quirks.restricted_deck_overlay_indices
            && (cell.overlay == 237 || cell.overlay == 238))
            || (!w.rules().quirks.restricted_deck_overlay_indices
                && w.cells.idx(c).is_some_and(|i| w.cells.deck[i])))
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0211 claim 12 (spot checks of the zone table)
    #[test]
    fn zone_table_rows() {
        use MovementZone as Z;
        assert_eq!(zone_table(Z::Normal, category::OPEN), Answer::Yes);
        assert_eq!(zone_table(Z::Normal, category::CRUSHABLE), Answer::No);
        assert_eq!(zone_table(Z::Crusher, category::CRUSHABLE), Answer::Yes);
        assert_eq!(zone_table(Z::Infantry, category::PARTIAL), Answer::Yes);
        assert_eq!(zone_table(Z::Water, category::OPEN), Answer::No);
        assert_eq!(zone_table(Z::WaterBeach, category::BEACH), Answer::Yes);
        assert_eq!(zone_table(Z::Subterranean, category::BLOCKED), Answer::Yes);
        assert_eq!(zone_table(Z::Fly, category::OFF_MAP), Answer::Never);
        assert_eq!(zone_table(Z::AmphibiousDestroyer, category::BLOCKED), Answer::No);
    }
}
