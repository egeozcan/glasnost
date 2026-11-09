//! Claim tests of S-0212 revision 97: claim 21's running retry delay ends
//! for the driving and walk locomotors when the destination is changed to
//! a different one or cleared, a delay started earlier in the same tick
//! included, so no delay remains after a failure that clears the
//! destination (claim 23) and a new destination is searched at once. The
//! ship locomotor keeps the delay; the same destination given again keeps
//! it; a harvester's docking offer keeps it (S-0402 claim 10a). Synthetic
//! data only: the test kit's invented walker and buggy, a ship of our own,
//! our own water cells, no game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::occupancy::in_progress;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, SIZE, mid};
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// The combat ruleset with the path-retry delay field at 0.1 game minutes,
/// R = 90 ticks (S-0212 claim 21 and P25 a), and a ship `BOAT` of our own.
fn ruleset_r90() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.path_delay = Some(Decimal("0.1".into()));
    r.sim.vehicles.insert(
        "BOAT".into(),
        UnitType {
            strength: Some(300),
            speed: Some(5),
            locomotion_class: Some(LocomotionClass::Float),
            locomotor: Some(Locomotor::Ship),
            ..Default::default()
        },
    );
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("BOAT".into());
    r
}

/// A game on the test kit's empty map with one unit of type `name` of house
/// 0 at the map's middle, orders open (S-0201 claim 14), the trace on.
fn one(kind: Kind, name: &str) -> (World, ObjectId) {
    let mut w = testkit::game_with(&ruleset_r90(), SIZE, 1);
    let id = testkit::spawn(&mut w, kind, name, 0, mid(SIZE, 0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    trace::switch_on(&mut w);
    (w, id)
}

/// Makes `c` an island: its eight neighbours become water.
fn island(w: &mut World, c: CellXY) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                let n = CellXY::new(c.x + dx, c.y + dy);
                w.cells.grid.get_mut(n).unwrap().land = land::WATER;
                glasnost_sim::regions::cell_changed(w, n);
            }
        }
    }
}

/// A move order to cell `c`.
fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    assert!(orders::issue(w, HouseId(0), o), "move order refused");
}

/// Gives `id` destination `d` where it reaches the locomotor, as an order
/// or a team script's move would (S-0107 claim 15's hook).
fn give(w: &mut World, id: ObjectId, d: Option<Target>) {
    let before = w.obj(id).unwrap().dest;
    let o = w.obj_mut(id).unwrap();
    o.dest = d;
    o.path.steps.clear();
    glasnost_sim::infantry::destination_assigned(w, id, before);
}

/// The ticks of `id`'s path computations and their results.
fn searches(events: &[trace::Event], id: ObjectId) -> Vec<(i32, Search)> {
    events
        .iter()
        .filter(|e| e.unit == id)
        .filter_map(|e| match e.what {
            What::Search { result, .. } => Some((e.tick, result)),
            _ => None,
        })
        .collect()
}

/// Steps up to `n` ticks until `id` makes a path computation; returns its
/// tick and result.
fn next_search(w: &mut World, id: ObjectId, n: i32) -> Option<(i32, Search)> {
    for _ in 0..n {
        tick::step(w);
        if let Some(&x) = searches(&trace::take(w), id).first() {
            return Some(x);
        }
    }
    None
}

/// A walker one cell from `near`, whose fresh search for it failed at tick
/// `t` and kept the destination (S-0212 claim 23, row 3).
fn walker_kept(w: &mut World, g: ObjectId) -> (i32, CellXY) {
    let c = w.obj(g).unwrap().pos.cell();
    let near = CellXY::new(c.x + 1, c.y);
    give(w, g, Some(Target::Cell(near)));
    let t = w.tick;
    path::failed(w, g, Search::SearchFailed, false);
    trace::take(w);
    (t, near)
}

// spec: S-0212 claim 21 (r97: for the walk locomotor the clearing of the gate row ends the delay its fresh search started, so no delay remains and the first search for a new destination is not held back)
// spec: S-0212 claim 23 (gate row: the walk locomotor's destination cleared after a fresh search)
#[test]
fn claim_21_walker_gate_failure_leaves_no_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let isle = mid(SIZE, 8, 0);
    island(&mut w, isle);
    move_to(&mut w, g, isle);
    let (t, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::GateFailed);
    assert_eq!(w.obj(g).unwrap().dest, None);
    assert!(!path::delay_runs(&w, g), "no delay remains");
    assert!(w.obj(g).unwrap().path.retry_at <= w.tick);
    let open = mid(SIZE, -6, 0);
    move_to(&mut w, g, open);
    let (s, r) = next_search(&mut w, g, 5).expect("a search for the new destination");
    assert_eq!(r, Search::Found);
    assert!(s < t + 5, "searched at once, not at {}", t + 90);
}

// spec: S-0212 claim 21 (r97: a walker that keeps its destination after a failed search, claim 23 row 3, waits for R before searching that destination again)
#[test]
fn claim_21_walker_kept_destination_waits_for_r() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let (t, near) = walker_kept(&mut w, g);
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(near)), "kept within 1 cell");
    assert_eq!(o.path.retry_at, t + 90);
    assert!(path::delay_runs(&w, g));
    assert_eq!(
        next_search(&mut w, g, 120),
        Some((t + 90, Search::Found)),
        "the kept destination is searched once R is over"
    );
}

// spec: S-0212 claim 21 (r97: a destination changed to a different one ends the running delay of a walker that kept its destination; the first search for it is not held back)
#[test]
fn claim_21_walker_different_destination_ends_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let (t, _) = walker_kept(&mut w, g);
    for _ in 0..20 {
        tick::step(&mut w);
    }
    assert!(searches(&trace::take(&w), g).is_empty(), "waiting");
    let open = mid(SIZE, -6, 0);
    give(&mut w, g, Some(Target::Cell(open)));
    assert!(!path::delay_runs(&w, g), "ended at the change");
    assert!(in_progress(&w, g));
    let (s, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::Found);
    assert!(s < t + 30, "not held back to {}", t + 90);
}

// spec: S-0212 claim 21 and Open questions (for every locomotor, giving a unit the destination it already holds keeps the delay running: this spec keeps it)
// design: Q-2896 (interim for the walk locomotor: S-0209 r76 claim 10 ends the delay at this assignment, S-0212 r97 keeps it; the S-0212 text is followed until the request is answered)
#[test]
fn claim_21_the_same_destination_again_keeps_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let (t, near) = walker_kept(&mut w, g);
    tick::step(&mut w);
    give(&mut w, g, Some(Target::Cell(near)));
    assert!(path::delay_runs(&w, g));
    assert_eq!(w.obj(g).unwrap().path.retry_at, t + 90);
    assert_eq!(next_search(&mut w, g, 120), Some((t + 90, Search::Found)));
}

// spec: S-0212 claim 21 (r97: a destination cleared ends a running delay of the walk locomotor, so the cell it kept, ordered again, is a different destination from none and is searched at once)
#[test]
fn claim_21_a_stop_ends_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let (t, near) = walker_kept(&mut w, g);
    tick::step(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: g }));
    tick::step(&mut w);
    assert_eq!(w.obj(g).unwrap().dest, None);
    assert!(!path::delay_runs(&w, g), "ended by the clearing");
    trace::take(&w);
    move_to(&mut w, g, near);
    let (s, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::Found);
    assert!(s < t + 10, "not held back to {}", t + 90);
}

// spec: S-0212 claim 21 (r97: for the driving locomotor no delay remains after a failed gate, claim 23's gate row, or a row 2 failure, both of which clear the destination)
// spec: S-0212 state table, row "destination changes" (for the driving locomotor a change to a different destination ends a running delay)
#[test]
fn claim_21_drive_failures_that_clear_leave_no_delay() {
    let (mut w, v) = one(Kind::Vehicle, "BUGGY");
    let isle = mid(SIZE, 8, 0);
    island(&mut w, isle);
    move_to(&mut w, v, isle);
    let (failed_at, r) = next_search(&mut w, v, 5).expect("a search");
    assert_eq!(r, Search::GateFailed);
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert!(!path::delay_runs(&w, v));
    move_to(&mut w, v, mid(SIZE, -6, 0));
    let (s, r) = next_search(&mut w, v, 120).expect("a search");
    assert_eq!(r, Search::Found);
    assert!(
        s < failed_at + 5,
        "searched at once, not at {}",
        failed_at + 90
    );

    // Row 2: a destination beyond 1 cell whose search finds no path.
    let (mut w, v) = one(Kind::Vehicle, "BUGGY");
    let c = w.obj(v).unwrap().pos.cell();
    give(&mut w, v, Some(Target::Cell(CellXY::new(c.x + 5, c.y))));
    path::failed(&mut w, v, Search::SearchFailed, false);
    assert_eq!(w.obj(v).unwrap().dest, None, "row 2 clears");
    assert!(!path::delay_runs(&w, v), "no delay remains");
}

// spec: S-0212 claim 22 (a re-path for a blocked step starts R again) and claim 21 (r97: a new, different destination ends it for the driving locomotor)
#[test]
fn claim_21_drive_repath_delay_ended_by_a_new_order() {
    let (mut w, v) = one(Kind::Vehicle, "BUGGY");
    let a = mid(SIZE, 6, 0);
    give(&mut w, v, Some(Target::Cell(a)));
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(v).unwrap(), t, r);
    assert!(path::delay_runs(&w, v), "the re-path's delay runs");
    trace::take(&w);
    let b = mid(SIZE, -6, 0);
    move_to(&mut w, v, b);
    let (s, res) = next_search(&mut w, v, 5).expect("a search");
    assert_eq!(res, Search::Found);
    assert!(s < t + 5);
}

// spec: S-0212 claim 21 and Open questions (the ship locomotor keeps a running delay when its destination changes or is cleared, low, unmeasured)
#[test]
fn claim_21_ship_keeps_its_delay() {
    let (mut w, s) = one(Kind::Vehicle, "BOAT");
    assert_eq!(
        glasnost_sim::lift::current_locomotor(&w, s),
        Some(glasnost_sim::rules::Locomotor::Ship)
    );
    give(&mut w, s, Some(Target::Cell(mid(SIZE, 6, 0))));
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(s).unwrap(), t, r);
    give(&mut w, s, Some(Target::Cell(mid(SIZE, -6, 0))));
    assert!(path::delay_runs(&w, s), "a different destination");
    give(&mut w, s, None);
    assert!(path::delay_runs(&w, s), "cleared");
    assert!(path::waits(&mut w, s), "its searches wait");
    assert_eq!(w.obj(s).unwrap().path.retry_at, t + r);
}

// spec: S-0402 claim 10a (an offer of the docking cell leads to a path computation that waits for a running retry delay like any fresh search) and S-0212 claim 21 (the docking offers are left to that claim)
#[test]
fn docking_offer_keeps_the_delay() {
    let (mut w, v) = one(Kind::Vehicle, "BUGGY");
    give(&mut w, v, Some(Target::Cell(mid(SIZE, 6, 0))));
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(v).unwrap(), t, r);
    let held = path::running_delay(&w, v);
    assert_eq!(held, Some(t + r));
    let dock = Some(Target::Cell(mid(SIZE, -6, 0)));
    give(&mut w, v, dock);
    assert!(!path::delay_runs(&w, v), "an ordinary change ends it");
    path::keep_delay(&mut w, v, held);
    assert!(path::delay_runs(&w, v), "the offer keeps it");
    assert_eq!(w.obj(v).unwrap().path.retry_at, t + r);
    assert_eq!(w.obj(v).unwrap().path.retry_for, dock);
}
