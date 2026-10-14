//! Releasing a done unit from a factory building (S-0404 claims 13 and
//! 14): the four rows of claim 13 (ground vehicle factory, naval vehicle
//! factory, infantry factory and cloning building, aircraft factory), the
//! exit lists of its Constants with `quirks.foundation_exit_lists`, and the
//! copies of claim 14.
//!
//! What happens after the unit appears is open (S-0404 Open questions,
//! request Q-0120): how a ground vehicle factory's unloading takes the
//! vehicle out, where a unit goes after leaving, the docking position of
//! an aircraft and "a cell near" the factory, how many docking places an
//! infantry factory has and the exact cell-entry test of the infantry exit
//! search. The interim rules for those parts are marked `Q-0120` below and
//! listed in the interim table of `docs/development.md`.

use glasnost_map::CellXY;
use glasnost_num::geom::{Facing, direction, n_step};
use serde::{Deserialize, Serialize};

use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// The result of a release (S-0404 claims 7 and 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The unit is on the map.
    Released,
    /// The factory building cannot let it out now.
    Busy,
    /// No exit, or the unit cannot be put on the map.
    Failed,
}

/// A released unit on its way out of its factory building (S-0404 claim
/// 13: it keeps the factory's rally point as its later destination). What
/// ends the way out is the interim rule of request Q-0120.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leaving {
    /// The factory building it leaves.
    pub factory: ObjectId,
    /// The exit cell it walks out to (infantry); none for a vehicle let
    /// out by a ground vehicle factory's unloading.
    pub exit: Option<CellXY>,
    /// The later destination: the factory's rally point.
    pub later: Option<Target>,
}

// spec: S-0404 Constants (facing of an appearing vehicle, ground and naval factories: 16384, map east)
const VEHICLE_FACING: Facing = 16384;
// spec: S-0404 Constants (facing of an aircraft entering from the edge: 0)
const EDGE_FACING: Facing = 0;

/// A foundation word: W, H and the 3 × 3 refinery word.
type Word = (i32, i32, bool);

/// The exit lists of S-0404 Constants per foundation word (W, H, the 3 × 3
/// refinery word), cells (*a*, *b*) from the top-left cell, in the order
/// tried.
// spec: S-0404 Constants (exit lists per foundation word, gaps and repeats included)
const EXIT_LISTS: &[(Word, &[(i32, i32)])] = &[
    (
        (1, 1, false),
        &[
            (0, 1),
            (-1, 1),
            (1, 1),
            (-1, 0),
            (1, 0),
            (0, -1),
            (-1, -1),
            (1, -1),
        ],
    ),
    (
        (2, 1, false),
        &[
            (0, 1),
            (1, 1),
            (-1, 1),
            (2, 1),
            (-1, 0),
            (2, 0),
            (0, -1),
            (1, -1),
            (-1, -1),
            (2, -1),
        ],
    ),
    (
        (1, 2, false),
        &[
            (0, 2),
            (1, 2),
            (-1, 2),
            (-1, 1),
            (1, 1),
            (-1, 0),
            (1, 0),
            (0, -1),
            (-1, -1),
            (1, -1),
        ],
    ),
    (
        (2, 2, false),
        &[
            (0, 2),
            (1, 2),
            (-1, 2),
            (2, 2),
            (-1, 1),
            (2, 1),
            (-1, 0),
            (2, 0),
            (0, -1),
            (1, -1),
            (-1, -1),
            (2, -1),
        ],
    ),
    (
        (2, 3, false),
        &[
            (0, 3),
            (1, 3),
            (-1, 3),
            (2, 3),
            (-1, 2),
            (2, 2),
            (-1, 1),
            (2, 1),
            (-1, 0),
            (2, 0),
            (0, -1),
            (1, -1),
            (-1, -1),
            (2, -1),
        ],
    ),
    (
        (3, 2, false),
        &[
            (0, 2),
            (1, 2),
            (2, 2),
            (-1, 2),
            (3, 2),
            (-1, 1),
            (3, 1),
            (-1, 0),
            (3, 0),
            (0, -1),
            (1, -1),
            (2, -1),
            (-1, -1),
            (3, -1),
        ],
    ),
    ((3, 3, false), THREE_BY_THREE),
    (
        (3, 5, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (-1, 0),
            (3, 0),
            (-1, 1),
            (3, 1),
            (-1, 2),
            (3, 2),
            (-1, 3),
            (3, 3),
            (-1, 4),
            (3, 4),
            (-1, 5),
            (0, 5),
            (1, 5),
            (2, 5),
            (3, 5),
        ],
    ),
    (
        (4, 2, false),
        &[
            (0, 2),
            (1, 2),
            (2, 2),
            (3, 2),
            (-1, 2),
            (4, 2),
            (-1, 1),
            (4, 1),
            (-1, 0),
            (4, 0),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (-1, -1),
            (4, -1),
        ],
    ),
    ((3, 3, true), &[(0, 0)]),
    (
        (1, 3, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (1, 1),
            (-1, 2),
            (1, 2),
            (-1, 3),
            (0, 3),
            (1, 3),
        ],
    ),
    (
        (3, 1, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (-1, 0),
            (3, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (2, 1),
            (3, 1),
        ],
    ),
    // spec: S-0404 Constants (the 4 × 3 list is the 3 × 3 list)
    ((4, 3, false), THREE_BY_THREE),
    (
        (1, 4, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (1, 1),
            (-1, 2),
            (1, 2),
            (-1, 3),
            (1, 3),
            (-1, 4),
            (0, 4),
            (1, 4),
        ],
    ),
    (
        (1, 5, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (1, 1),
            (-1, 2),
            (1, 2),
            (-1, 3),
            (1, 3),
            (-1, 4),
            (1, 4),
            (-1, 5),
            (0, 5),
            (1, 5),
        ],
    ),
    (
        (2, 6, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-1, 0),
            (2, 0),
            (-1, 1),
            (2, 1),
            (-1, 2),
            (2, 2),
            (-1, 3),
            (2, 3),
            (-1, 5),
            (2, 5),
            (-1, 6),
            (0, 6),
            (1, 6),
            (2, 6),
        ],
    ),
    (
        (2, 5, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-1, 0),
            (2, 0),
            (-1, 1),
            (2, 1),
            (-1, 2),
            (2, 2),
            (-1, 3),
            (2, 3),
            (-1, 5),
            (0, 5),
            (1, 5),
            (2, 5),
        ],
    ),
    (
        (5, 3, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (4, -1),
            (5, -1),
            (-1, 0),
            (5, 0),
            (-1, 1),
            (5, 1),
            (-1, 2),
            (5, 2),
            (-1, 3),
            (0, 3),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
            (5, 3),
        ],
    ),
    (
        (4, 4, false),
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (3, -1),
            (4, -1),
            (-1, 4),
            (0, 4),
            (1, 4),
            (2, 4),
            (3, 4),
            (4, 4),
            (-1, 0),
            (-1, 1),
            (-1, 2),
            (-1, 3),
            (-1, 4),
            (4, 0),
            (4, 1),
            (4, 2),
            (4, 3),
            (4, 4),
        ],
    ),
    (
        (3, 4, false),
        &[
            (0, 4),
            (1, 4),
            (2, 4),
            (-1, 4),
            (3, 4),
            (-1, 2),
            (3, 2),
            (-1, 1),
            (3, 1),
            (-1, 0),
            (3, 0),
            (0, -1),
            (1, -1),
            (2, -1),
            (-1, -1),
            (3, -1),
        ],
    ),
    ((6, 4, false), &[(2, -1)]),
    ((0, 0, false), &[]),
];

/// The 3 × 3 list, which the 4 × 3 word shares (S-0404 Constants).
// spec: S-0404 Constants (the 3 × 3 exit list)
const THREE_BY_THREE: &[(i32, i32)] = &[
    (0, 3),
    (1, 3),
    (2, 3),
    (-1, 3),
    (3, 3),
    (-1, 2),
    (3, 2),
    (-1, 1),
    (3, 1),
    (-1, 0),
    (3, 0),
    (0, -1),
    (1, -1),
    (2, -1),
    (-1, -1),
    (3, -1),
];

/// The full ring of outside cells around a W × H foundation (S-0404 Edge
/// cases, `quirks.foundation_exit_lists` off): the row below from left to
/// right, corners included; the cells beside it from the bottom row
/// upwards, left before right; the row above from left to right, corners
/// included. A foundation without cells has an empty ring (interim,
/// request Q-0195).
pub fn ring(fw: i32, fh: i32) -> Vec<(i32, i32)> {
    if fw <= 0 || fh <= 0 {
        // Q-0195 (interim: a foundation without cells keeps its empty list with the switch off too)
        return Vec::new();
    }
    let mut v = Vec::new();
    // spec: S-0404 Edge cases (the row below the foundation from left to right, corners included)
    for a in -1..=fw {
        v.push((a, fh));
    }
    // spec: S-0404 Edge cases (then the cells beside it from the bottom row upwards, left before right)
    for b in (0..fh).rev() {
        v.push((-1, b));
        v.push((fw, b));
    }
    // spec: S-0404 Edge cases (then the row above from left to right, corners included)
    for a in -1..=fw {
        v.push((a, -1));
    }
    v
}

/// The exit list of a foundation (S-0404 Constants; the full ring with
/// `quirks.foundation_exit_lists` off). A size outside the closed set of
/// foundation words (S-0108; the importer gives 1 × 1 for an unknown word)
/// cannot come from imported data and takes its ring.
pub fn exit_list(fw: i32, fh: i32, refinery: bool, exact: bool) -> Vec<(i32, i32)> {
    if !exact {
        return ring(fw, fh);
    }
    // spec: S-0404 Constants (the 3 × 3 refinery word has its own list)
    let key = (fw, fh, refinery && (fw, fh) == (3, 3));
    EXIT_LISTS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or_else(|| ring(fw, fh), |(_, l)| l.to_vec())
}

/// The start cell *S* of an aircraft's edge entry (S-0404 claim 13): `local`
/// is the local area (*L*ₓ, *L*ᵧ, *L*w, *L*h), `m` the map width and `b` the
/// cell of the factory's reference point.
// spec: S-0009 claim 4 (the local area: four numbers)
pub fn edge_start(local: [i32; 4], m: i32, b: CellXY) -> CellXY {
    let [lx, ly, lw, _] = local;
    // spec: S-0404 claim 13 (Q = (Lx + Ly + 1, M − Lx + Ly))
    let q = (lx + ly + 1, m - lx + ly);
    // spec: S-0404 claim 13 (S = (Qx − 1 + Lw, Qy − Lw) when (bx − Qx) − (by − Qy) > Lw, else (Qx − 1, Qy))
    if (b.x - q.0) - (b.y - q.1) > lw {
        CellXY::new(q.0 - 1 + lw, q.1 - lw)
    } else {
        CellXY::new(q.0 - 1, q.1)
    }
}

/// Releases `unit` from factory building `k` (S-0404 claim 13; claim 14's
/// copies for infantry).
pub fn release(w: &mut World, k: ObjectId, unit: ObjectId) -> Outcome {
    let Some(kind) = w.obj(unit).map(|u| u.kind) else {
        return Outcome::Failed;
    };
    let naval = w
        .obj(k)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
        .is_some_and(|b| b.naval);
    match kind {
        // spec: S-0404 claim 13 (infantry factory row; claim 14's copies)
        Kind::Infantry => infantry(w, k, unit, true),
        // spec: S-0404 claim 13 (naval vehicle factory row)
        Kind::Vehicle if naval => naval_row(w, k, unit),
        // spec: S-0404 claim 13 (ground vehicle factory row)
        Kind::Vehicle => ground(w, k, unit),
        // spec: S-0404 claim 13 (aircraft factory row)
        Kind::Aircraft => aircraft(w, k, unit, super::production::dock_free(w, k)),
        _ => Outcome::Failed,
    }
}

/// K's rally point (S-0213 claim 20).
fn rally(w: &World, k: ObjectId) -> Option<Target> {
    w.obj(k).and_then(|b| b.guard_spot)
}

/// The point of a target.
fn point(w: &World, t: Target) -> Pos {
    match t {
        Target::Cell(c) => Pos::centre(c, 0),
        Target::Object(o) => w.reference_point(o),
    }
}

/// Puts `unit` on the map at `p` on the surface there (`z` added).
fn put(w: &mut World, unit: ObjectId, x: i32, y: i32, dz: i32, facing: Facing) {
    let z = w.surface_at(x, y) + dz;
    crate::objects::place(w, unit, Pos::new(x, y, z), facing);
}

/// A move to `t` (S-0404 claim 13, "a move order"): destination and an
/// immediate start of *move*.
fn move_to(w: &mut World, unit: ObjectId, t: Target) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    let Some(o) = w.obj_mut(unit) else { return };
    let before = o.dest;
    o.dest = Some(t);
    o.beh.queue_and_start_at_once(Behaviour::Move, tick, stale);
    if o.kind == Kind::Infantry {
        // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
        crate::infantry::destination_assigned(w, unit, before);
    }
}

/// Whether a building's foundation covers cell `c` (S-0406 claim 2).
fn covers(w: &World, k: ObjectId, c: CellXY) -> bool {
    w.obj(k).is_some_and(|b| {
        super::placement::foundation_cells(w, b.type_index, b.pos.cell()).contains(&c)
    })
}

/// The type's exit displacement (S-0404 Import mapping; absent 0, 0, 0).
// spec: S-0404 Import mapping (ExitCoord: an integer triple; absent 0, 0, 0)
fn exit_coord(w: &World, k: ObjectId) -> [i32; 3] {
    w.obj(k)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
        .and_then(|b| b.raw.econ.exit_coord)
        .unwrap_or([0; 3])
}

/// The ground vehicle factory row of S-0404 claim 13.
fn ground(w: &mut World, k: ObjectId, unit: ObjectId) -> Outcome {
    let Some(b) = w.obj(k).cloned() else {
        return Outcome::Failed;
    };
    let mut k = k;
    if b.beh.current == Some(Behaviour::Unload) {
        // spec: S-0404 claim 13 (K unloading: the first other building of the house, in acquisition order, of the same type whose current behaviour is guard; its power switch and claim 7's conditions are not judged)
        let Some(owner) = b.owner else {
            return Outcome::Busy;
        };
        let twin = w.houses[usize::from(owner.0)]
            .econ
            .acquired
            .iter()
            .copied()
            .find(|&x| {
                x != k
                    && w.obj(x).is_some_and(|o| {
                        o.kind == Kind::Building
                            && o.on_map
                            && o.owner == Some(owner)
                            && o.type_index == b.type_index
                            && o.beh.current == Some(Behaviour::Guard)
                    })
            });
        // spec: S-0404 claim 13 (no such building: busy)
        let Some(twin) = twin else {
            return Outcome::Busy;
        };
        k = twin;
    }
    let Some(b) = w.obj(k).cloned() else {
        return Outcome::Failed;
    };
    let d = exit_coord(w, k);
    // spec: S-0404 claim 13 (the exit point: K's position, the centre of its top-left cell, plus the exit displacement; c when the displacement is 0, 0, 0)
    let p = if d == [0; 3] {
        w.reference_point(k)
    } else {
        let tl = Pos::centre(b.pos.cell(), b.pos.z);
        Pos::new(tl.x + d[0], tl.y + d[1], tl.z + d[2])
    };
    // spec: S-0404 claim 13 and Constants (it appears inside K facing 16384)
    crate::objects::place(w, unit, p, VEHICLE_FACING);
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    if let Some(o) = w.obj_mut(k) {
        // spec: S-0404 claim 13 (K then starts unloading)
        o.beh.force(Behaviour::Unload, tick, stale);
    }
    let later = rally(w, k);
    if let Some(o) = w.obj_mut(unit) {
        // Q-0120 (interim: the vehicle is in contact with K while K lets it out)
        o.contact = Some(k);
        // spec: S-0404 claim 13 (the unit keeps K's rally point as its later destination)
        o.econ.leaving = Some(Leaving {
            factory: k,
            exit: None,
            later,
        });
    }
    Outcome::Released
}

/// K's unloading takes a vehicle out (request Q-0120, interim): the vehicle
/// is moved, keeping its facing, to the centre of the cell that the
/// nearby-cell search (S-0212 claim 25, scatter use) finds around K's
/// reference point cell. Returns whether it was moved.
fn take_out(w: &mut World, k: ObjectId, id: ObjectId) -> bool {
    w.note("Q-0120: a ground vehicle factory's unloading moves the vehicle to the nearby cell around its centre");
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let class = w.rules().unit(o.kind, o.type_index).and_then(|u| u.class);
    let centre = w.reference_point(k).cell();
    crate::regions::ensure(w);
    let Some(c) = crate::path::nearby(
        w,
        centre,
        crate::path::NearbyUse::Scatter { class, deck: false },
    ) else {
        return false;
    };
    let from = o.pos.cell();
    crate::drive::release_claims(w, id);
    w.cells.leave(from, o.layer, id);
    let p = Pos::centre(c, 0);
    let z = w.surface_at(p.x, p.y);
    w.cells.enter(c, o.layer, id, false);
    w.cells.claim_vehicle(c, o.layer, true);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(p.x, p.y, z);
        if let Some(d) = ob.drive.as_mut() {
            d.claims = vec![c];
        }
    }
    // spec: S-0211 claim 5 (an object put into a cell, or taken out of one, recomputes it)
    crate::load::recalc_cells(w, &[from, c]);
    crate::regions::cell_changed(w, from);
    crate::regions::cell_changed(w, c);
    true
}

/// Whether cell `c` holds a unit, infantry, aircraft or building (S-0404
/// claim 13, the naval exit cell).
fn holds_object(w: &World, c: CellXY) -> bool {
    [crate::world::Layer::Ground, crate::world::Layer::Deck]
        .iter()
        .flat_map(|&l| w.cells.list(c, l).iter())
        .any(|&id| {
            w.obj(id).is_some_and(|o| {
                matches!(
                    o.kind,
                    Kind::Vehicle | Kind::Infantry | Kind::Aircraft | Kind::Building
                )
            })
        })
}

/// The naval vehicle factory row of S-0404 claim 13.
fn naval_row(w: &mut World, k: ObjectId, unit: ObjectId) -> Outcome {
    let in_contact = w.objects.values().any(|o| o.contact == Some(k));
    if !in_contact {
        let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
        let tick = w.tick;
        if let Some(o) = w.obj_mut(k) {
            // spec: S-0404 claim 13 (when K has no unit in contact, K starts unloading)
            o.beh.force(Behaviour::Unload, tick, stale);
        }
    }
    let c = w.reference_point(k);
    let cc = c.cell();
    let goal = rally(w, k);
    let mut exit = None;
    if let Some(t) = goal {
        let r = point(w, t);
        let table = w.rules().quirks.table_direction;
        // spec: S-0404 claim 13 (e = the direction from c to the rally point rounded to 8 steps, S-0204 F3 and F4)
        let e = n_step(
            direction(i64::from(r.x - c.x), i64::from(r.y - c.y), table),
            8,
        ) as usize;
        let (dx, dy) = crate::regions::STEPS[e];
        // spec: S-0404 claim 13 (step from c's cell in direction e, S-0204 claim 9, until the first cell not covered by K)
        let mut cell = cc;
        let limit = w.obj(k).map_or(0, |b| b.foundation.0 + b.foundation.1) + 1;
        for _ in 0..=limit {
            if !covers(w, k, cell) {
                break;
            }
            cell = CellXY::new(cell.x + dx, cell.y + dy);
        }
        let water = w
            .cells
            .cell(cell)
            .is_some_and(|x| x.land == glasnost_map::terrain::land::WATER);
        // spec: S-0404 claim 13 (that cell is the exit cell when it is water, holds no unit, infantry, aircraft or building, and is usable)
        if water && !holds_object(w, cell) && w.cells.usable(cell) {
            exit = Some(cell);
        }
    }
    if exit.is_none() {
        let class = w
            .obj(unit)
            .and_then(|u| w.rules().unit(u.kind, u.type_index))
            .and_then(|u| u.class);
        crate::regions::ensure(w);
        // spec: S-0404 claim 13 (otherwise the nearby-cell search of S-0212 claim 25 around c's cell for the unit's movement kind)
        exit = crate::path::nearby(
            w,
            cc,
            crate::path::NearbyUse::Scatter { class, deck: false },
        );
    }
    let Some(exit) = exit else {
        // spec: S-0404 claim 13 (a unit that cannot be put on the map: failed)
        return Outcome::Failed;
    };
    let p = Pos::centre(exit, 0);
    // spec: S-0404 claim 13 and Constants (the exit cell's centre, facing 16384)
    put(w, unit, p.x, p.y, 0, VEHICLE_FACING);
    if let Some(t) = goal {
        // spec: S-0404 claim 13 (with a rally point, a move order to it)
        move_to(w, unit, t);
    }
    Outcome::Released
}

/// The infantry row of S-0404 claim 13 for an infantry factory or a
/// cloning building `k`; `copies`: claim 14's copies are made (not for a
/// copy itself).
fn infantry(w: &mut World, k: ObjectId, unit: ObjectId, copies: bool) -> Outcome {
    let Some(b) = w.obj(k).cloned() else {
        return Outcome::Failed;
    };
    let Some(bt) = w.rules().buildings.get(b.type_index as usize).cloned() else {
        return Outcome::Failed;
    };
    // Q-0120 (interim: an infantry factory or cloning building has one docking place)
    let docks = 1;
    let held = w.objects.values().filter(|o| o.contact == Some(k)).count();
    if held >= docks {
        // spec: S-0404 claim 13 (none of K's docking places is free: busy)
        return Outcome::Busy;
    }
    let e = &bt.raw.econ;
    let (x0, y0) = (b.pos.cell().x, b.pos.cell().y);
    let (fw, fh) = b.foundation;
    let mut doors = Vec::new();
    // spec: S-0404 claim 13 (door cells: (x0 + 1, y0 + 2), (x0 + 2, y0 + 2), (x0 + 2, y0 + 1) for the first, second and third kinds, in that order)
    for (has, (a, bb)) in [
        (e.door_first, (1, 2)),
        (e.door_second, (2, 2)),
        (e.door_third, (2, 1)),
    ] {
        if has == Some(true) {
            doors.push(CellXY::new(x0 + a, y0 + bb));
        }
    }
    let exact = w.rules().quirks.foundation_exit_lists;
    let refinery = bt.raw.foundation_refinery == Some(true);
    // spec: S-0404 claim 13 and Constants (then the cells of the foundation's exit list, in list order; quirks.foundation_exit_lists)
    let listed = exit_list(fw, fh, refinery, exact)
        .into_iter()
        .map(|(a, bb)| CellXY::new(x0 + a, y0 + bb));
    let reference = crate::occupancy::reference_level(w, unit);
    w.note("Q-0120: the infantry exit search's cell-entry test is the infantry entry outcome being clear");
    let legal = |w: &World, c: CellXY| {
        // spec: S-0404 claim 13 (a legal map cell, S-0009 claim 1, which the unit could enter now, S-0209 and S-0211)
        // Q-0120 (interim: could enter now is the infantry entry outcome of S-0209, which includes S-0211's test, being clear)
        w.cells.footprint().is_legal(c)
            && crate::occupancy::infantry_outcome(w, unit, c, reference)
                == crate::occupancy::Outcome::Clear
    };
    let door_count = doors.len();
    let Some((i, exit)) = doors
        .into_iter()
        .chain(listed)
        .enumerate()
        .find(|&(_, c)| legal(w, c))
    else {
        // spec: S-0404 claim 13 (no candidate: failed)
        return Outcome::Failed;
    };
    if copies && e.cloning != Some(true) {
        clone_copies(w, &b, unit);
    }
    // spec: S-0404 claim 13 (the foundation cell next to the exit cell: each coordinate outside the foundation moved one cell back into it)
    let inside = CellXY::new(
        exit.x.clamp(x0, x0 + fw.max(1) - 1),
        exit.y.clamp(y0, y0 + fh.max(1) - 1),
    );
    let mut p = Pos::centre(inside, 0);
    let mut dz = 0;
    if i < door_count {
        // spec: S-0404 claim 13 (at a door cell of a door kind the type has, the exit displacement is added)
        let d = e.exit_coord.unwrap_or([0; 3]); // spec: S-0404 Import mapping (ExitCoord: an integer triple; absent 0, 0, 0)
        p = Pos::new(p.x + d[0], p.y + d[1], 0);
        dz = d[2]; // spec: S-0404 Import mapping (the third integer of ExitCoord: height)
    }
    let c = w.reference_point(k);
    let ec = Pos::centre(exit, 0);
    let table = w.rules().quirks.table_direction;
    // spec: S-0404 claim 13 (the facing: the direction from c to the exit cell's centre rounded to 256 steps; a value v is v × 256 facing units, S-0204 claim 10)
    let facing = (n_step(
        direction(i64::from(ec.x - c.x), i64::from(ec.y - c.y), table),
        256,
    ) * 256) as Facing;
    put(w, unit, p.x, p.y, dz, facing);
    let later = rally(w, k);
    if let Some(o) = w.obj_mut(unit) {
        // Q-0120 (interim: the unit takes K's docking place, in contact with K, until it reaches its exit cell)
        o.contact = Some(k);
        // spec: S-0404 claim 13 (it keeps K's rally point as its later destination)
        o.econ.leaving = Some(Leaving {
            factory: k,
            exit: Some(exit),
            later,
        });
    }
    // spec: S-0404 claim 13 (told by K to leave, it walks out to the exit cell)
    move_to(w, unit, Target::Cell(exit));
    Outcome::Released
}

/// Claim 14: each cloning building of the house, in acquisition order,
/// gets a copy of the released infantry and releases it by the infantry
/// row; a copy that is not released does not appear.
fn clone_copies(w: &mut World, k: &crate::world::Object, unit: ObjectId) {
    let Some(owner) = k.owner else { return };
    let Some(u) = w.obj(unit).map(|u| (u.kind, u.type_index)) else {
        return;
    };
    let r = w.rules_arc();
    // spec: S-0404 claim 14 (each cloning building the house owns, in the order the house acquired them)
    let cloners: Vec<ObjectId> = w.houses[usize::from(owner.0)]
        .econ
        .acquired
        .iter()
        .copied()
        .filter(|&x| {
            w.obj(x).is_some_and(|o| {
                o.kind == Kind::Building
                    && o.on_map
                    && o.owner == Some(owner)
                    && r.buildings
                        .get(o.type_index as usize)
                        .is_some_and(|b| b.raw.econ.cloning == Some(true))
            })
        })
        .collect();
    for cb in cloners {
        // spec: S-0404 claim 14 (a new infantry unit of the same type for its owner: a newly created object, its creation draw, S-0202 claim 10)
        let copy = crate::objects::create(w, u.0, u.1, Some(owner));
        // spec: S-0404 claim 14 (released by claim 13's infantry row; nothing paid or refunded)
        if infantry(w, cb, copy, false) != Outcome::Released {
            // spec: S-0404 claim 14 (a copy whose release is busy or failed does not appear)
            super::production::delete_item_object(w, copy);
        }
    }
}

/// The aircraft factory row of S-0404 claim 13; `dock_free`: K has a
/// docking place that is empty or already in contact with this aircraft.
pub fn aircraft(w: &mut World, k: ObjectId, unit: ObjectId, dock_free: bool) -> Outcome {
    let Some(u) = w
        .obj(unit)
        .and_then(|u| w.rules().unit(u.kind, u.type_index))
        .cloned()
    else {
        return Outcome::Failed;
    };
    // spec: S-0404 Import mapping (AirportBound: the type must return to a dock, S-0407)
    let bound = u.raw.econ.airport_bound.unwrap_or(false);
    let goal = rally(w, k);
    if dock_free {
        // Q-0120 (interim: an aircraft's docking position is K's reference point)
        w.note("Q-0120: an aircraft released onto a dock appears at its factory's reference point");
        let c = w.reference_point(k);
        let pose = w.rules().econ.production.pose_dir;
        // spec: S-0404 claim 13 and Constants (facing the game-wide pose direction × 256, S-0204 claim 10; absent 0)
        let facing = (pose.rem_euclid(256) * 256) as Facing;
        put(w, unit, c.x, c.y, 0, facing);
        if !bound && let Some(t) = goal {
            // spec: S-0404 claim 13 (with a rally point, a type that need not return to a dock gets a move order to it)
            move_to(w, unit, t);
        }
        return Outcome::Released;
    }
    if bound {
        // spec: S-0404 claim 13 (a type that must return to a dock, with no docking place: failed)
        return Outcome::Failed;
    }
    let fp = *w.cells.footprint();
    let b = w.reference_point(k).cell();
    let s = edge_start(fp.local, fp.width, b);
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0404 claim 13 (one ranged draw r over 0 … Lh from the scenario stream; none when Lh = 0, S-0202 claim 4)
    let r = w.rng.ranged(0, fp.local[3], half);
    let cell = CellXY::new(s.x + r, s.y + r);
    let p = Pos::centre(cell, 0);
    // spec: S-0404 claim 13 and Constants (that cell's centre, facing 0)
    put(w, unit, p.x, p.y, 0, EDGE_FACING);
    // spec: S-0404 claim 13 (a move order to the rally point, otherwise to a cell near K)
    // Q-0120 (interim: the cell near K is the cell of K's reference point)
    let t = goal.unwrap_or(Target::Cell(b));
    move_to(w, unit, t);
    Outcome::Released
}

/// The end of the way out (request Q-0120, interim), in the production part
/// of every tick: a leaving unit that is gone, off the map, no longer in
/// contact with its factory, out of the factory (a vehicle) or on its exit
/// cell (infantry), or that has stopped moving, breaks contact and moves on
/// to its later destination; then a factory building that is unloading
/// with no unit leaving it goes back to guard.
pub fn leaving_part(w: &mut World) {
    let leaving: Vec<(ObjectId, Leaving)> = w
        .objects
        .values()
        .filter_map(|o| o.econ.leaving.map(|l| (o.id, l)))
        .collect();
    for (id, l) in leaving {
        let Some(o) = w.obj(id) else { continue };
        let alive = o.on_map && o.health > 0;
        let held = o.contact == Some(l.factory);
        if alive && held && l.exit.is_none() && covers(w, l.factory, o.pos.cell()) {
            // Q-0120 (interim: K's unloading takes the vehicle out in the production part after the release)
            take_out(w, l.factory, id);
        }
        let Some(o) = w.obj(id) else { continue };
        let cell = o.pos.cell();
        let out = match l.exit {
            Some(e) => cell == e,
            None => !covers(w, l.factory, cell),
        };
        let stopped =
            l.exit.is_some() && o.dest.is_none() && o.beh.current != Some(Behaviour::Move);
        // Q-0120 (interim: the way out ends when the vehicle is out of the factory, or nowhere to go, or the infantry is on its exit cell or stops; or on losing contact or dying)
        if alive && held && !out && !stopped && l.exit.is_some() {
            continue;
        }
        if let Some(o) = w.obj_mut(id) {
            o.econ.leaving = None;
            if held {
                o.contact = None;
            }
        }
        if alive
            && held
            && let Some(t) = l.later
        {
            // spec: S-0404 claim 13 (the later destination: K's rally point)
            // Q-0120 (interim: it moves on to its later destination once out)
            move_to(w, id, t);
        }
    }
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    let unloading: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|b| b.kind == Kind::Building && b.beh.current == Some(Behaviour::Unload))
        .map(|b| b.id)
        .collect();
    for k in unloading {
        let busy = w
            .objects
            .values()
            .any(|o| o.econ.leaving.is_some_and(|l| l.factory == k));
        if !busy && let Some(b) = w.obj_mut(k) {
            // Q-0120 (interim: K's unloading ends, back to guard, once no unit is leaving it)
            b.beh.force(Behaviour::Guard, tick, stale);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0404 Constants (the 2 × 2 list) and Edge cases (the full ring with quirks.foundation_exit_lists off)
    #[test]
    fn exit_lists_and_rings() {
        assert_eq!(
            exit_list(2, 2, false, true)[..4],
            [(0, 2), (1, 2), (-1, 2), (2, 2)]
        );
        assert_eq!(
            exit_list(2, 2, false, false),
            vec![
                (-1, 2),
                (0, 2),
                (1, 2),
                (2, 2),
                (-1, 1),
                (2, 1),
                (-1, 0),
                (2, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1)
            ]
        );
        // The 4 × 3 list never names a cell beside the right edge.
        let l = exit_list(4, 3, false, true);
        assert!(!l.iter().any(|&(a, b)| a == 4 && (0..3).contains(&b)));
        assert!(exit_list(4, 3, false, false).contains(&(4, 1)));
        assert_eq!(exit_list(3, 3, true, true), vec![(0, 0)]);
        assert_eq!(exit_list(3, 3, false, true).len(), 16);
        assert_eq!(exit_list(6, 4, false, true), vec![(2, -1)]);
        assert!(exit_list(0, 0, false, true).is_empty());
        assert!(exit_list(0, 0, false, false).is_empty());
        // The 4 × 4 list names (−1, 4) and (4, 4) twice.
        let l = exit_list(4, 4, false, true);
        assert_eq!(l.iter().filter(|&&c| c == (-1, 4)).count(), 2);
        assert_eq!(l.iter().filter(|&&c| c == (4, 4)).count(), 2);
        // Every word of the closed set has a list.
        for (k, l) in EXIT_LISTS {
            assert_eq!(exit_list(k.0, k.1, k.2, true), l.to_vec());
        }
    }

    // spec: S-0404 claim 13 (the start cell S of the edge entry by the two cases)
    #[test]
    fn edge_start_cases() {
        // Lx = 2, Ly = 3, Lw = 10, M = 40: Q = (6, 41).
        let local = [2, 3, 10, 6];
        // (bx − Qx) − (by − Qy) = (20 − 6) − (30 − 41) = 25 > 10: the right edge.
        assert_eq!(
            edge_start(local, 40, CellXY::new(20, 30)),
            CellXY::new(15, 31)
        );
        // (10 − 6) − (40 − 41) = 5 ≤ 10: the left edge.
        assert_eq!(
            edge_start(local, 40, CellXY::new(10, 40)),
            CellXY::new(5, 41)
        );
    }
}
