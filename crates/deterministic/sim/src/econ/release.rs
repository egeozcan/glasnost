//! Releasing a done unit from a factory building (S-0404 claims 13 and
//! 14): the four rows of claim 13 (ground vehicle factory, naval vehicle
//! factory, infantry factory and cloning building, aircraft factory), the
//! exit lists of its Constants with `quirks.foundation_exit_lists`, and the
//! copies of claim 14; and factory unloading (S-0417): the stages of a
//! ground vehicle factory's *unload* with its door, apron clearing and exit
//! track, a naval factory's two processings, the tether of a released unit
//! and its clear report, the run-away reply and the arrival fallback.
//!
//! An infantry unit released without a rally destination rests where it
//! is put; S-0417 claim 17's request sends it off the factory's cell in
//! its next turn (`scatter::resting_turn`). What stays open (S-0417 Open
//! questions): the behaviour of jumpjet infantry released that way, and
//! the draws of putting a released unit on the map (none are taken).
//! An infantry unit's cell-arrival update is its walking arrival tick
//! (S-0208 claim 23), called from `walk`.
//! The production and idle animations of a factory are not modelled
//! (building animations are a seam), so they take no draws.

use glasnost_map::CellXY;
use glasnost_num::geom::{Facing, direction, n_step};
use serde::{Deserialize, Serialize};

use crate::behaviour::Door;
use crate::combat::effects::{Effect, Sound};
use crate::econ::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::scatter::ScatterRequest;
use crate::world::{Layer, ObjectId, Pos, Target, World};

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

/// A released unit in contact with its factory building (S-0417 claims 2,
/// 7 and 10): the factory and whether the two are tethered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exit {
    /// The factory building K.
    pub factory: ObjectId,
    /// The tether link with K (glossary *tether link*).
    pub tethered: bool,
    /// The contact place of K it is in contact through, numbered from 0
    /// (S-0417 claim 14, S-0404 claim 15).
    #[serde(default)]
    pub place: u32,
}

/// K's contact places *n* = max(*d*, 1), *d* the imported docking-place
/// count (S-0417 claim 14).
pub fn contact_places(w: &World, k: ObjectId) -> u32 {
    // spec: S-0417 claim 14 and S-0404 Import mapping (NumberOfDocks, rules, absent 1; n = max(d, 1))
    let d = building_rules(w, k).map_or(1, |b| b.raw.econ.number_of_docks.unwrap_or(1));
    d.max(1) as u32
}

/// The first of K's places no unit is in contact through (S-0417 claim
/// 14: a place is free while no unit is in contact through it); `limit`
/// when every place below it is held.
pub fn first_free_place(w: &World, k: ObjectId, limit: u32) -> u32 {
    let held: std::collections::BTreeSet<u32> = w
        .objects
        .values()
        .filter(|o| o.contact == Some(k))
        .filter_map(|o| o.econ.exit.filter(|e| e.factory == k).map(|e| e.place))
        .collect();
    (0..limit).find(|i| !held.contains(i)).unwrap_or(limit)
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
/// included. A foundation without cells has an empty ring (S-0404 Edge
/// cases, Q-0195).
pub fn ring(fw: i32, fh: i32) -> Vec<(i32, i32)> {
    if fw <= 0 || fh <= 0 {
        // spec: S-0404 Edge cases (Q-0195: for the 0 × 0 word the exit list is empty with the switch off as well as on; no ring is inferred)
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
    present_exit_list(fw, fh, refinery, exact).unwrap_or_else(|| ring(fw, fh))
}

/// The foundation exit list of S-0404 Constants when one is present: with
/// `quirks.foundation_exit_lists` off the ring alternative; on, the listed
/// word's list. Every imported foundation word has a present list (S-0314
/// claim 25; S-0108 claim 8's parser falls back to a supported word), so
/// `None`, a size without a listed word, arises only from synthetic data.
pub fn present_exit_list(fw: i32, fh: i32, refinery: bool, exact: bool) -> Option<Vec<(i32, i32)>> {
    if !exact {
        return Some(ring(fw, fh));
    }
    // spec: S-0404 Constants (the 3 × 3 refinery word has its own list)
    let key = (fw, fh, refinery && (fw, fh) == (3, 3));
    EXIT_LISTS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, l)| l.to_vec())
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
    let out = match kind {
        // spec: S-0404 claim 13 (infantry factory row; claim 14's copies)
        Kind::Infantry => infantry(w, k, unit, true),
        // spec: S-0404 claim 13 (naval vehicle factory row)
        Kind::Vehicle if naval => naval_row(w, k, unit),
        // spec: S-0404 claim 13 (ground vehicle factory row)
        Kind::Vehicle => ground(w, k, unit),
        // spec: S-0404 claim 13 (aircraft factory row)
        Kind::Aircraft => aircraft(w, k, unit, super::production::dock_free(w, k)),
        _ => Outcome::Failed,
    };
    if out == Outcome::Released
        && let (Some(u), Some(f)) = (w.obj(unit), w.obj(k))
    {
        // A presentation event: the unit-ready sound and announcement (S-0701 claims 23 and 28); no state, no draw.
        let e = Effect::Sound {
            sound: Sound::UnitReady,
            of: Some(TypeRef {
                kind: u.kind,
                index: u.type_index,
            }),
            house: u.owner,
            pos: Some(f.pos),
        };
        w.effects.push(e);
    }
    out
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
    o.beh.queue_and_start_at_once(Behaviour::Move, tick, stale);
    // spec: S-0107 claim 15 and S-0207 claim 20 (a) (the destination assigned: get-up, and a teleporter's mark moves at once)
    super::assign_destination(w, unit, Some(t));
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
    if b.beh.seen() == Some(Behaviour::Unload) {
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
                            && o.beh.seen() == Some(Behaviour::Guard)
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
    // spec: S-0404 claim 13 (no entry test of the exit point's cell: neither K, nor any other occupant of any house, moving or not, nor its terrain or overlay keeps the unit out, so the cell never gives failed; the unit shares the cell with whatever stands there) and S-0309 claim 31 (Q-2841 answered)
    // spec: S-0404 claim 13 and Constants (it appears inside K facing 16384)
    crate::objects::place(w, unit, p, VEHICLE_FACING);
    // spec: S-0404 claim 13 (the new unit's contact takes the free place of K's max(d, 1) with the lowest number; places are numbered from 1 there and from 0 here)
    let places = contact_places(w, k);
    let mut place = first_free_place(w, k, places);
    if place >= places {
        // spec: S-0404 claim 13 and S-0309 claim 31 (no place free: the unit that holds place 1 loses its contact and tether link with K at this release, and the new unit takes that place)
        place = 0;
        drop_place(w, k, place);
    }
    // spec: S-0417 claim 1 (K queues unloading; claim 2 governs when it becomes current)
    queue_unload(w, k);
    // design: C7-P5 rally spread (quirks.rally_point_shared off: the saved guard spot is a cell of the unit's own round the rally point)
    let spot = crate::orders::rally_cell(w, unit, rally(w, k));
    if let Some(o) = w.obj_mut(unit) {
        // spec: S-0417 claim 1 (a ground vehicle factory releases u into contact with K)
        o.contact = Some(k);
        // spec: S-0417 claim 8 (the saved guard spot: K's rally point copied at release)
        o.guard_spot = spot;
        // spec: S-0417 Terms (the glossary's tether link: held from the moment K begins its unloading with u, at release)
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: true,
            place,
        });
    }
    Outcome::Released
}

/// The unit that holds K's contact place `place` loses its contact and
/// tether link with K; it keeps its behaviours, destination and position
/// (S-0404 claim 13, S-0309 claim 31).
fn drop_place(w: &mut World, k: ObjectId, place: u32) {
    let holder = w
        .objects
        .values()
        .find(|o| {
            o.contact == Some(k)
                && o.econ
                    .exit
                    .is_some_and(|e| e.factory == k && e.place == place)
        })
        .map(|o| o.id);
    if let Some(o) = holder.and_then(|h| w.obj_mut(h)) {
        // spec: S-0404 claim 13 (the holder loses its contact and tether link with K at this release) and S-0309 claim 31 (it keeps its behaviour and stays where it is)
        o.contact = None;
        o.econ.exit = None;
    }
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
        // spec: S-0417 claim 1 (a naval factory queues unloading only when it had no unit in contact)
        queue_unload(w, k);
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
    // design: C7-P5 rally spread (quirks.rally_point_shared off: the move goes to a cell of the unit's own round the rally point)
    if let Some(t) = crate::orders::rally_cell(w, unit, goal) {
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
    let places = contact_places(w, k);
    let place = first_free_place(w, k, places);
    if place >= places {
        // spec: S-0404 claim 13 and S-0417 claim 14 (none of K's n contact places is free: busy)
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
    let legal = |w: &World, c: CellXY| {
        // spec: S-0404 claim 13 (a legal map cell, S-0009 claim 1, which the unit could enter now) and S-0417 claim 14 (it passes only on the free result of the unit's own cell-entry test, S-0209 and S-0211; results that only make a mover wait reject it)
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
    let inside = CellXY::new(toward(exit.x, x0, fw), toward(exit.y, y0, fh));
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
    // design: C7-P5 rally spread (quirks.rally_point_shared off: the saved guard spot is a cell of the unit's own round the rally point)
    let spot = crate::orders::rally_cell(w, unit, rally(w, k));
    if let Some(o) = w.obj_mut(unit) {
        // spec: S-0417 claim 10 (before placement the released unit saves K's rally point as guard spot, or no spot without one)
        o.guard_spot = spot;
    }
    // spec: S-0417 claim 10 (placement makes the unit fall back to rest at once: S-0215 claim 13's guard-spot row gives a rally point off its own cell as destination, queues move and clears the guard spot)
    put(w, unit, p.x, p.y, dz, facing);
    let Some(o) = w.obj(unit) else {
        return Outcome::Failed;
    };
    let dest = o.dest;
    let straight = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.jumpjet || u.raw.teleporter.unwrap_or(false));
    match dest {
        Some(d) if !straight => {
            // spec: S-0417 claim 10 (a unit with a destination, neither jumpjet infantry nor marked as teleporting: the guard spot becomes the destination, move is queued, the destination becomes the exit cell; S-0404 claim 13)
            if let Some(ob) = w.obj_mut(unit) {
                ob.guard_spot = Some(d);
                ob.beh.queue(Some(Behaviour::Move));
            }
            super::assign_destination(w, unit, Some(Target::Cell(exit)));
        }
        // spec: S-0417 claim 10 (a jumpjet or teleporting type heads straight for the rally point, with an empty guard spot)
        Some(_) => {}
        // spec: S-0404 claim 13 and S-0417 claim 10 (no rally point, or one in its own cell: the release gives no destination; the unit rests in guard where it is put, and claim 17 sends a walking or teleporting unit off a cell that holds K; for jumpjet infantry S-0417 Open question 3)
        None => {}
    }
    if let Some(o) = w.obj_mut(unit) {
        // spec: S-0417 claim 10 (then K offers contact, and K's clearance to unload tethers the unit; the exchange takes no draw)
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: true,
            place,
        });
    }
    Outcome::Released
}

/// One exit-cell coordinate adjusted one cell toward the foundation
/// rectangle from `lo` of extent `n` (S-0404 claim 13).
fn toward(v: i32, lo: i32, n: i32) -> i32 {
    // spec: S-0404 claim 13 (at or above lo + n: reduced by 1; below lo: raised by 1; otherwise unchanged; integer cell operations, also for the 0 × 0 word)
    if v >= lo + n {
        v - 1
    } else if v < lo {
        v + 1
    } else {
        v
    }
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
        // spec: S-0404 claim 14 (a new infantry unit of the same type for its owner: a newly created object, its creation draw, S-0202 claim 10; copies in cloning-building acquisition order, an earlier copy's placement draws before the next copy's creation draw)
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
        let (p, place) = pad_position(w, k, unit);
        let pose = w.rules().econ.production.pose_dir;
        // spec: S-0404 claim 13 and Constants (facing the game-wide pose direction × 256, S-0204 claim 10; absent 0)
        let facing = (pose.rem_euclid(256) * 256) as Facing;
        crate::objects::place(w, unit, p, facing);
        if let Some(o) = w.obj_mut(unit) {
            // spec: S-0417 claim 11 (docked at K: K offers contact, then a tether request on the same tick; a type that must return to a dock refuses the tether; no draw)
            o.contact = Some(k);
            o.econ.exit = Some(Exit {
                factory: k,
                tethered: !bound,
                place,
            });
        }
        // spec: S-0501 claim 1 and S-0404 claim 15 (it stands docked on the ground at its pad position; the tether ends at take-off, S-0501 claim 8)
        crate::air::released_docked(w, unit, p);
        if !bound && let Some(t) = goal {
            // spec: S-0404 claim 13 (with a rally point, a type that need not return to a dock gets a move order to it) and S-0501 claim 48 (d) (that move is not a behaviour order: the release gives the aircraft the rally point as its destination and queues move, with none of S-0213 table T3's effects; it becomes current at claim 31's first point)
            crate::air::queue(w, unit, Behaviour::Move);
            // spec: S-0107 claim 15 and S-0207 claim 20 (a) (the destination assigned)
            super::assign_destination(w, unit, Some(t));
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
    // spec: S-0417 claim 14 (with a rally point, move forced with it as destination; without one, the nearby-cell search's result around the cell of K's reference point, or no destination when it finds none; the search takes no draw)
    let dest = match goal {
        Some(t) => Some(t),
        None => edge_nearby(w, unit, b).map(Target::Cell),
    };
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    if let Some(o) = w.obj_mut(unit) {
        // spec: S-0417 claim 14 (the aircraft's move is forced)
        o.beh.force(Behaviour::Move, tick, stale);
    }
    super::assign_destination(w, unit, dest);
    Outcome::Released
}

/// The nearby destination of an edge-entering aircraft without a rally
/// point (S-0417 claim 14): S-0212 claim 25's search around `centre`.
fn edge_nearby(w: &mut World, unit: ObjectId, centre: CellXY) -> Option<CellXY> {
    let o = w.obj(unit)?.clone();
    let u = w.rules().unit(o.kind, o.type_index)?.clone();
    let deck = o.layer == Layer::Deck;
    crate::regions::ensure(w);
    let settings = crate::path::NearbySettings {
        // spec: S-0417 claim 14 (the aircraft type's locomotion class, winged counted as track)
        class: u.class.map(|k| match k {
            glasnost_rules::sim::LocomotionClass::Winged => {
                glasnost_rules::sim::LocomotionClass::Track
            }
            other => other,
        }),
        // spec: S-0417 claim 14 (its own movement zone; the reach region of the centre cell for that zone)
        zone: u.zone,
        reach: Some(crate::regions::reach_read(w, centre, u.zone, deck)),
        // spec: S-0417 claim 14 (deck mode only on a bridge deck; no height condition, further test or reference cell)
        deck,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    // spec: S-0417 claim 14 (the release tick selects among the preferred cells, S-0212 claim 25)
    crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(settings))
}

/// The pad position of a docked aircraft (S-0404 claim 15) and the
/// contact place it takes: *c*, K's foundation centre with its world
/// height, plus the docking displacement of its place for a pad or
/// repair building.
pub fn pad_position(w: &World, k: ObjectId, unit: ObjectId) -> (Pos, u32) {
    let c = w.reference_point(k);
    let Some(b) = building_rules(w, k) else {
        return (c, 0);
    };
    // spec: S-0404 Import mapping (NumberOfDocks, rules, absent 1)
    let count = b.raw.econ.number_of_docks.unwrap_or(1);
    let kept = w
        .obj(unit)
        .filter(|o| o.contact == Some(k))
        .and_then(|o| o.econ.exit)
        .filter(|e| e.factory == k)
        .map(|e| e.place);
    // spec: S-0404 claim 15 (an aircraft already in contact keeps that place; otherwise it takes the first empty place)
    let place = kept.unwrap_or_else(|| first_free_place(w, k, count.max(1) as u32));
    (pad_position_at(w, k, place), place)
}

/// The position of contact place `place` of building `k` (S-0404 claim 15;
/// S-0314 claim 18 for repair depots).
pub fn pad_position_at(w: &World, k: ObjectId, place: u32) -> Pos {
    let c = w.reference_point(k);
    let Some(b) = building_rules(w, k) else {
        return c;
    };
    // spec: S-0404 Import mapping (NumberOfDocks, rules, absent 1)
    let count = b.raw.econ.number_of_docks.unwrap_or(1);
    // spec: S-0404 Import mapping (Helipad, UnitRepair absent: no)
    let pad = b.raw.helipad.unwrap_or(false) || b.raw.unit_repair.unwrap_or(false);
    if !pad {
        // spec: S-0404 claim 15 (without either flag: c)
        return c;
    }
    let offsets = b.raw.econ.docking_offsets.clone().unwrap_or_default();
    let at = |i: usize| offsets.get(i).copied().unwrap_or([0; 3]); // spec: S-0404 Import mapping (new places start at 0, 0, 0)
    let d = if count == 1 {
        // spec: S-0404 claim 15 (a count of 1: the sole displacement, whatever the contact index)
        Some(at(0))
    } else if count <= 0 || place >= count as u32 {
        // spec: S-0404 claim 15 (a count of 0, or a contact index outside the displacement count: c)
        None
    } else {
        Some(at(place as usize))
    };
    match d {
        // spec: S-0404 claim 15 (c + d_i component by component, signed 32-bit wrapping, not rotated)
        Some(d) => Pos::new(
            c.x.wrapping_add(d[0]),
            c.y.wrapping_add(d[1]),
            c.z.wrapping_add(d[2]),
        ),
        None => c,
    }
}

/// The stages of a factory's unloading (S-0417 table S1), kept in the
/// progress count, which the behaviour start of *unload* sets to 0.
mod stage {
    // nocite: our own numbering of S-0417 table S1's states, in table order
    pub const STARTING: i32 = 0;
    pub const APRON: i32 = 1; // nocite: our numbering (apron clearing)
    pub const LETTING_OUT: i32 = 2; // nocite: our numbering (letting out)
    pub const WAITING: i32 = 3; // nocite: our numbering (waiting)
    pub const CLOSING: i32 = 4; // nocite: our numbering (closing)
}

/// A building type's rules.
fn building_rules(w: &World, k: ObjectId) -> Option<&crate::rules::BuildingRules> {
    w.obj(k)
        .filter(|b| b.kind == Kind::Building)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
}

/// Whether building `k` is a vehicle factory: its type has the vehicle
/// factory flag (S-0404 Import mapping, `WeaponsFactory`; S-0417 Import
/// mapping).
pub fn vehicle_factory(w: &World, k: ObjectId) -> bool {
    building_rules(w, k).is_some_and(|b| b.raw.weapons_factory.unwrap_or(false))
}

/// K's door time *T*d (S-0417 F2: S-0215 F9's conversion of the deploy
/// time; a negative duration behaves as 0).
fn door_time(w: &World, k: ObjectId) -> i32 {
    // spec: S-0417 F2 (S-0215 F9's deploy-time conversion; negative converted durations behave as 0)
    building_rules(w, k).map_or(0, |b| b.gate_door_time.max(0))
}

/// The unit in contact with K (S-0417 claim 2, "contact"), the first in
/// object order.
fn contacted(w: &World, k: ObjectId) -> Option<ObjectId> {
    w.objects
        .values()
        // A vehicle in depot contact with a repairing naval yard is not a
        // factory release (S-0314 claim 17 and S-0417 are separate contacts).
        .find(|o| o.contact == Some(k) && o.depot.link.is_none_or(|l| l.depot != k))
        .map(|o| o.id)
}

/// Whether unit `id` is tethered to the building it is in contact with by
/// a factory release (S-0417 claims 2 and 10).
pub fn tethered(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.econ
            .exit
            .is_some_and(|e| e.tethered && o.contact == Some(e.factory))
    })
}

/// The exit apron cell E of factory K (S-0417 claim 3, F3 and Edge cases).
pub fn apron(w: &World, k: ObjectId) -> Option<CellXY> {
    let b = w.obj(k)?;
    let bt = building_rules(w, k)?;
    let (fw, fh) = b.foundation;
    let refinery = bt.raw.foundation_refinery == Some(true);
    // spec: S-0417 claim 3 and Edge cases (apron selection uses the original lists under either state of quirks.foundation_exit_lists)
    let list = exit_list(fw, fh, refinery, true);
    let short = w.rules().quirks.factory_apron_short_list; // spec: S-0417 Edge cases (quirks.factory_apron_short_list)
    // spec: S-0417 Constants (apron list position 11, counted from 1)
    let (a, bb) = match list.get(10) {
        Some(&p) => p,
        None if short => {
            // spec: S-0417 Edge cases (by the original list's length L: L = 10 gives (32767, 32767), L < 10 gives (0, 0); engine constants)
            if list.len() == 10 {
                (32767, 32767)
            } else {
                (0, 0)
            }
        }
        // spec: S-0417 Edge cases (the fixed alternative, switch off: the original short list's last pair, (0, 0) for an empty list)
        None => list.last().copied().unwrap_or((0, 0)),
    };
    let c = b.pos.cell();
    // spec: S-0417 F3 (E = (x0 + a − 1, y0 + b), each result reduced into −32768 … 32767)
    let x = i32::from(c.x.wrapping_add(a).wrapping_sub(1) as i16);
    let y = i32::from(c.y.wrapping_add(bb) as i16);
    Some(CellXY::new(x, y))
}

/// The points of the exit track (S-0417 table T1) before its target, as
/// displacements from E's centre with facing 64; the 69th point is the
/// target itself (E's centre).
pub fn exit_track_points() -> Vec<crate::tracks::Point> {
    // spec: S-0417 Constants (exit track facing 64, map east)
    const F: u8 = 64;
    // spec: S-0417 table T1 (points 1 to 3, then point 4)
    let mut v = vec![
        (-670, -68, F),
        (-660, -67, F),
        (-650, -66, F),
        (-639, -65, F),
    ];
    // spec: S-0417 table T1 (points 5 to 68: x = −630 + 10 (n − 5), y = n − 69)
    for n in 5..=68 {
        v.push((-630 + 10 * (n - 5), n - 69, F));
    }
    v
}

/// Whether cell `c`'s ground layer lists an infantry unit, vehicle,
/// aircraft or building other than K (S-0417 claim 4).
fn occupied(w: &World, c: CellXY, k: ObjectId) -> bool {
    w.cells.list(c, Layer::Ground).iter().any(|&x| {
        x != k
            && w.obj(x).is_some_and(|o| {
                matches!(
                    o.kind,
                    Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
                )
            })
    })
}

/// Idle apron clearing (S-0417 claim 13), in a processing of an unarmed
/// building without the stupid-guard-mode flag in *guard*, *sticky* or
/// *area guard*, before its delay draw; `first`: the first processing
/// after the behaviour started.
pub fn idle_apron(w: &mut World, k: ObjectId, first: bool) {
    // spec: S-0417 claim 13 (a building whose type has the vehicle factory flag, naval or not; Refinery boundary: a refinery clears only if it is also a vehicle factory, and only E, never its docking cell as such)
    if !vehicle_factory(w, k) {
        return;
    }
    if first {
        // spec: S-0417 claim 13 (the first processing after the behaviour starts returns the animation to idle and makes no apron test)
        w.note("S-0417 claim 13: the factory's animation returns to idle (a seam: no animation state in the simulation)");
        return;
    }
    // spec: S-0417 claim 13 (every later processing: claim 4's blocked test on E and its requests, E then the occupied neighbours; nothing else changes)
    if let Some(e) = apron(w, k) {
        clear_apron(w, k, e);
    }
}

/// Apron clearing (S-0417 claim 4). Returns whether E is free.
fn clear_apron(w: &mut World, k: ObjectId, e: CellXY) -> bool {
    if !occupied(w, e, k) {
        // spec: S-0417 claim 4 (neighbours receive nothing when E is free)
        return true;
    }
    // spec: S-0417 claim 4 (every object in E's ground layer: forced, move-anywhere, no threat point; S-0209 claim 11 receipt)
    crate::scatter::cell_request(
        w,
        e,
        Layer::Ground,
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: true,
        },
    );
    let ep = Pos::centre(e, 0);
    let threat = Pos::new(ep.x, ep.y, w.surface_at(ep.x, ep.y));
    // spec: S-0417 claim 4 (then the apron neighbours in step-direction order 0 to 7, S-0204 claim 9)
    for (dx, dy) in crate::regions::STEPS {
        let n = CellXY::new(e.x + dx, e.y + dy);
        if occupied(w, n, k) {
            // spec: S-0417 claim 4 (a neighbour listing such an occupant: forced, move-anywhere, E's centre as threat point)
            crate::scatter::cell_request(
                w,
                n,
                Layer::Ground,
                ScatterRequest {
                    threat: Some(threat),
                    forced: true,
                    anywhere: true,
                },
            );
        }
    }
    false
}

/// Starts K's door moving: `open` towards open, otherwise towards closed
/// (S-0417 F2, Terms *door*).
fn move_door(w: &mut World, k: ObjectId, open: bool) {
    let t = w.tick;
    let ticks = door_time(w, k);
    if let Some(b) = w.obj_mut(k) {
        // spec: S-0417 F2 (opening and closing take the door time; stationary at the timer update once it has elapsed)
        b.beh.gate.door = if open {
            Door::Opening { since: t, ticks }
        } else {
            Door::Closing { since: t, ticks }
        };
    }
}

/// A release queues *unload* on K without a start (S-0417 claims 1 and
/// 2): it becomes current at a per-tick start that finds the ready mark
/// set. With `quirks.factory_unload_waits_ready_mark` off, queueing it also
/// sets the mark.
fn queue_unload(w: &mut World, k: ObjectId) {
    let waits = w.rules().quirks.factory_unload_waits_ready_mark;
    if let Some(o) = w.obj_mut(k) {
        // spec: S-0417 claims 1 and 2 (unload queued at release, without a start; nothing in the release sets or clears the ready mark)
        o.beh.queue(Some(Behaviour::Unload));
        if !waits {
            // spec: S-0417 Edge cases (quirks.factory_unload_waits_ready_mark off: queueing unload at release also sets the ready mark)
            o.beh.ready_mark = true;
        }
    }
}

/// K falls back to rest at the end of its unloading: it queues *guard*
/// (S-0417 claims 2 and 9). The deferred request for the idle body state
/// leaves the running idle range unchanged (S-0108 claim 6: the same
/// state), as no production animation is modelled.
fn end_unloading(w: &mut World, k: ObjectId) {
    let waits = w.rules().quirks.factory_unload_waits_ready_mark;
    if let Some(b) = w.obj_mut(k) {
        // spec: S-0417 claim 2 and table S1 (closing: queue guard without a start; guard becomes current at the first per-tick start that finds the ready mark set, Q-0222)
        b.beh.queue(Some(Behaviour::Guard));
        if !waits {
            // spec: S-0417 Edge cases (quirks.factory_unload_waits_ready_mark off: the processing that queues guard also sets the ready mark)
            b.beh.ready_mark = true;
        }
    }
}

/// *Unload* of a factory building (S-0417 claims 2 and 9, table S1);
/// returns the delay.
pub(crate) fn process_unload(w: &mut World, k: ObjectId) -> i32 {
    let Some(b) = w.obj(k).cloned() else {
        return crate::process::standard_delay(w, k);
    };
    let naval = building_rules(w, k).is_some_and(|x| x.naval);
    let door = b.beh.gate.door;
    let set = |w: &mut World, s: i32| {
        if let Some(o) = w.obj_mut(k) {
            o.beh.progress = s;
        }
    };
    if naval {
        if b.beh.progress == stage::STARTING {
            // spec: S-0417 claim 9 (the first processing opens the door and advances directly to closing)
            move_door(w, k, true);
            set(w, stage::CLOSING);
        } else {
            // spec: S-0417 claim 9 (the second processing falls back to rest regardless of door state, queueing guard)
            end_unloading(w, k);
        }
        // spec: S-0417 claims 9 and 12 (each processing assigns one standard-delay draw)
        return crate::process::standard_delay(w, k);
    }
    match b.beh.progress {
        stage::STARTING => {
            let tick = w.tick;
            let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
            if let Some(u) = contacted(w, k)
                && let Some(o) = w.obj_mut(u)
            {
                // spec: S-0417 claim 2 (the contacted unit starts guard without the ordinary readiness test)
                o.beh.force(Behaviour::Guard, tick, stale);
            }
            // spec: S-0309 claim 31 (a door left open by an unloading that an owner change ended stays open until a later unloading reaches its closing stage: that unloading's letting-out stage finds it open and stationary and goes on without waiting for it)
            if door != Door::Open {
                // spec: S-0417 claim 2 (the door starts opening)
                move_door(w, k, true);
            }
            set(w, stage::APRON);
        }
        stage::APRON => {
            // spec: S-0417 claim 2 (apron clearing does not wait for the door; a blocked apron stays in this stage)
            if apron(w, k).is_none_or(|e| clear_apron(w, k, e)) {
                // spec: S-0417 table S1 (a free apron: letting out, no exit movement yet)
                set(w, stage::LETTING_OUT);
            }
        }
        stage::LETTING_OUT => {
            // spec: S-0417 claim 2 (letting out waits for an open, stationary door)
            if door == Door::Open {
                match contacted(w, k) {
                    None => {
                        // spec: S-0417 claim 2 (contact absent: K starts closing)
                        move_door(w, k, false);
                        set(w, stage::CLOSING);
                    }
                    Some(u) => {
                        if let Some(o) = w.obj_mut(u) {
                            // spec: S-0417 claim 2 (K queues move on u)
                            o.beh.queue(Some(Behaviour::Move));
                        }
                        send_out(w, k, u);
                        set(w, stage::WAITING);
                    }
                }
            }
        }
        stage::WAITING => {
            let held = contacted(w, k).is_some_and(|u| tethered(w, u));
            if !held {
                // spec: S-0417 claim 2 (the tether ended: K starts closing)
                move_door(w, k, false);
                set(w, stage::CLOSING);
            }
        }
        _ => {
            // spec: S-0417 claim 2 (closing waits for a stationary closed door)
            if door == Door::Closed {
                end_unloading(w, k);
            }
        }
    }
    // spec: S-0417 claims 2 and 12 and F1 (every processing assigns one standard-delay draw over 0 … 2, after the scatter draws)
    crate::process::standard_delay(w, k)
}

/// Sends u out of K (S-0417 claims 5 and 6).
fn send_out(w: &mut World, k: ObjectId, u: ObjectId) {
    use glasnost_rules::sim::Locomotor as L;
    let Some(o) = w.obj(u).cloned() else { return };
    let loco = w
        .rules()
        .unit(o.kind, o.type_index)
        .map_or(L::Teleport, |x| x.type_locomotor());
    let Some(e) = apron(w, k) else { return };
    if !w.cells.footprint().is_legal(e) {
        // spec: S-0417 Edge cases (an apron that is not a legal map cell: free; the exit instruction is refused, u stays where it appeared and reports clear at once, so K can close; claim 8's reply then applies; under both states of quirks.factory_apron_short_list)
        report(w, u);
        return;
    }
    match loco {
        // spec: S-0417 claim 5 (a drive vehicle is put on T1 towards E's centre, without a path search)
        L::Drive => crate::drive::begin_exit_track(w, u, e),
        // spec: S-0417 claim 5 (tunnelling and teleporting vehicles temporarily receive driving locomotion and use the same track)
        L::Tunnel | L::Teleport => {
            install_temporary_drive(w, u);
            crate::drive::begin_exit_track(w, u, e);
        }
        _ => {
            let c = w.obj(k).map_or(e, |b| b.pos.cell());
            // spec: S-0417 F4 ((x0 + 4, y0 + 1), integer cells with the same wrapping) and Edge cases (quirks.factory_fixed_hover_exit off: E)
            let dest = if w.rules().quirks.factory_fixed_hover_exit {
                CellXY::new(
                    i32::from(c.x.wrapping_add(4) as i16),
                    i32::from(c.y.wrapping_add(1) as i16),
                )
            } else {
                e
            };
            // spec: S-0417 claim 5 (any other locomotor gets the destination and moves by its own locomotor's rules; S-0207 claim 20 (a))
            super::assign_destination(w, u, Some(Target::Cell(dest)));
        }
    }
    if let Some(d) = w.obj_mut(u).and_then(|o| o.drive.as_mut()) {
        // spec: S-0417 claims 5 and 6 and Constants (after the exit instruction K sets the speed fraction to 0.5)
        d.rho = crate::consts::HALF;
    }
    // spec: S-0417 claims 5 and 6 (a hovering vehicle's speed fraction is set to 0.5 too); S-0503 claim 4 (it persists until a listed event)
    crate::hover::set_exit_fraction(w, u);
}

/// Gives a tunnelling or teleporting vehicle driving locomotion for its
/// factory exit (S-0417 claim 5, S-0207 claim 21); nothing when it already
/// drives. The mark of its actual cell passes to the driving state, which
/// clears it on departure; a pending reservation and, under the ground
/// reservation quirk, a separate remembered reservation stay marked.
pub fn install_temporary_drive(w: &mut World, u: ObjectId) {
    if w.obj(u).is_none_or(|o| o.drive.is_some()) {
        return;
    }
    if !crate::teleport::is_teleporter(w, u) {
        // A tunnelling vehicle: S-0207 claim 21 covers teleporting
        // locomotion only, so its cell claim is released as before.
        crate::drive::release_claims(w, u);
        if let Some(ob) = w.obj_mut(u) {
            ob.drive = Some(crate::drive::DriveState::new());
            ob.econ.temporary_drive = true;
        }
        return;
    }
    if let Some(t) = w.obj_mut(u).and_then(|o| o.teleport.as_mut()) {
        // spec: S-0207 claim 21 (our engine cancels a jump pending when temporary driving is installed over teleporting, as an empty assignment cancels one, claim 20 h: the landing point's reservation stays and no jump is pending at restoration; no quirk switch)
        t.landing = None;
        t.for_dest = None;
    }
    // Its own cell claim is handed to the driving state's claims.
    // spec: S-0207 claim 21 (the suspended teleporting state keeps its reservation; while driving, destinations do not move it)
    let pending = w
        .obj(u)
        .and_then(|o| o.teleport.as_ref())
        .is_some_and(|t| t.mark.is_some());
    let mut handed = Vec::new();
    if !pending && let Some(o) = w.obj(u) {
        let actual = (o.pos.cell(), o.layer);
        // spec: S-0207 claim 21 (installing temporary driving clears neither the remembered ground reservation nor the actual mark: the actual cell's mark passes to driving, which clears it on departure)
        for m in crate::teleport::removal_marks(w, u) {
            if m == actual {
                handed.push(m.0);
            } else {
                // spec: S-0207 Edge cases (a ground-reservation quirk off keeps no separate reservation)
                w.cells.claim_vehicle(m.0, m.1, false);
            }
        }
        if let Some(t) = w.obj_mut(u).and_then(|o| o.teleport.as_mut()) {
            // A separate remembered reservation is left behind, no longer held (claim 21).
            t.held = None;
            t.deck_mark = None;
        }
    }
    if let Some(ob) = w.obj_mut(u) {
        let mut d = crate::drive::DriveState::new();
        d.claims = handed;
        ob.drive = Some(d);
        ob.econ.temporary_drive = true;
    }
}

/// Whether a vehicle's own locomotor is temporarily replaced by driving
/// for its factory exit (S-0417 claim 5).
pub fn temporary_drive(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.econ.temporary_drive && o.drive.is_some())
}

/// At a fall-back to rest, a vehicle that drives temporarily gets its own
/// locomotor back (S-0215 claim 13 row 1, S-0417 claim 5). Returns
/// whether it did.
pub fn restore_locomotor(w: &mut World, id: ObjectId) -> bool {
    if !restorable(w, id, false) {
        return false;
    }
    restore(w, id)
}

/// Whether a temporarily driving vehicle's suspended predecessor can be
/// restored (S-0417 claim 5, S-0207 claim 27 table row 1): a suspended
/// predecessor, no driving movement report, no restoration lock and a
/// vehicle that is not seized. Completing the exit track alone is not
/// enough: a retained locomotor destination point still prevents it.
/// `assigning`: the test is made inside a destination assignment, before the
/// locomotor takes the offered destination.
pub fn restorable(w: &World, id: ObjectId, assigning: bool) -> bool {
    // spec: S-0207 claim 27 (the suspended predecessor is present: the vehicle drives temporarily)
    temporary_drive(w, id)
        // spec: S-0207 claim 27 and S-0607 claim 20 (not seized)
        && !crate::lift::seized(w, id)
        // spec: S-0207 Terms, Restoration lock (it starts absent on a new driving locomotor and nothing in the engine sets it, so it never holds)
        // spec: S-0417 claim 5 and S-0207 claim 27 (no driving movement report: no locomotor destination point and no heading point horizontally different from the actual position)
        && !crate::drive::movement_report(w, id, assigning)
}

/// Gives the vehicle its own locomotor back, the eligibility already
/// judged (S-0215 claim 13 row 1).
pub fn restore(w: &mut World, id: ObjectId) -> bool {
    if !temporary_drive(w, id) {
        return false;
    }
    crate::drive::release_claims(w, id);
    let Some(o) = w.obj_mut(id) else { return false };
    // spec: S-0215 claim 13 row 1 (it gets its own locomotor back)
    o.drive = None;
    o.econ.temporary_drive = false;
    // spec: S-0207 claim 21 (restoration makes no new, empty teleporting state: a fixed landing point's reservation and a phasing-in state with its original expiry are retained; no jump is pending, the install cancelled it)
    let pending = o.teleport.as_ref().is_some_and(|t| t.mark.is_some());
    let (c, l) = (o.pos.cell(), o.layer);
    // spec: S-0207 claim 21 (no occupancy mark is moved merely by restoration: a retained reservation stays where it is) and S-0209 claim 1 (otherwise a vehicle without the driving locomotor holds the vehicle-present flag of its cell)
    if !pending {
        w.cells.claim_vehicle(c, l, true);
    }
    true
}

/// The clear report of a tethered unit: its contact with K and the tether
/// end (S-0417 claims 7 and 10). Returns the factory.
fn report(w: &mut World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj_mut(id)?;
    let e = o.econ.exit.take()?;
    // spec: S-0417 claim 7 (the report ends contact and the tether)
    if o.contact == Some(e.factory) {
        o.contact = None;
    }
    Some(e.factory)
}

/// The clear report of a released jumpjet unit climbing away from its
/// factory (S-0417 claim 15, S-0502 claim 7): contact and tether end in the
/// same update, then the saved guard spot becomes the destination or the
/// order-level destination is discarded and a forced scatter request is made.
/// The caller tests the height window.
pub fn ascent_report(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0417 claim 15 (an infantry unit or vehicle using jumpjet locomotion)
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) || !crate::jump::is_jumpjet(w, id) {
        return;
    }
    // spec: S-0417 claim 15 (its type is marked as jumpjet or balloon-hover; the importer keys JumpJet and BalloonHover)
    let flagged = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.jumpjet || u.raw.balloon_hover.unwrap_or(false));
    if !flagged {
        return;
    }
    let Some(e) = o.econ.exit else { return };
    // spec: S-0417 claim 15 (still in contact with a vehicle factory or a building with one of the barracks door kinds)
    if o.contact != Some(e.factory) {
        return;
    }
    let door = building_rules(w, e.factory).is_some_and(|b| {
        let x = &b.raw.econ;
        x.door_first == Some(true) || x.door_second == Some(true) || x.door_third == Some(true)
    });
    if !(vehicle_factory(w, e.factory) || door) {
        return;
    }
    // spec: S-0417 claim 15 (both factory contact and the tether end in this unit update; no foundation, distance or arrival test)
    report(w, id);
    if let Some(gs) = o.guard_spot
        && o.dest != Some(gs)
    {
        // spec: S-0417 claim 15 (a saved guard spot different from the order-level destination becomes the destination)
        super::assign_destination(w, id, Some(gs));
        return;
    }
    // spec: S-0417 claim 15 (otherwise the order-level destination is discarded without stopping the jumpjet locomotion: the movement point and commitment stay)
    crate::jump::discard_order_destination(w, id);
    // spec: S-0417 claim 15 (then a forced scatter request without a threat point, not move-anywhere; S-0207 claim 16, S-0208 claims 11 to 13)
    crate::scatter::request(
        w,
        id,
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

/// Whether cell `c` holds a building (S-0417 claims 7 and 8).
fn building_cell(w: &World, c: CellXY) -> bool {
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
}

/// A unit's cell-arrival update (S-0417 claims 7, 8 and 10): the clear
/// report of a tethered unit, a vehicle factory's run-away reply and the
/// arrival fallback. A driving vehicle's arrival is reaching a track
/// target, also when a further track follows in the same tick (S-0417
/// claim 7, Q-0223; the chain-point update of a chaining vehicle never
/// occurs, as curve chaining, S-0207 claim 8, is not modelled); a walking
/// unit's is its arrival tick at each walking target (S-0208 claim 23).
pub fn arrival(w: &mut World, id: ObjectId) {
    if !tethered(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let here = o.pos.cell();
    if o.kind == Kind::Infantry {
        // spec: S-0417 claim 10 and S-0208 claim 23 (a tethered infantry unit reports clear on its first standing-position arrival; no run-away reply)
        report(w, id);
        return;
    }
    let k = o.econ.exit.map(|e| e.factory);
    let cur = o.beh.seen();
    // spec: S-0417 claim 7 (no report in unload, or in enter towards its contacted building)
    let enter_k =
        cur == Some(Behaviour::Enter) && k.is_some_and(|k| o.dest == Some(Target::Object(k)));
    // spec: S-0417 claim 7 (no report with a destination other than its present cell while its cell holds a building; an object destination is not its own cell)
    let elsewhere = o.dest.is_some_and(|d| d != Target::Cell(here));
    let held = cur == Some(Behaviour::Unload) || enter_k || (elsewhere && building_cell(w, here));
    if !held
        && let Some(k) = report(w, id)
        && vehicle_factory(w, k)
    {
        run_away(w, id);
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0417 claim 8 (the arrival fallback: still on a building cell with no destination, no queued destinations and no stored path)
    if building_cell(w, o.pos.cell())
        && o.dest.is_none()
        && o.queued_dests.is_empty()
        && o.path.steps.is_empty()
    {
        // spec: S-0417 claim 8 (a forced, move-anywhere scatter request without a threat point)
        crate::scatter::request(
            w,
            id,
            ScatterRequest {
                threat: None,
                forced: true,
                anywhere: true,
            },
        );
    }
}

/// A vehicle factory's run-away reply to a clear report (S-0417 claim 8).
pub(crate) fn run_away(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let here = o.pos.cell();
    // spec: S-0417 claim 8 (a destination other than u's current cell is kept)
    if o.dest.is_some_and(|d| d != Target::Cell(here)) {
        return;
    }
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let tick = w.tick;
    if crate::econ::harvest::is_harvester(w, id) {
        let ready = crate::process::ready(w, id);
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0417 claim 8 (an ore-gathering vehicle queues harvest with a readiness-dependent immediate start)
            ob.beh
                .queue_and_start(Some(Behaviour::Harvest), ready, tick, stale);
        }
        return;
    }
    // spec: S-0417 claim 8 (in our engine every player house uses the human rows)
    let human = o.owner.is_some_and(|h| w.houses[usize::from(h.0)].human);
    if !human {
        // The original's non-human rule for the neutral and special houses is not established (S-0417 claim 8).
        w.note("S-0417 claim 8: the run-away reply of a neutral or special house's vehicle is not established; it gives none");
        return;
    }
    if let Some(gs) = o.guard_spot
        && o.dest != Some(gs)
    {
        // spec: S-0417 claim 8 (the saved guard spot becomes the destination, without queueing a new move; S-0207 claim 20 (a))
        super::assign_destination(w, id, Some(gs));
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0417 claim 8 (otherwise it discards its stored path)
        ob.path.steps.clear();
    }
    // spec: S-0417 claim 8 (a forced scatter request without a threat point, not move-anywhere)
    crate::scatter::request(
        w,
        id,
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

/// The vehicle-factory conditions of a vehicle's readiness (S-0215 claim
/// 11): true when they hold it back.
pub fn holds_back(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    match o.contact {
        // spec: S-0215 claim 11 (in contact with a vehicle-factory building and the queued behaviour is neither move nor enter)
        Some(c) => {
            vehicle_factory(w, c)
                && !matches!(o.beh.queued, Some(Behaviour::Move | Behaviour::Enter))
        }
        // spec: S-0215 claim 11 (in contact with nothing, on cell (x, y + 1) of a vehicle-factory building)
        None => {
            let here = o.pos.cell();
            w.cells.list(here, Layer::Ground).iter().any(|&b| {
                vehicle_factory(w, b)
                    && w.obj(b).is_some_and(|f| {
                        let c = f.pos.cell();
                        here == CellXY::new(c.x, c.y + 1)
                    })
            })
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
