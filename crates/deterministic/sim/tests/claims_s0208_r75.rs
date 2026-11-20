//! Claim tests of S-0208 revision 75 (Q-3301 and Q-3303 answered). Claim
//! 25: a walker's failed fresh search past the gate clears its destination
//! near its locomotor destination, or else leaves it with no locomotor
//! destination, for every kind of destination; a walker in *move* is given
//! the destination it holds again at the end of the same update (S-0215
//! claim 22), so it searches once per tick; a walker standing in its own
//! destination cell searches. Claim 26: S-0212 claim 24's substitute is
//! assigned as the walker's destination whatever it held; the mission target
//! that the search reads; `quirks.walker_search_ignores_queued_enter`.
//! Synthetic data only: the test kit's invented walker and buggy, our own
//! road row with a zero foot multiplier, no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S45*.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LandRow, LandType, SequenceEntry};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::occupancy::{self, in_progress};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, SIZE, mid};
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The combat ruleset with R = 90 ticks (the path-retry delay field at 0.1
/// game minutes), the road row's foot multiplier at 0 (so a road cell next
/// to a walker passes the gate and gives no path, S-0212 claim 15) and the
/// close-enough distance at `close` cells, else the default.
fn ruleset(close: Option<&str>) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.path_delay = Some(Decimal("0.1".into()));
    if let Some(c) = close {
        r.sim.general.close_enough = Some(Decimal(c.into()));
    }
    r.sim.land_types.insert(
        LandType::Road,
        LandRow {
            foot: Some(Decimal("0".into())),
            ..Default::default()
        },
    );
    r
}

/// A game under `r` with a WALKER of house 0 at the map's middle, orders
/// open (S-0201 claim 14), the trace on.
fn one(r: &Ruleset) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 1);
    let g = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    trace::switch_on(&mut w);
    (w, g)
}

/// Makes the cell east of the walker's a road cell, which it cannot enter;
/// returns that cell.
fn road_east(w: &mut World, g: ObjectId) -> CellXY {
    let c = w.obj(g).unwrap().pos.cell();
    let near = CellXY::new(c.x + 1, c.y);
    w.cells.grid.get_mut(near).unwrap().land = land::ROAD;
    glasnost_sim::regions::cell_changed(w, near);
    near
}

/// A move order to `d`.
fn move_to(w: &mut World, id: ObjectId, d: Target) {
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(d),
    };
    assert!(orders::issue(w, HouseId(0), o), "move order refused");
}

/// Gives `id` destination `d` where it reaches the locomotor, with no
/// behaviour change (S-0107 claim 15's hook, as a rule would).
fn give(w: &mut World, id: ObjectId, d: Option<Target>) {
    glasnost_sim::econ::assign_destination(w, id, d);
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

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

fn fraction(w: &World, id: ObjectId) -> bool {
    w.obj(id).unwrap().walk.as_ref().unwrap().fraction
}

fn kept(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id).unwrap().walk.as_ref().unwrap().kept
}

/// The centre of cell `c` at the surface.
fn centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::new(p.x, p.y, w.surface_at(p.x, p.y))
}

/// Steps up to `n` ticks until `id` has left position `from`.
fn run_until_moved(w: &mut World, id: ObjectId, from: Pos, n: i32) -> bool {
    for _ in 0..n {
        tick::step(w);
        if w.obj(id).unwrap().pos != from {
            return true;
        }
    }
    false
}

/// Steps up to `n` ticks until `id` stands in cell `c`.
fn run_until_cell(w: &mut World, id: ObjectId, c: CellXY, n: i32) -> bool {
    for _ in 0..n {
        tick::step(w);
        if w.obj(id).unwrap().pos.cell() == c {
            return true;
        }
    }
    false
}

// spec: S-0208 claim 25 (r75, Every kind of destination and After a failed search: for a cell destination a failed fresh search past the gate that keeps the destination, S-0212 claim 23 row 3, leaves the walker with no locomotor destination, so a walker in a behaviour other than move makes no search when the retry delay runs out) and S-0212 claim 21 (r75 Changes)
#[test]
fn r75_a_kept_cell_destination_is_not_searched_when_r_runs_out() {
    let (mut w, g) = one(&ruleset(Some("0.5")));
    let near = road_east(&mut w, g);
    give(&mut w, g, Some(Target::Cell(near)));
    let (c, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::SearchFailed, "void unless it passed the gate");
    assert_ne!(
        w.obj(g).unwrap().beh.seen(),
        Some(Behaviour::Move),
        "void unless the walker is not in move"
    );
    assert_eq!(dest(&w, g), Some(Target::Cell(near)), "kept (row 3)");
    assert!(!in_progress(&w, g), "its movement in progress ends");
    assert!(!fraction(&w, g), "speed fraction 0 (S-0310 Terms)");
    assert_eq!(
        next_search(&mut w, g, 150),
        None,
        "no search after R (c = {c})"
    );
    assert_eq!(dest(&w, g), Some(Target::Cell(near)), "still kept");
}

// spec: S-0208 claim 25 (r75, Every kind of destination: a walker whose cell destination's point is less than the close-enough distance away loses it when its fresh search fails past the gate)
#[test]
fn r75_a_failed_search_near_a_cell_destination_clears_it() {
    let (mut w, g) = one(&ruleset(None));
    let near = road_east(&mut w, g);
    give(&mut w, g, Some(Target::Cell(near)));
    let (_, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::SearchFailed, "void unless it passed the gate");
    assert_eq!(
        dest(&w, g),
        None,
        "D (256) is less than the close-enough distance"
    );
    assert!(!in_progress(&w, g));
}

// spec: S-0208 claim 25 (r75, After a failed search: a walker in move that keeps its destination is given it again at the end of the same update, S-0215 claim 22: its stored path cleared, its retry delay ended, a locomotor destination again and a speed fraction of 1 with no walking step; it searches once per tick, does not change its position and ends every tick with its destination, movement in progress and a speed fraction of 1) and scenario W20 part a; S-0310 Terms (r75 Changes: the speed fraction is 1 after that assignment, so the unit is moving for claim 8)
#[test]
fn r75_w20_a_walker_in_move_searches_every_tick() {
    let (mut w, g) = one(&ruleset(Some("0.5")));
    let near = road_east(&mut w, g);
    let home = w.obj(g).unwrap().pos;
    // An unarmed enemy far away, only to ask the firing check about.
    let e = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(SIZE, -15, 15), 0);
    move_to(&mut w, g, Target::Cell(near));
    let (c, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::SearchFailed, "void unless it passed the gate");
    for k in 1..=20 {
        assert_eq!(dest(&w, g), Some(Target::Cell(near)), "kept, tick {k}");
        assert_eq!(w.obj(g).unwrap().pos, home, "it stays, tick {k}");
        assert!(in_progress(&w, g), "movement in progress, tick {k}");
        assert!(
            fraction(&w, g),
            "speed fraction 1 at the tick's end, tick {k}"
        );
        // spec: S-0310 claim 8 (an infantry unit with a speed fraction of 1 is moving) and Terms (r75 Changes)
        assert_eq!(
            check::check(&w, g, Some(Target::Object(e)), 0, false),
            Answer::Moving,
            "the firing check reads it as moving, tick {k}"
        );
        assert!(!path::delay_runs(&w, g), "no retry delay, tick {k}");
        assert_eq!(
            w.obj(g).unwrap().beh.seen(),
            Some(Behaviour::Move),
            "still in move"
        );
        tick::step(&mut w);
        assert_eq!(
            searches(&trace::take(&w), g),
            vec![(c + k, Search::SearchFailed)],
            "one failed search per tick"
        );
    }
}

// spec: S-0208 claim 25 (r75, A cell destination that is the walker's own cell, Q-3301 question 3: with movement in progress it makes the fresh search even within two height levels of the cell's point; the search finds no path, S-0212 claim 12; with D not less than the close-enough distance it keeps the destination and in move searches again on every tick without changing its position) and scenario W20 part a
#[test]
fn r75_w20_a_own_cell_destination_in_move_is_kept_and_searched() {
    let (mut w, g) = one(&ruleset(Some("0")));
    let home = w.obj(g).unwrap().pos;
    move_to(&mut w, g, Target::Cell(home.cell()));
    let (c, _) = next_search(&mut w, g, 5).expect("the walker searches its own cell");
    for k in 1..=10 {
        assert_eq!(
            dest(&w, g),
            Some(Target::Cell(home.cell())),
            "kept, tick {k}"
        );
        assert_eq!(w.obj(g).unwrap().pos, home, "it stays, tick {k}");
        assert!(fraction(&w, g), "speed fraction 1, tick {k}");
        tick::step(&mut w);
        let s = searches(&trace::take(&w), g);
        assert_eq!(s.len(), 1, "one search per tick, tick {}", c + k);
        assert_ne!(s[0].1, Search::Found, "no path in its own cell");
    }
}

// spec: S-0208 claim 25 (r75, A cell destination that is the walker's own cell: the walker loses it when D is less than the close-enough distance and it is not held by a tether link) and scenario W20 part b
#[test]
fn r75_w20_b_an_own_cell_destination_within_close_enough_is_cleared() {
    let (mut w, g) = one(&ruleset(None));
    let home = w.obj(g).unwrap().pos;
    move_to(&mut w, g, Target::Cell(home.cell()));
    testkit::run(&mut w, 3);
    assert_eq!(dest(&w, g), None, "cleared");
    assert_eq!(w.obj(g).unwrap().pos, home);
}

// spec: S-0208 claim 25 (r75, After a failed search: for a destination that names an object the end-of-update assignment in move gives the object's movement point at that moment, so a walker standing in its old point's cell heads for the object's new place) and S-0215 claim 22 (r75 Changes)
#[test]
fn r75_a_walker_in_move_takes_its_objects_new_point() {
    let (mut w, g) = one(&ruleset(Some("0")));
    let i = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(g).unwrap().beh.force(Behaviour::Move, t, stale);
    give(&mut w, g, Some(Target::Object(i)));
    testkit::run(&mut w, 3);
    let home = w.obj(g).unwrap().pos;
    assert_eq!(dest(&w, g), Some(Target::Object(i)), "void unless kept");
    move_to(&mut w, i, Target::Cell(mid(SIZE, 0, 8)));
    let ran = run_until_moved(&mut w, g, home, 200);
    assert!(ran, "the walker follows its object's new place");
}

/// A prone, crawling WALKER of human house 0 at the map's middle, with image
/// frames for the sequences involved (S-0107 claim 6 row 1).
fn prone(close: &str, fraidycat: bool) -> (World, ObjectId) {
    let mut r = ruleset(Some(close));
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.fraidycat = Some(fraidycat);
    wk.crawls = Some(true);
    for k in ["stand", "walk", "go_prone", "get_up", "prone", "crawl"] {
        wk.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let (mut w, g) = one(&r);
    w.obj_mut(g).unwrap().infantry.as_mut().unwrap().prone = true;
    (w, g)
}

/// Whether a prone walker in [`prone`]'s scene, in move to its own cell,
/// asks for get-up within 5 ticks of lying down again in that loop.
fn loop_asks_get_up(fraidycat: bool) -> bool {
    use glasnost_sim::infantry::Seq;
    let (mut w, g) = prone("0", fraidycat);
    let home = w.obj(g).unwrap().pos.cell();
    move_to(&mut w, g, Target::Cell(home));
    // The order itself asks for get-up; once the walker is in the loop it
    // lies down again, and only the loop's assignments can ask once more.
    testkit::run(&mut w, 10);
    assert_eq!(dest(&w, g), Some(Target::Cell(home)), "void unless kept");
    let s = w.obj_mut(g).unwrap().infantry.as_mut().unwrap();
    s.prone = true;
    s.seq = Some(Seq::Prone);
    let mut asked = false;
    for _ in 0..5 {
        tick::step(&mut w);
        let s = w.obj(g).unwrap().infantry.clone().unwrap();
        asked |= s.seq == Some(Seq::GetUp) || !s.prone;
    }
    asked
}

// spec: S-0208 claim 25 (r75, After a failed search: each end-of-update assignment in move gives the destination the walker already holds, so a prone walker that S-0213 claim 21 makes eligible asks for get-up, S-0107 claim 15; a fraidycat type is not eligible, so nothing asks for it)
#[test]
fn r75_a_prone_walker_in_that_loop_asks_for_get_up() {
    assert!(
        loop_asks_get_up(false),
        "the repeated assignment asks for get-up"
    );
    assert!(
        !loop_asks_get_up(true),
        "control: a fraidycat type, nothing else asks"
    );
}

// spec: S-0208 claim 25 (r75, Every kind of destination: a kept locomotor destination with no destination, S-0507 claim 6a; a failed fresh search past the gate that does not clear leaves no locomotor destination, so no search follows)
#[test]
fn r75_a_kept_locomotor_destination_is_lost_at_a_failed_search() {
    let (mut w, g) = one(&ruleset(Some("0.5")));
    let near = road_east(&mut w, g);
    let k = centre(&w, near);
    w.obj_mut(g).unwrap().walk.as_mut().unwrap().kept = Some(k);
    let (_, r) = next_search(&mut w, g, 5).expect("a search for the kept point's cell");
    assert_eq!(r, Search::SearchFailed, "void unless it passed the gate");
    assert_eq!(dest(&w, g), None);
    assert_eq!(kept(&w, g), None, "no locomotor destination is left");
    assert_eq!(next_search(&mut w, g, 150), None, "no search after R");
}

/// A WALKER of house 0 at (−6, 0) and a parked BUGGY of the same house at
/// (0, 0): an allied vehicle at rest, row WP16's *stationary friendly*.
fn walker_and_parked_buggy() -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&ruleset(None), SIZE, 1);
    let g = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, -6, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, 0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    (w, g, v)
}

/// Runs up to 10 ticks until `g` holds a cell destination; returns it.
fn cell_dest_within(w: &mut World, g: ObjectId) -> Option<CellXY> {
    for _ in 0..10 {
        tick::step(w);
        if let Some(Target::Cell(c)) = dest(w, g) {
            return Some(c);
        }
    }
    None
}

// spec: S-0208 claim 26 (r75: when S-0212 claim 24 adopts a substitute cell for the cell a walker's search looks for, the walker is given that cell as its destination by an assignment, replacing a destination that names an object, which it then no longer holds)
#[test]
fn r75_claim_26_a_substitute_replaces_an_object_destination() {
    let (mut w, g, v) = walker_and_parked_buggy();
    let vc = w.obj(v).unwrap().pos.cell();
    give(&mut w, g, Some(Target::Object(v)));
    let c = cell_dest_within(&mut w, g).expect("the substitute is its destination");
    assert_ne!(c, vc, "a cell near the vehicle, not its own");
    let walked = run_until_cell(&mut w, g, c, 300);
    assert!(walked, "it walks to the substitute");
    let mut ended = false;
    for _ in 0..100 {
        tick::step(&mut w);
        if dest(&w, g).is_none() {
            ended = true;
            break;
        }
        assert_eq!(
            dest(&w, g),
            Some(Target::Cell(c)),
            "it holds the substitute"
        );
    }
    assert!(ended, "its walk ends (claim 16)");
    assert_eq!(w.obj(g).unwrap().pos.cell(), c, "in the substitute cell");
}

// spec: S-0208 claim 26 (r75: the assignment replaces no destination with a kept locomotor destination after a loss of S-0507 claim 6a; the walker has a destination again, the substitute cell, and the kept point is gone)
#[test]
fn r75_claim_26_a_substitute_replaces_a_kept_locomotor_destination() {
    let (mut w, g, v) = walker_and_parked_buggy();
    let p = w.obj(v).unwrap().pos;
    w.obj_mut(g).unwrap().walk.as_mut().unwrap().kept = Some(p);
    let c = cell_dest_within(&mut w, g).expect("the substitute is its destination");
    assert_ne!(c, p.cell());
    assert_eq!(kept(&w, g), None, "the point it held before is gone");
}

/// Sets `id`'s current and queued behaviours.
fn behaviours(w: &mut World, id: ObjectId, cur: Option<Behaviour>, queued: Option<Behaviour>) {
    let o = w.obj_mut(id).unwrap();
    o.beh.current = cur;
    o.beh.queued = queued;
}

/// A world with a WALKER (engineer flag `engineer`) holding a cell
/// destination, under `quirk` for `walker_search_ignores_queued_enter`.
fn mission(engineer: bool, quirk: bool) -> (World, ObjectId, Target) {
    let mut r = ruleset(None);
    r.sim.infantry.get_mut("WALKER").unwrap().engineer = Some(engineer);
    r.globals.quirks.walker_search_ignores_queued_enter = quirk;
    let (mut w, g) = one(&r);
    let d = Target::Cell(mid(SIZE, 5, 0));
    w.obj_mut(g).unwrap().dest = Some(d);
    (w, g, d)
}

// spec: S-0208 claim 26 (r75: guard, area guard and patrolling give a walker its destination as mission target only for an engineer type, Terms and Import mapping Engineer; no other type gives one) and S-0209 Terms (mission target) and claim 4 Open questions (engineer-like unit)
#[test]
fn r75_claim_26_guard_gives_a_mission_target_only_to_an_engineer_type() {
    for b in [Behaviour::Guard, Behaviour::AreaGuard, Behaviour::Patrol] {
        let (mut w, g, d) = mission(true, true);
        behaviours(&mut w, g, Some(b), None);
        assert_eq!(occupancy::mission_target(&w, g), Some(d), "engineer, {b:?}");
        let (mut w, g, _) = mission(false, true);
        behaviours(&mut w, g, Some(b), None);
        assert_eq!(
            occupancy::mission_target(&w, g),
            None,
            "not an engineer, {b:?}"
        );
    }
}

// spec: S-0208 claim 26 (r75: the mission target the search reads is the current behaviour's, a queued one's only while no behaviour is current, S-0215 claim 1; a queued enter behind a current behaviour is not read) and Edge cases (quirks.walker_search_ignores_queued_enter on, the original)
#[test]
fn r75_claim_26_a_queued_enter_behind_guard_gives_no_mission_target() {
    let (mut w, g, d) = mission(false, true);
    behaviours(&mut w, g, Some(Behaviour::Guard), Some(Behaviour::Enter));
    assert_eq!(occupancy::mission_target(&w, g), None, "switch on");
    behaviours(&mut w, g, None, Some(Behaviour::Enter));
    assert_eq!(
        occupancy::mission_target(&w, g),
        Some(d),
        "no current behaviour"
    );
}

// spec: S-0208 Edge cases (r75, quirks.walker_search_ignores_queued_enter off: while the walker's queued behaviour is enter and its current behaviour has no mission target, enter's mission target is the one S-0209 reads; a current behaviour with a mission target keeps its own)
#[test]
fn r75_switch_off_a_queued_enter_gives_its_mission_target() {
    let (mut w, g, d) = mission(false, false);
    behaviours(&mut w, g, Some(Behaviour::Guard), Some(Behaviour::Enter));
    assert_eq!(occupancy::mission_target(&w, g), Some(d), "switch off");
    behaviours(&mut w, g, Some(Behaviour::Guard), Some(Behaviour::Move));
    assert_eq!(
        occupancy::mission_target(&w, g),
        None,
        "only a queued enter"
    );
    let t = Target::Object(g);
    w.obj_mut(g).unwrap().dest = None;
    w.obj_mut(g).unwrap().target = Some(t);
    behaviours(&mut w, g, Some(Behaviour::Guard), Some(Behaviour::Enter));
    assert_eq!(
        occupancy::mission_target(&w, g),
        Some(t),
        "enter's target when it has no destination"
    );
}
