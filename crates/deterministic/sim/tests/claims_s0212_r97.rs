//! Claim tests of S-0212 revision 97: claim 21's running retry delay ends
//! for the driving and walk locomotors when the destination is changed to
//! a different one or cleared, a delay started earlier in the same tick
//! included, so no delay remains after a failure that clears the
//! destination (claim 23) and a new destination is searched at once. The
//! ship locomotor keeps the delay; a harvester's docking offer keeps it
//! (S-0402 claim 10a). Revision 98 (Q-2896, claim 21 and scenario P25 e):
//! for the walk locomotor the destination it already holds, given again,
//! ends the delay too; the driving and ship locomotors keep it then.
//! Synthetic data only: the test kit's invented walker and buggy, a ship of
//! our own, our own water cells, no game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LandRow, LandType, LocomotionClass, Locomotor, UnitType};
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

/// [`one`]'s walker with a close-enough distance of half a cell, below the
/// distance of 256 to a neighbouring cell's centre, so that [`walker_kept`]'s
/// failed search keeps its destination.
// spec: S-0208 claim 25 (r75, Every kind of destination: a cell destination is cleared at a failed search when D is less than the close-enough distance; Changes for S-0212 P25: a test close-enough distance below D)
fn one_near() -> (World, ObjectId) {
    let mut r = ruleset_r90();
    r.sim.general.close_enough = Some(Decimal("0.5".into()));
    let mut w = testkit::game_with(&r, SIZE, 1);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    trace::switch_on(&mut w);
    (w, id)
}

/// A walker one cell from `near`, whose fresh search for it failed at tick
/// `t` and kept the destination (S-0212 claim 23, row 3); use [`one_near`]'s
/// walker.
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

// spec: S-0212 claim 21 (r97: a walker that keeps its destination after a failed search, claim 23 row 3, has a retry delay of R running)
// spec: S-0208 claim 25 (r75, After a failed search, and Changes for S-0212 claim 21: such a walker has no locomotor destination and makes no search when R runs out; in a behaviour other than move nothing assigns it again)
#[test]
fn claim_21_walker_kept_destination_waits_for_r() {
    let (mut w, g) = one_near();
    let (t, near) = walker_kept(&mut w, g);
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(near)), "kept within 1 cell");
    assert_ne!(
        o.beh.seen(),
        Some(Behaviour::Move),
        "void unless not in move"
    );
    assert_eq!(o.path.retry_at, t + 90);
    assert!(path::delay_runs(&w, g));
    assert_eq!(next_search(&mut w, g, 120), None, "no search after R");
    assert_eq!(
        w.obj(g).unwrap().dest,
        Some(Target::Cell(near)),
        "still kept"
    );
}

// spec: S-0212 claim 21 (r97: a destination changed to a different one ends the running delay of a walker that kept its destination; the first search for it is not held back)
#[test]
fn claim_21_walker_different_destination_ends_the_delay() {
    let (mut w, g) = one_near();
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

// spec: S-0212 claim 21 (r98, Q-2896: for the walk locomotor the destination it already holds, the same cell, given again by another rule in a later tick ends the running delay, so it is searched for at once)
#[test]
fn claim_21_r98_the_same_destination_again_ends_the_delay() {
    let (mut w, g) = one_near();
    let (t, near) = walker_kept(&mut w, g);
    tick::step(&mut w);
    give(&mut w, g, Some(Target::Cell(near)));
    assert!(!path::delay_runs(&w, g), "ended at the assignment");
    assert!(w.obj(g).unwrap().path.retry_at <= w.tick);
    let (s, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::Found);
    assert!(s < t + 10, "not held back to {}", t + 90);
}

// spec: S-0212 claim 21 (r98: the walker's delay ends at a destination changed to a different one or cleared, or at the same cell or the same object given again; no destination given to a walker that has none is none of these, so the delay runs on)
#[test]
fn seams7b_nothing_given_to_a_walker_with_no_destination_keeps_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    assert_eq!(w.obj(g).unwrap().dest, None);
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(g).unwrap(), t, r);
    give(&mut w, g, None);
    assert!(path::delay_runs(&w, g), "the delay runs on");
    assert_eq!(w.obj(g).unwrap().path.retry_at, t + r);
    tick::step(&mut w);
    give(&mut w, g, None);
    assert!(path::delay_runs(&w, g), "a later tick too");
    assert_eq!(w.obj(g).unwrap().path.retry_at, t + r);
}

// spec: S-0212 claim 21 (r98, Q-2896: the same object given again ends the walker's running delay, a delay started earlier in the same tick included)
#[test]
fn claim_21_r98_the_same_object_again_ends_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, 6, 0), 0);
    give(&mut w, g, Some(Target::Object(v)));
    let t = w.tick;
    let r = w.rules().general.retry_delay;
    path::start_delay(w.obj_mut(g).unwrap(), t, r);
    assert!(path::delay_runs(&w, g));
    give(&mut w, g, Some(Target::Object(v)));
    assert!(!path::delay_runs(&w, g), "ended in the same tick");
    assert_eq!(w.obj(g).unwrap().path.retry_at, t);
}

// spec: S-0212 claim 21 and Open questions (r98: for every locomotor other than the walk locomotor, the destination a unit already holds, given again, keeps a running delay running: the driving and ship locomotors here)
#[test]
fn claim_21_r98_others_keep_the_delay_at_the_same_destination() {
    for name in ["BUGGY", "BOAT"] {
        let (mut w, v) = one(Kind::Vehicle, name);
        let d = Some(Target::Cell(mid(SIZE, 6, 0)));
        give(&mut w, v, d);
        let t = w.tick;
        let r = w.rules().general.retry_delay;
        path::start_delay(w.obj_mut(v).unwrap(), t, r);
        give(&mut w, v, d);
        assert!(path::delay_runs(&w, v), "{name}: the delay runs on");
        assert_eq!(w.obj(v).unwrap().path.retry_at, t + r, "{name}");
        tick::step(&mut w);
        give(&mut w, v, d);
        assert!(path::delay_runs(&w, v), "{name}: a later tick too");
        assert_eq!(w.obj(v).unwrap().path.retry_at, t + r, "{name}");
    }
}

/// Scenario P25(e) on [`walker_kept`]'s walker: a move order in tick
/// *c* + 10 to `to`. Returns *c*, the end-of-tick readings of the
/// order's tick (whether a delay runs, its tick) and the tick of the
/// walker's first path computation after *c*, read up to *c* + 100.
fn p25_e(to: impl Fn(CellXY) -> CellXY) -> (i32, (bool, i32), Option<i32>) {
    let (mut w, g) = one_near();
    let (c, near) = walker_kept(&mut w, g);
    assert_eq!(w.obj(g).unwrap().path.retry_at, c + 90);
    let mut first = None;
    let mut at_order = None;
    while w.tick <= c + 100 {
        let k = w.tick;
        if k == c + 10 {
            move_to(&mut w, g, to(near));
        }
        tick::step(&mut w);
        if first.is_none() {
            first = searches(&trace::take(&w), g).first().map(|s| s.0);
        } else {
            trace::take(&w);
        }
        if k < c + 10 {
            // Our walker's kept cell is open, so it walks once it searches;
            // up to the order it stays, as P25(e) requires.
            assert_eq!(w.obj(g).unwrap().pos.cell(), mid(SIZE, 0, 0), "it stays");
        }
        if k == c + 10 {
            at_order = Some((path::delay_runs(&w, g), w.obj(g).unwrap().path.retry_at));
        }
        if first.is_some() && at_order.is_some() {
            break;
        }
    }
    (c, at_order.expect("the order's tick"), first)
}

/// The P25(e) expectation for one run: at the end of the order's tick the
/// timer has run out or reads R from a computation in that tick, and the
/// first computation after *c* comes before *c* + 90, at the order or later.
fn p25_e_expect(run: &str, (c, (runs, at), first): (i32, (bool, i32), Option<i32>)) {
    let s = first.unwrap_or_else(|| panic!("{run}: no computation"));
    assert!(s >= c + 10, "{run}: none before the order: {s}");
    assert!(s < c + 90, "{run}: not held back to {}: {s}", c + 90);
    assert!(
        !runs || (s == c + 10 && at == s + 90),
        "{run}: the timer still counts from {c}: {at}"
    );
}

// spec: S-0212 claim 21 (r98, Q-2896: a walker's running retry delay ends when it is given the destination it already holds, by an order in a later tick than the search that started it) and scenario P25 part e run 1
#[test]
fn p25_e_run_1_an_order_to_the_cell_it_holds_ends_the_delay() {
    p25_e_expect("run 1", p25_e(|near| near));
}

// spec: S-0212 claim 21 (a destination changed to a different cell ends a walker's running retry delay) and scenario P25 part e run 2
#[test]
fn p25_e_run_2_an_order_to_the_second_tile_ends_the_delay() {
    p25_e_expect("run 2", p25_e(|near| CellXY::new(near.x, near.y + 1)));
}

/// What [`p25_e_real`] returns: *c*, the end-of-tick readings of the order's
/// tick (whether a delay runs, its tick), the ticks after *c* up to *c* +
/// 100 with the results of the walker's computations in each, and whether a
/// delay ran at the end of tick *c*.
struct RealRun {
    c: i32,
    at_order: (bool, i32),
    ticks: Vec<(i32, Vec<Search>)>,
    runs_at_c: bool,
}

/// Scenario P25(a, e) through the order and tick path, with no failure made
/// by hand: the road land type of our ruleset gets a zero foot multiplier
/// (its other multipliers stay at 1, so its category stays passable), and
/// the cell east of the walker and the one south of that become road. A
/// move order to the first (part a) makes a search that passes the gate and
/// fails, the destination next to the walker (claim 15); the walker keeps it
/// (claim 23 row 3), with a close-enough distance of half a cell below its
/// D. In tick *c* + 10 a second move order goes to `to`. Checks that the
/// walker stays in its cell from *c* to *c* + 100.
// spec: S-0208 claim 25 (r75, Changes for S-0212 P25 a: a test close-enough distance below D)
fn p25_e_real(to: impl Fn(CellXY) -> CellXY) -> RealRun {
    let mut r = ruleset_r90();
    r.sim.general.close_enough = Some(Decimal("0.5".into()));
    r.sim.land_types.insert(
        LandType::Road,
        LandRow {
            foot: Some(Decimal("0".into())),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 1);
    let home = mid(SIZE, 0, 0);
    let g = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, home, 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    trace::switch_on(&mut w);
    let near = CellXY::new(home.x + 1, home.y);
    for c in [near, CellXY::new(near.x, near.y + 1)] {
        w.cells.grid.get_mut(c).unwrap().land = land::ROAD;
        glasnost_sim::regions::cell_changed(&mut w, c);
    }
    move_to(&mut w, g, near);
    let (c, first_result) = next_search(&mut w, g, 5).expect("a computation at c");
    // Part (a): the passed-gate search fails and the walker keeps its
    // destination, its timer 90 ticks from that computation.
    assert_eq!(
        first_result,
        Search::SearchFailed,
        "passed the gate, failed"
    );
    assert_eq!(w.obj(g).unwrap().dest, Some(Target::Cell(near)), "kept");
    let runs_at_c = path::delay_runs(&w, g);
    let mut at_order = None;
    let mut ticks = Vec::new();
    while w.tick <= c + 100 {
        let k = w.tick;
        if k == c + 10 {
            move_to(&mut w, g, to(near));
        }
        tick::step(&mut w);
        let found = searches(&trace::take(&w), g);
        ticks.push((k, found.iter().map(|s| s.1).collect()));
        assert_eq!(w.obj(g).unwrap().pos.cell(), home, "it stays, tick {k}");
        if k == c + 10 {
            assert_eq!(
                w.obj(g).unwrap().dest,
                Some(Target::Cell(to(near))),
                "no targeting substitution"
            );
            at_order = Some((path::delay_runs(&w, g), w.obj(g).unwrap().path.retry_at));
        }
    }
    RealRun {
        c,
        at_order: at_order.expect("the order's tick"),
        ticks,
        runs_at_c,
    }
}

/// The P25(a, e) expectation for a walker in *move* (S-0208 r75, Changes
/// for S-0212 P25): the timer has run out at the end of tick *c*, and on
/// every later tick, the order's tick included, the walker makes one
/// computation, which fails again on our blocked tiles (claim 15) and keeps
/// the destination (claim 23 row 3); at the end of the order's tick no
/// delay runs.
fn p25_e_real_expect(run: &str, got: RealRun) {
    assert!(
        !got.runs_at_c,
        "{run}: the timer has run out at the end of c"
    );
    assert!(!got.at_order.0, "{run}: no delay at the order's tick");
    assert!(got.ticks.len() >= 90, "{run}: ticks read");
    for (k, results) in &got.ticks {
        assert!(*k > got.c, "{run}: after c");
        assert_eq!(
            results,
            &vec![Search::SearchFailed],
            "{run}: one computation in tick {k}"
        );
    }
}

// spec: S-0212 claim 21 (r98, Q-2896: a walker's running retry delay ends when it is given the destination it already holds, by an order in a later tick than the search that started it), claim 15 (a blocked destination next to the unit gives no path) and claim 23 row 3 (a walker keeps a destination within 1 cell); scenario P25 parts a and e run 1, through the order and tick path
// spec: S-0208 claim 25 (r75, After a failed search, and Changes for S-0212 P25 a and e: for a walker in move the end-of-update assignment ends the delay in tick c, and it computes on every later tick)
#[test]
fn seams7b_p25_e_run_1_real_failed_search_an_order_to_the_held_cell() {
    p25_e_real_expect("run 1", p25_e_real(|near| near));
}

// spec: S-0212 claim 21 (a destination changed to a different cell ends a walker's running retry delay), claim 15 and claim 23 row 3; scenario P25 parts a and e run 2, through the order and tick path
// spec: S-0208 claim 25 (r75, Changes for S-0212 P25 a and e, as for run 1)
#[test]
fn seams7b_p25_e_run_2_real_failed_search_an_order_to_the_second_tile() {
    p25_e_real_expect("run 2", p25_e_real(|near| CellXY::new(near.x, near.y + 1)));
}

// spec: S-0212 claim 21 (r97: a destination cleared ends a running delay of the walk locomotor, so the cell it kept, ordered again, is a different destination from none and is searched at once)
#[test]
fn claim_21_a_stop_ends_the_delay() {
    let (mut w, g) = one_near();
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
