//! The movement trace (M8 A4): a read-only record of what the movement code
//! decided and why, for the pathing-report tool (`glasnost-headless
//! path-report`). It records path computations with their cell searches,
//! step checks that did not let a unit step, blocked-step re-paths, give-ups,
//! give-way (scatter) requests and their answers, and group-destination
//! spreading.
//!
//! The trace is not game state. It is off unless a caller switches it on
//! with [`switch_on`]; it is never saved, never hashed, never cloned with
//! its world, and the simulation never reads it. Recording only copies
//! values the movement code already has in hand, plus pure reads of the
//! cell lists, so a traced run and an untraced run take the same steps and
//! give the same state hash after every tick. It holds integers, cells,
//! object ids and outcomes only: no floats (a search cost is kept as the
//! bits of the simulation's own single-precision value), no hash
//! containers, no I/O.
//!
//! Our own diagnostics; no spec applies.

use std::cell::RefCell;

use glasnost_map::CellXY;

use crate::occupancy::Outcome;
use crate::path::{Mode, Search};
use crate::world::{ObjectId, World};

/// Which movement code made a step check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mover {
    /// The driving (or ship) locomotor.
    Drive,
    /// The walking locomotor.
    Walk,
    /// The hovering locomotor.
    Hover,
}

/// One cell search inside a path computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellSearch {
    /// Whether a corridor confined it.
    pub corridor: bool,
    /// The pairs expanded.
    pub expansions: u32,
    /// The pairs recorded besides the start.
    pub recorded: u32,
    /// The steps found, if it found a path (before flattening and
    /// shortening).
    pub found: Option<u32>,
    /// The cost g of the end pair it reached, as the bits of the
    /// simulation's single-precision value, if it found a path.
    pub cost_bits: Option<u32>,
}

/// What happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum What {
    /// A path computation (S-0212): its mode as asked and as run, the
    /// start, the destination cell before and after substitution, the
    /// cells carrying a congestion mark, whether a corridor was used, its
    /// cell searches, the result and the stored steps.
    Search {
        /// The mode asked for.
        asked: Mode,
        /// The mode the search ran in (a blocked re-path without
        /// congestion marks runs fresh).
        mode: Option<Mode>,
        /// The start cell.
        from: CellXY,
        /// The destination cell before substitution.
        goal: Option<CellXY>,
        /// The destination cell searched for (after substitution).
        to: Option<CellXY>,
        /// The cells with a congestion mark.
        congested: Vec<CellXY>,
        /// Whether the corridor was used.
        corridor: Option<bool>,
        /// The cell searches made, in order.
        searches: Vec<CellSearch>,
        /// The result.
        result: Search,
        /// The stored steps (directions 0 to 7), when a path was stored.
        steps: Vec<u8>,
    },
    /// What followed a failed path computation (S-0212 claims 21, 23).
    Failed {
        /// The result that failed.
        result: Search,
        /// Whether the unit gave up (its destination cleared and a guard
        /// behaviour queued).
        gave_up: bool,
        /// Whether its destination is gone afterwards.
        dest_cleared: bool,
        /// The retry tick set.
        retry_at: i32,
    },
    /// A step check whose outcome was not clear: the unit did not step
    /// into `to` on this check.
    Step {
        /// The movement code.
        mover: Mover,
        /// The unit's cell.
        from: CellXY,
        /// The cell asked for.
        to: CellXY,
        /// The entry outcome.
        outcome: Outcome,
        /// A repeated check (after a search on the same tick), or the
        /// second cell of a curve.
        repeated: bool,
        /// The objects listed in the asked cell's layer, in listing order
        /// (at most 8).
        blockers: Vec<ObjectId>,
    },
    /// A blocked step by a moving blocker (S-0212 claim 22, S-0209 claim
    /// 10): the blockage start and whether the retry timer held the
    /// re-path back.
    Blocked {
        /// The tick the blockage began.
        since: i32,
        /// The retry tick, when the timer held the re-path back.
        waiting_until: Option<i32>,
    },
    /// The unit waited without a step check, and why.
    Waited {
        /// Why, in our own words.
        why: &'static str,
    },
    /// The unit gave up its move (S-0207 Terms, S-0209 claim 8).
    GaveUp,
    /// The unit asked the occupants of a cell to make room (a scatter
    /// request, S-0207 claim 15, S-0209 claim 8).
    AskedRoom {
        /// The cell.
        cell: CellXY,
    },
    /// The unit was asked to make room (S-0207 claim 16, S-0208 claim
    /// 11).
    Asked {
        /// Whether it accepted.
        accepted: bool,
        /// Its destination afterwards.
        dest: Option<CellXY>,
    },
    /// The unit gives way in a deadlock (our design, C7 part 5, yield).
    GaveWay {
        /// The free cell it goes to.
        refuge: CellXY,
        /// The unit with the right of way.
        to: Option<ObjectId>,
    },
    /// The unit resumes the order it gave way from.
    Resumed,
    /// The unit had the extra locomotion of an accepted scatter (S-0208
    /// claim 6, r46).
    ExtraLocomotion {
        /// The simulation draws it took.
        draws: u64,
        /// Its walking target afterwards.
        target: Option<CellXY>,
    },
    /// A group order spread the unit's destination (S-0213 claim 7).
    Spread {
        /// The common destination.
        common: CellXY,
        /// The unit's own cell.
        cell: CellXY,
    },
}

/// One event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// The tick.
    pub tick: i32,
    /// The unit.
    pub unit: ObjectId,
    /// What happened.
    pub what: What,
}

/// The record.
#[derive(Debug, Default)]
pub struct Trace {
    /// The events, in the order they happened.
    pub events: Vec<Event>,
    /// What the path computation in progress has noted.
    pending: Pending,
}

/// What a path computation in progress has noted so far.
#[derive(Debug, Default)]
pub(crate) struct Pending {
    /// The destination cell before substitution.
    pub goal: Option<CellXY>,
    /// The mode the search runs in.
    pub mode: Option<Mode>,
    /// The cells with a congestion mark.
    pub congested: Vec<CellXY>,
    /// Whether the corridor is used.
    pub corridor: Option<bool>,
    /// The cell searches made.
    pub searches: Vec<CellSearch>,
}

/// The world's slot for a trace: empty unless switched on. A clone of a
/// world has no trace (the trace belongs to the run it was switched on
/// for), and the slot is skipped by serialization.
#[derive(Debug, Default)]
pub struct Slot(Option<Box<RefCell<Trace>>>);

impl Clone for Slot {
    fn clone(&self) -> Self {
        Slot(None)
    }
}

/// Switches the trace on for `w` (a fresh, empty record).
pub fn switch_on(w: &mut World) {
    w.trace = Slot(Some(Box::default()));
}

/// Takes the events recorded so far, leaving the trace on.
pub fn take(w: &World) -> Vec<Event> {
    match &w.trace.0 {
        Some(t) => std::mem::take(&mut t.borrow_mut().events),
        None => Vec::new(),
    }
}

/// Whether the trace is on.
pub fn on(w: &World) -> bool {
    w.trace.0.is_some()
}

/// Records an event for `unit`; `what` runs only when the trace is on.
pub(crate) fn record(w: &World, unit: ObjectId, what: impl FnOnce() -> What) {
    if let Some(t) = &w.trace.0 {
        let what = what();
        t.borrow_mut().events.push(Event {
            tick: w.tick,
            unit,
            what,
        });
    }
}

/// Notes something about the path computation in progress.
pub(crate) fn note(w: &World, f: impl FnOnce(&mut Pending)) {
    if let Some(t) = &w.trace.0 {
        f(&mut t.borrow_mut().pending);
    }
}

/// Notes a cell search of the path computation in progress.
pub(crate) fn cell_search(w: &World, s: impl FnOnce() -> CellSearch) {
    if let Some(t) = &w.trace.0 {
        let s = s();
        t.borrow_mut().pending.searches.push(s);
    }
}

/// Starts a path computation: forgets anything noted outside one.
pub(crate) fn begin_search(w: &World) {
    if let Some(t) = &w.trace.0 {
        t.borrow_mut().pending = Pending::default();
    }
}

/// What the path computation noted since [`begin_search`].
pub(crate) fn pending(w: &World) -> Pending {
    match &w.trace.0 {
        Some(t) => std::mem::take(&mut t.borrow_mut().pending),
        None => Pending::default(),
    }
}

/// The objects listed in cell `c`'s layer for a unit at height level
/// `reference` (a pure read), at most 8.
pub(crate) fn listed(w: &World, c: CellXY, reference: i32) -> Vec<ObjectId> {
    let layer = crate::occupancy::layers_for(w, c, reference).0;
    w.cells.list(c, layer).iter().copied().take(8).collect() // nocite: our own diagnostics (a short list)
}

/// Records a step check that did not give clear.
pub(crate) fn step(
    w: &World,
    unit: ObjectId,
    mover: Mover,
    from: CellXY,
    to: CellXY,
    outcome: Outcome,
    repeated: bool,
) {
    if outcome == Outcome::Clear {
        return;
    }
    record(w, unit, || {
        let reference = crate::occupancy::reference_level(w, unit);
        What::Step {
            mover,
            from,
            to,
            outcome,
            repeated,
            blockers: listed(w, to, reference),
        }
    });
}
