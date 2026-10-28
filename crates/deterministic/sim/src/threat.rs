//! What the objects on the map give the path search of S-0212 beyond the
//! cell lists: the corridor exemption of buildings' exemption rectangles
//! and of aircraft (claim 13), kept as counts so that sources add up.
//!
//! Every hook here acts only on an object it tracks: one put on the map
//! through [`crate::econ::events::entered_map`]. An object that never
//! entered the map (a production item, a passenger created inside) has no
//! track, and its removal changes nothing.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{ObjectId, World};

/// What one object on the map has given (S-0212 claim 13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// A building's exemption rectangle, top-left and bottom-right cells
    /// inclusive, as given at its placement (`None`: a wall type, or not a
    /// building).
    pub rect: Option<(CellXY, CellXY)>,
    /// An aircraft's recorded cell (claims 13 and 16a): the cell it was
    /// placed on the ground in or last landed in; `None` for one placed in
    /// the air that has not landed yet.
    pub cell: Option<CellXY>,
}

/// The state of this module.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatWorld {
    /// How many buildings' exemption rectangles hold each cell (only cells
    /// with a count above 0).
    pub rects: BTreeMap<CellXY, u16>,
    /// How many aircraft give the exemption around each cell (only cells
    /// with a count above 0): the cell's eight neighbours are exempt.
    pub aircraft: BTreeMap<CellXY, u16>,
    /// The tracked objects.
    pub tracks: BTreeMap<ObjectId, Track>,
}

fn add(m: &mut BTreeMap<CellXY, u16>, c: CellXY) {
    let n = m.entry(c).or_insert(0);
    *n = n.saturating_add(1);
}

fn sub(m: &mut BTreeMap<CellXY, u16>, c: CellXY) {
    if let Some(n) = m.get_mut(&c) {
        *n = n.saturating_sub(1);
        if *n == 0 {
            m.remove(&c);
        }
    }
}

fn rect_cells((a, b): (CellXY, CellXY)) -> impl Iterator<Item = CellXY> {
    (a.y..=b.y).flat_map(move |y| (a.x..=b.x).map(move |x| CellXY::new(x, y)))
}

/// The exemption rectangle of building `id` (S-0212 Terms), or `None` for a
/// wall type.
fn exemption_rect(w: &World, id: ObjectId) -> Option<(CellXY, CellXY)> {
    let o = w.obj(id)?;
    let p = crate::econ::placement::place_type(w, o.type_index);
    // spec: S-0212 claim 13 (a wall type becomes its wall overlay with no building placed: it has no exemption rectangle)
    if p.wall {
        return None;
    }
    let tl = o.pos.cell();
    let (fw, fh) = p.size;
    // spec: S-0212 Terms (exemption rectangle: the foundation rectangle w × h grown by one cell on every side, from one lower in x and y than its top-left cell; the 8-cell 3 × 3 word's full rectangle; a zero-size foundation covers its own cell and the cells one lower in x, in y and in both)
    Some((
        CellXY::new(tl.x - 1, tl.y - 1),
        CellXY::new(tl.x + fw, tl.y + fh),
    ))
}

/// Whether an aircraft is placed on the ground, for its recorded cell
/// (S-0212 claim 16a).
fn placed_on_ground(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.ground)
}

/// An object entered the map ([`crate::econ::events::entered_map`]).
pub fn entered(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (kind, cell) = (o.kind, o.pos.cell());
    if !matches!(
        kind,
        Kind::Building | Kind::Vehicle | Kind::Infantry | Kind::Aircraft
    ) || w.threat.tracks.contains_key(&id)
    {
        return;
    }
    let mut t = Track::default();
    match kind {
        Kind::Building => {
            t.rect = exemption_rect(w, id);
            if let Some(r) = t.rect {
                // spec: S-0212 claim 13 (every building gives the exemption to its rectangle from the moment it is put on the map, whatever its owner, gates included; sources add up)
                for c in rect_cells(r) {
                    add(&mut w.threat.rects, c);
                }
            }
        }
        Kind::Aircraft => {
            // spec: S-0212 claim 16a (placed on the ground, an aircraft is recorded in that cell; placed in the air it has none) and quirks.aircraft_threat_stays_put (off: recorded in its placement cell)
            if placed_on_ground(w, id) || !w.rules().quirks.aircraft_threat_stays_put {
                t.cell = Some(cell);
            }
            // spec: S-0212 claim 13 (an aircraft gives the exemption around the cell where it is placed on the map)
            add(&mut w.threat.aircraft, cell);
        }
        _ => {}
    }
    w.threat.tracks.insert(id, t);
}

/// An object is about to leave the map
/// ([`crate::econ::events::leaving_map`]).
pub fn leaving(w: &mut World, id: ObjectId) {
    let Some(t) = w.threat.tracks.remove(&id) else {
        return;
    };
    if let Some(r) = t.rect {
        // spec: S-0212 claim 13 (when a building leaves the map the exemption is taken away from its rectangle, a to-tile type's included: the lead's fix of 2026-10-24, no switch)
        for c in rect_cells(r) {
            sub(&mut w.threat.rects, c);
        }
    }
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
        // spec: S-0212 claim 13 (leaving the map: the exemption is taken away around the recorded cell, or around the cell whose coordinates are both 0 for one that never had one; quirks.aircraft_threat_stays_put)
        sub(&mut w.threat.aircraft, t.cell.unwrap_or(CellXY::new(0, 0)));
    }
}

/// An aircraft landed in the cell it stands in (S-0501 claim 7).
pub fn landed(w: &mut World, id: ObjectId) {
    let Some(cell) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    let Some(t) = w.threat.tracks.get_mut(&id) else {
        return;
    };
    let old = t.cell.replace(cell);
    // spec: S-0212 claim 13 (at a landing the exemption moves from around the recorded cell to around the landing cell; an aircraft placed in the air has none at its first landing, so the exemption round its placement cell stays)
    if let Some(r) = old {
        sub(&mut w.threat.aircraft, r);
    }
    add(&mut w.threat.aircraft, cell);
}

/// Whether cell `c` lies in the exemption rectangle of a building on the
/// map (S-0212 claim 13).
pub fn in_rect(w: &World, c: CellXY) -> bool {
    w.threat.rects.contains_key(&c)
}

/// Whether an aircraft gives the exemption around cell `c` (S-0212 claim
/// 13): `c`'s neighbours are exempt.
pub fn aircraft_around(w: &World, c: CellXY) -> bool {
    w.threat.aircraft.contains_key(&c)
}
