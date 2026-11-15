//! Which of the game's buildings cast their light (S-0405 claim 9, S-0104
//! claim 12), followed from one drawn frame to the next.
//!
//! A building's light does not follow its in-operation result: once the
//! building is on the map it changes only at claim 9's events. The view
//! sees those events as changes between the frames it draws: a switch-off
//! or switch-on, an ownership change, a building put on or taken off the
//! map. A pulse's start and end are not followed: no pulse exists in the
//! game yet (`glasnost_sim::econ::power::under_pulse`).

use std::collections::BTreeMap;

use glasnost_sim::world::{HouseId, ObjectId};

/// A building on the map, as one drawn frame sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    /// The building.
    pub id: ObjectId,
    /// Switched off by the power switch (S-0405 Terms).
    pub switched_off: bool,
    /// Its owner.
    pub owner: Option<HouseId>,
    /// Its type has the needs-engineer flag.
    pub needs_engineer: bool,
    /// Manned: it had an ownership change (S-0405 Terms).
    pub manned: bool,
    /// The map's building list placed it.
    pub map_placed: bool,
}

/// What a building's light was last seen with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seen {
    lit: bool,
    switched_off: bool,
    owner: Option<HouseId>,
}

/// The light of `b`, last seen as `prev` (`None`: not on the map then).
fn next(prev: Option<Seen>, b: &Building) -> Seen {
    let lit = match prev {
        // spec: S-0405 claim 9 (a building put on the map casts its light, except one of a needs-engineer
        // type the map's building list places unmanned)
        // design: render (#152): one first seen switched off (a game continued from a save) starts out
        None => !(b.map_placed && b.needs_engineer && !b.manned) && !b.switched_off,
        Some(p) => {
            let mut lit = p.lit;
            // spec: S-0405 claim 9 (out at a switch-off, on at a switch-on)
            if b.switched_off != p.switched_off {
                lit = !b.switched_off;
            }
            // spec: S-0405 claim 9 (on at every ownership change of a needs-engineer type, switched off or not)
            // design: render (#152): seen in the same frame as a switch, the ownership change counts last
            if b.needs_engineer && b.owner != p.owner {
                lit = true;
            }
            lit
        }
    };
    Seen {
        lit,
        switched_off: b.switched_off,
        owner: b.owner,
    }
}

/// The buildings' lights, as last seen.
#[derive(Clone, Debug, Default)]
pub struct Tracker {
    seen: BTreeMap<ObjectId, Seen>,
}

impl Tracker {
    /// Follows the lights to `on_map`, every building on the map now;
    /// returns the lit ones, in the order given.
    pub fn update(&mut self, on_map: &[Building]) -> Vec<ObjectId> {
        let mut seen = BTreeMap::new();
        let mut lit = Vec::new();
        for b in on_map {
            let s = next(self.seen.get(&b.id).copied(), b);
            if s.lit {
                lit.push(b.id);
            }
            seen.insert(b.id, s);
        }
        // spec: S-0405 claim 9 (the light goes out when the building leaves the map in any way)
        self.seen = seen;
        lit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn building(id: u32) -> Building {
        Building {
            id: ObjectId(id),
            switched_off: false,
            owner: Some(HouseId(0)),
            needs_engineer: false,
            manned: false,
            map_placed: false,
        }
    }

    /// spec: S-0405 claim 9 (a needs-engineer building the map's building list places unmanned
    /// starts without its light and has it from its ownership change on, manned or not
    /// afterwards; an ordinary map building starts lit)
    #[test]
    fn claim_9_an_unmanned_map_building_starts_dark() {
        let mut t = Tracker::default();
        let tech = Building {
            needs_engineer: true,
            map_placed: true,
            ..building(1)
        };
        let plain = Building {
            map_placed: true,
            ..building(2)
        };
        assert_eq!(t.update(&[tech, plain]), [ObjectId(2)]);
        let taken = Building {
            owner: Some(HouseId(1)),
            manned: true,
            ..tech
        };
        assert_eq!(t.update(&[taken, plain]), [ObjectId(1), ObjectId(2)]);
        assert_eq!(t.update(&[taken, plain]), [ObjectId(1), ObjectId(2)]);
    }

    /// spec: S-0405 claim 9 (an ownership change of a type without the needs-engineer flag changes
    /// no light; a switched-off building stays dark through it)
    #[test]
    fn claim_9_an_ordinary_capture_changes_no_light() {
        let mut t = Tracker::default();
        let b = building(1);
        assert_eq!(t.update(&[b]), [ObjectId(1)]);
        let off = Building {
            switched_off: true,
            ..b
        };
        assert!(t.update(&[off]).is_empty());
        let taken = Building {
            owner: Some(HouseId(2)),
            manned: true,
            ..off
        };
        assert!(t.update(&[taken]).is_empty());
    }

    /// spec: S-0405 claim 9 (a building that left the map has no light; one put on the map again
    /// starts as a new one)
    #[test]
    fn claim_9_a_building_off_the_map_has_no_light() {
        let mut t = Tracker::default();
        assert_eq!(t.update(&[building(1)]), [ObjectId(1)]);
        assert!(t.update(&[]).is_empty());
        assert_eq!(t.update(&[building(1)]), [ObjectId(1)]);
    }
}
