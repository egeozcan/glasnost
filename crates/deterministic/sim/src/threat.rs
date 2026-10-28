//! What the objects on the map give the path search of S-0212 beyond the
//! cell lists: the corridor exemption of buildings' exemption rectangles
//! and of aircraft (claim 13), kept as counts so that sources add up, and
//! each house's threat records (claim 16a).
//!
//! Every hook here acts only on an object it tracks: one put on the map
//! through [`crate::econ::events::entered_map`]. An object that never
//! entered the map (a production item, a passenger created inside) has no
//! track, and its removal changes nothing.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// A threat block: (⌊x/4⌋, ⌊y/4⌋) of its cells (S-0212 Terms).
pub type Block = (i32, i32);

/// The threat block of cell `c` (S-0212 Terms).
pub fn block_of(c: CellXY) -> Block {
    // spec: S-0212 Terms and Constants (threat block of cell (x, y): (⌊x/4⌋, ⌊y/4⌋), 4 × 4 cells)
    (c.x.div_euclid(4), c.y.div_euclid(4))
}

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
    /// The threat amount last added for the object (claim 16a): one figure
    /// for every recording house, which a rebuild leaves as it is.
    pub amount: i32,
    /// The threat block it was last added at.
    pub block: Block,
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
    /// Each house's threat records, by house index and threat block (only
    /// blocks with a record above 0).
    pub records: Vec<BTreeMap<Block, i32>>,
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
    // spec: S-0212 claim 16a (a building's block holds its top-left foundation cell; a vehicle's or infantry unit's its recorded cell; an aircraft's amount is added at the block of its placement cell)
    t.block = block_of(cell);
    t.amount = amount(w, id);
    let owner = w.obj(id).and_then(|o| o.owner);
    // spec: S-0212 claim 16a (placed on the map: its current amount is added at its block)
    spread_into(w, &recorders(w, owner), t.block, t.amount, 1);
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
    let owner = w.obj(id).and_then(|o| o.owner);
    // spec: S-0212 claim 16a (leaving the map: the amount last added is taken away where it was added; an aircraft's by its own rule)
    let at = removal_block(w, id, &t);
    spread_into(w, &recorders(w, owner), at, t.amount, -1);
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
    // spec: S-0212 claim 16a (a landing changes no record) and quirks.aircraft_threat_stays_put (off: its amount moves to the landing cell's block as a vehicle's does)
    if !w.rules().quirks.aircraft_threat_stays_put {
        moved_to(w, id, block_of(cell));
    }
}

/// A vehicle or infantry unit was listed in cell `c` (S-0212 claims 13
/// and 16a: its recorded cell): when `c` lies in another threat block, its
/// amount moves there.
pub fn relisted(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    if !matches!(o.kind, Kind::Vehicle | Kind::Infantry) {
        return;
    }
    moved_to(w, id, block_of(c));
}

/// The amount of tracked object `id` moves to block `b`, if that is not
/// where it was last added.
fn moved_to(w: &mut World, id: ObjectId, b: Block) {
    let Some(t) = w.threat.tracks.get(&id).cloned() else {
        return;
    };
    // spec: S-0212 claim 16a (moving within one threat block changes nothing)
    if t.block == b {
        return;
    }
    let owner = w.obj(id).and_then(|o| o.owner);
    let houses = recorders(w, owner);
    // spec: S-0212 claim 16a (recorded in a cell of another block: the amount last added is taken away at the old block, and its current amount is added at the new)
    spread_into(w, &houses, t.block, t.amount, -1);
    let a = amount(w, id);
    spread_into(w, &houses, b, a, 1);
    if let Some(t) = w.threat.tracks.get_mut(&id) {
        t.block = b;
        t.amount = a;
    }
}

/// The owner of tracked object `id` changed from `old` (S-0212 claim 16a).
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(t) = w.threat.tracks.get(&id).cloned() else {
        return;
    };
    let new = w.obj(id).and_then(|o| o.owner);
    let at = removal_block(w, id, &t);
    // spec: S-0212 claim 16a (an owner change: the amount last added is taken away from the houses that record the old owner's objects, and the current amount added to those that record the new owner's; an aircraft's at the block of its recorded cell, or of the cell whose coordinates are both 0)
    spread_into(w, &recorders(w, old), at, t.amount, -1);
    let a = amount(w, id);
    spread_into(w, &recorders(w, new), at, a, 1);
    if let Some(t) = w.threat.tracks.get_mut(&id) {
        t.amount = a;
        t.block = at;
    }
}

/// A building's occupants changed: an infantry unit entered it as an
/// occupant, or occupants left it or were killed (S-0212 claim 16a).
pub fn refigure(w: &mut World, b: ObjectId) {
    let Some(t) = w.threat.tracks.get(&b).cloned() else {
        return;
    };
    let owner = w.obj(b).and_then(|o| o.owner);
    let houses = recorders(w, owner);
    // spec: S-0212 claim 16a (an occupant entering, leaving or killed: the building's amount last added is taken away and its current amount added, at its block)
    spread_into(w, &houses, t.block, t.amount, -1);
    let a = amount(w, b);
    spread_into(w, &houses, t.block, a, 1);
    if let Some(t) = w.threat.tracks.get_mut(&b) {
        t.amount = a;
    }
}

/// Whether house `h` counts as computer-controlled in a rebuild (S-0212
/// claim 16a).
fn computer_controlled(w: &World, h: HouseId) -> bool {
    w.houses.get(usize::from(h.0)).is_some_and(|house| {
        // spec: S-0212 claim 16a (an AI house counts as computer-controlled here, decided at the delegated sign-off of 2026-10-25)
        !house.human || house.econ.setup.ai
    })
}

/// Rebuilds house `h`'s threat records after it made or broke an alliance
/// (S-0212 claim 16a).
pub fn rebuild(w: &mut World, h: HouseId) {
    let q = w.rules().quirks;
    let computer = computer_controlled(w, h);
    let mut adds: Vec<(Block, i32)> = Vec::new();
    for (&id, t) in &w.threat.tracks {
        let Some(o) = w.obj(id) else { continue };
        // spec: S-0212 claim 16a (filled again from every object on the map that is not destroyed and whose current amount is above 0)
        if crate::combat::damage::dead(o) {
            continue;
        }
        let a = amount(w, id);
        if a <= 0 {
            continue;
        }
        let own = o.owner == Some(h);
        let allied = w.allied(Some(h), o.owner);
        let counts = if !q.threat_rebuild_counts_friends {
            // spec: S-0212 Edge cases (quirks.threat_rebuild_counts_friends off: the house's own objects and those of houses it allies are left out)
            !own && !allied
        } else if o.kind == Kind::Building {
            // spec: S-0212 claim 16a (every building counts, whoever owns it; quirks.threat_rebuild_counts_friends)
            true
        } else {
            // spec: S-0212 claim 16a (a unit not the house's own: when the house is human-controlled and does not ally its owner, and always when it is computer-controlled)
            !own && (computer || !allied)
        };
        if !counts {
            continue;
        }
        let at = if o.kind == Kind::Aircraft && q.aircraft_threat_stays_put {
            // spec: S-0212 claim 16a (in a rebuild an aircraft counts at the block of its recorded cell; one with none is skipped; quirks.aircraft_threat_stays_put)
            match t.cell {
                Some(c) => block_of(c),
                None => continue,
            }
        } else {
            t.block
        };
        adds.push((at, a));
    }
    if let Some(r) = records_mut(w, h) {
        // spec: S-0212 claim 16a (the records are set to 0, then filled again)
        r.clear();
    }
    for (at, a) in adds {
        spread_into(w, &[h], at, a, 1);
    }
}

/// The houses whose records hold the objects of `owner`: every house other
/// than it that does not ally it, by the recording house's own side
/// (S-0212 claim 16a).
fn recorders(w: &World, owner: Option<HouseId>) -> Vec<HouseId> {
    let Some(o) = owner else {
        return Vec::new();
    };
    (0..w.houses.len())
        .map(|i| HouseId(i as u8))
        // spec: S-0212 claim 16a (every house other than its owner that does not ally its owner; the recording house's own side of the relation decides)
        .filter(|&r| r != o && !w.allied(Some(r), Some(o)))
        .collect()
}

/// The block an object's amount is taken away at (S-0212 claim 16a): where
/// it was added, except for an aircraft with
/// `quirks.aircraft_threat_stays_put` on, at the block of its recorded
/// cell or of the cell whose coordinates are both 0.
fn removal_block(w: &World, id: ObjectId, t: &Track) -> Block {
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft)
        && w.rules().quirks.aircraft_threat_stays_put
    {
        // spec: S-0212 claim 16a (an aircraft's amount is taken away at the block of its recorded cell, or of the cell whose two coordinates are both 0 when it has never had one)
        return block_of(t.cell.unwrap_or(CellXY::new(0, 0)));
    }
    t.block
}

/// The current threat amount of object `id` (S-0212 claim 16a).
pub fn amount(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let field =
        |x: ObjectId| crate::combat::veterancy::type_combat(w, x).map_or(0, |c| c.threat_posed);
    let a = if o.kind == Kind::Building {
        let n = crate::structures::garrison::count(w, id);
        if n > 0 {
            // spec: S-0212 claim 16a (a building with at least one occupant: the number of occupants × the per-occupant threat)
            n.saturating_mul(w.rules().combat.general.record_threat_per_occupant)
        } else if let Some(v) = crate::structures::bunker::occupant(w, id) {
            // spec: S-0212 claim 16a (a building with no occupant that holds a bunkered vehicle: the threat field of that vehicle's type)
            field(v)
        } else {
            // spec: S-0212 claim 16a (any other building: the threat field of its own type)
            field(id)
        }
    } else {
        // spec: S-0212 claim 16a (a vehicle, infantry unit or aircraft: the threat field of its own type)
        field(id)
    };
    if w.rules().quirks.negative_threat_field {
        a
    } else {
        // spec: S-0212 Edge cases (quirks.negative_threat_field off: a negative threat field is read as 0)
        a.max(0)
    }
}

/// The number of threat blocks along each axis of the map's coordinate
/// square (S-0009 claim 2): every block holding one of its cells.
fn blocks_per_side(w: &World) -> i32 {
    let side = i32::from(w.cells.footprint().coordinate_side());
    // spec: S-0212 Terms (threat blocks of 4 × 4 cells: ⌈side / 4⌉ along each axis)
    (side + 3).div_euclid(4)
}

fn records_mut(w: &mut World, h: HouseId) -> Option<&mut BTreeMap<Block, i32>> {
    let n = w.houses.len();
    if w.threat.records.len() < n {
        w.threat.records.resize(n, BTreeMap::new());
    }
    w.threat.records.get_mut(usize::from(h.0))
}

/// Adds (`sign` 1) or takes away (`sign` −1) amount `a` spread over the
/// 3 × 3 blocks round block `b` in the records of `houses` (S-0212 claim
/// 16a), each record floored at 0. Records exist only for the map's
/// blocks: a spread reaching past the map's edge changes nothing there.
fn spread_into(w: &mut World, houses: &[HouseId], b: Block, a: i32, sign: i32) {
    if a == 0 || houses.is_empty() {
        return;
    }
    let side = blocks_per_side(w);
    for &h in houses {
        let Some(r) = records_mut(w, h) else { continue };
        for dy in -1..=1 {
            for dx in -1..=1 {
                let at = (b.0 + dx, b.1 + dy);
                if at.0 < 0 || at.1 < 0 || at.0 >= side || at.1 >= side {
                    continue;
                }
                // spec: S-0212 claim 16a (a in its block, ⌊a/2⌋ in the four sharing an edge, ⌊a/4⌋ in the four diagonal ones)
                let v = match dx.abs() + dy.abs() {
                    0 => a,
                    1 => a.div_euclid(2),
                    _ => a.div_euclid(4),
                };
                let e = r.entry(at).or_insert(0);
                // spec: S-0212 claim 16a (taking away subtracts the same three amounts; a record that would fall below 0 becomes 0)
                *e = e.saturating_add(v.saturating_mul(sign)).max(0);
                if *e == 0 {
                    r.remove(&at);
                }
            }
        }
    }
}

/// House `h`'s threat record for block `b` (S-0212 claim 16a).
pub fn record(w: &World, h: HouseId, b: Block) -> i32 {
    w.threat
        .records
        .get(usize::from(h.0))
        .and_then(|r| r.get(&b).copied())
        .unwrap_or(0)
}

/// The threat value of cell `c` for house `h` (S-0212 Terms).
pub fn value(w: &World, h: HouseId, c: CellXY) -> i32 {
    record(w, h, block_of(c))
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
