//! Tracked flying objects and the flying-object search (S-0303 Terms,
//! claims 6 and 11, Formulas F7).
//!
//! A tracked flying object is registered in the sector of the cell it is
//! over and moves between sectors as it flies. Rocket missiles register at
//! the end of their tilt or raise and leave before their own detonation
//! (S-0506 claims 15 and 19). Aircraft in ordinary flight and jumpjets
//! belong to the movement lanes: they call [`register`], [`moved`] and
//! [`unregister`] at the boundaries their specs give.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// A sector (i, j), 0 ≤ i, j ≤ 19 (S-0303 F7).
pub type Sector = (i32, i32);

/// The tracked flying objects (S-0303 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flying {
    /// Every tracked object, in the order its current tracking period
    /// started (S-0303 claim 11, the order with `quirks.flying_search_sectors` off).
    pub order: Vec<ObjectId>,
    /// The objects registered in each sector, in registration order there.
    pub sectors: BTreeMap<Sector, Vec<ObjectId>>,
    /// Each tracked object's sector.
    pub sector_of: BTreeMap<ObjectId, Sector>,
}

/// The sector size w = h = trunc(B / 20), B = W + H − 1 (S-0303 F7); 0
/// on a small map (S-0303 Edge cases: the zero-divisor rule of S-0205
/// claim 3 gives quotient 0, so every cell lies in sector (0, 0)).
fn sector_size(w: &World) -> i32 {
    let fp = w.cells.footprint();
    let b = fp.width + fp.height - 1;
    b / 20 // spec: S-0303 F7 (w = h = trunc(B / 20))
}

/// σ of one coordinate (S-0303 F7): min(trunc(max(x, 0) / w), 19).
fn sigma(x: i32, size: i32) -> i32 {
    if size == 0 {
        // spec: S-0303 Edge cases (a small map: the quotient of a zero divisor is 0, S-0205 claim 3)
        return 0;
    }
    (x.max(0) / size).min(19) // spec: S-0303 F7 (σ = min(trunc(max(x, 0) / w), 19))
}

/// The sector of a cell (S-0303 F7).
pub fn sector(w: &World, c: CellXY) -> Sector {
    let s = sector_size(w);
    (sigma(c.x, s), sigma(c.y, s))
}

/// Whether an object is a tracked flying object.
pub fn tracked(w: &World, id: ObjectId) -> bool {
    w.flying.sector_of.contains_key(&id)
}

/// Registers an object as a tracked flying object in the sector of the
/// cell it is over (S-0303 Terms). Registering a tracked object again
/// does nothing.
pub fn register(w: &mut World, id: ObjectId) {
    if tracked(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let s = sector(w, o.pos.cell());
    // spec: S-0303 claim 11 (a newly tracked object follows every object already registered, in its sector and globally)
    w.flying.order.push(id);
    w.flying.sectors.entry(s).or_default().push(id);
    w.flying.sector_of.insert(id, s);
}

/// Takes an object out of the tracked flying objects.
pub fn unregister(w: &mut World, id: ObjectId) {
    let Some(s) = w.flying.sector_of.remove(&id) else {
        return;
    };
    w.flying.order.retain(|&x| x != id);
    if let Some(list) = w.flying.sectors.get_mut(&s) {
        list.retain(|&x| x != id);
        if list.is_empty() {
            w.flying.sectors.remove(&s);
        }
    }
}

/// Updates a tracked object's sector after it moved (S-0303 Terms: the
/// game updates the sector as it flies). An object that crosses into
/// another sector follows every object already there; the others keep
/// their order, and the global order of tracking starts is unchanged
/// (S-0303 claim 11).
pub fn moved(w: &mut World, id: ObjectId) {
    let Some(&old) = w.flying.sector_of.get(&id) else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let s = sector(w, o.pos.cell());
    if s == old {
        return;
    }
    if let Some(list) = w.flying.sectors.get_mut(&old) {
        list.retain(|&x| x != id);
        if list.is_empty() {
            w.flying.sectors.remove(&old);
        }
    }
    // spec: S-0303 claim 11 (a crossing object follows every object already registered in its destination sector)
    w.flying.sectors.entry(s).or_default().push(id);
    w.flying.sector_of.insert(id, s);
}

/// The ordered sector set of the flying-object search around centre cell
/// `c` with search radius `rho` cells (S-0303 F7).
pub fn sector_set(w: &World, c: CellXY, rho: i32) -> Vec<Sector> {
    let size = sector_size(w);
    let sx = |x: i32| sigma(x, size);
    let sy = |y: i32| sigma(y, size);
    let (cx, cy) = (sx(c.x), sy(c.y));
    // spec: S-0303 F7 ("the sector (i + a, j + b)": each coordinate clamped to 0 … 19)
    let clamp = |i: i32, j: i32| (i.clamp(0, 19), j.clamp(0, 19));
    let mut out = vec![(cx, cy)];
    // spec: S-0303 F7 (the four runs: east, north, south, west, each from its far sector inwards)
    let e = sx(c.x + rho) - cx;
    let n = cy - sy(c.y - rho);
    let s = sy(c.y + rho) - cy;
    let u = cx - sx(c.x - rho);
    for k in (1..=e).rev() {
        out.push((cx + k, cy));
    }
    for k in (1..=n).rev() {
        out.push((cx, cy - k));
    }
    for k in (1..=s).rev() {
        out.push((cx, cy + k));
    }
    for k in (1..=u).rev() {
        out.push((cx - k, cy));
    }
    // spec: S-0303 F7 (diagonal groups: north and west, south and west, north and east, south and east)
    for (a, b, dx, dy) in [(n, u, -1, -1), (s, u, -1, 1), (n, e, 1, -1), (s, e, 1, 1)] {
        for k in 1..=a.min(b) {
            out.push(clamp(cx + k * dx, cy + k * dy));
        }
        if a > 1 && b > 1 {
            // spec: S-0303 F7 (the two extra sectors C + (2dₓ, d_y) and C + (dₓ, 2d_y))
            out.push(clamp(cx + 2 * dx, cy + dy));
            out.push(clamp(cx + dx, cy + 2 * dy));
        }
    }
    out
}

/// The candidates of the flying-object search (S-0303 claim 6, F7) for a
/// detonation with centre cell `c` and truncated spread `spread_trunc` in cells (the
/// warhead's spread S): the tracked objects of the sector set in sector
/// order, each once; with `quirks.flying_search_sectors` off every tracked
/// object once, in the order of starting its current tracking period.
pub fn candidates(w: &World, c: CellXY, spread_trunc: i32) -> Vec<ObjectId> {
    if !w.rules().quirks.flying_search_sectors {
        // spec: S-0303 Edge cases and claim 11 (quirks.flying_search_sectors off: every tracked flying object once, in the global order of starting the current tracking period; sector crossings do not reorder it)
        return w.flying.order.clone();
    }
    // spec: S-0303 F7 (ρ = max(trunc(S), 1) cells)
    let rho = spread_trunc.max(1);
    let mut out: Vec<ObjectId> = Vec::new();
    for s in sector_set(w, c, rho) {
        if let Some(list) = w.flying.sectors.get(&s) {
            for &id in list {
                // spec: S-0303 F7 (every tracked flying object of the set is a candidate exactly once)
                if !out.contains(&id) {
                    out.push(id);
                }
            }
        }
    }
    out
}
