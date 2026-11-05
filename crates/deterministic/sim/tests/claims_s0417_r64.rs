//! Claim tests of S-0417 revision 64 (answers Q-1850 and the walk-out of
//! Q-1757): claim 17's scatter request to an infantry unit resting on a
//! cell that holds a building, and the factory-release consequences in
//! claims 10, 12 and 14 (*Unit after unit*). Synthetic data only: the
//! production ruleset's invented barracks and infantry types, no game data.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::behaviour::Door;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};

fn new_unit(w: &mut World, name: &str) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    objects::create(w, Kind::Infantry, ti, Some(HouseId(0)))
}

/// Whether cell `c` lists building `k`.
fn holds(w: &World, c: CellXY, k: ObjectId) -> bool {
    w.cells.list(c, Layer::Ground).contains(&k)
}

/// The cell `id` stands in.
fn cell_of(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

/// The scatter answers recorded for `id`: (tick, accepted, destination).
fn answers(events: &[trace::Event], id: ObjectId) -> Vec<(i32, bool, Option<CellXY>)> {
    events
        .iter()
        .filter(|e| e.unit == id)
        .filter_map(|e| match e.what {
            What::Asked { accepted, dest } => Some((e.tick, accepted, dest)),
            _ => None,
        })
        .collect()
}

/// A barracks with no rally point and one GRUNT released from it, the
/// trace on. Returns the world, K and the unit.
fn released(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    trace::switch_on(&mut w);
    let g = new_unit(&mut w, "GRUNT");
    assert_eq!(release::release(&mut w, k, g), Outcome::Released);
    (w, k, g)
}

// spec: S-0417 claim 10 (no rally point: no destination at the release; the unit rests on a cell that holds K, in contact and tethered)
// spec: S-0417 claim 17 (the first request comes in the unit's first turn, the next tick; it is forced with no movement in progress, so it is accepted; the search destination holds no K; the behaviour stays guard, no move queued)
// spec: S-0417 claim 12 (the request's draw belongs to that turn of the unit, not to its release)
#[test]
fn claim_17_first_turn_request() {
    let (mut w, k, g) = released(&ruleset());
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, None);
    assert!(holds(&w, cell_of(&w, g), k), "it rests on a cell of K");
    assert!(
        answers(&trace::take(&w), g).is_empty(),
        "no request at the release"
    );
    let t = w.tick;
    step(&mut w);
    let asked = answers(&trace::take(&w), g);
    assert_eq!(asked.len(), 1, "one request in its first turn: {asked:?}");
    let (tick, accepted, dest) = asked[0];
    assert_eq!(tick, t, "in the tick after the release");
    assert!(accepted);
    let d = dest.expect("the search finds a cell");
    assert!(!holds(&w, d, k), "the destination holds no K");
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(d)));
    // spec: S-0417 claim 17 (During the walk-off: a search destination leaves the current behaviour as it was)
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_ne!(o.beh.queued, Some(Behaviour::Move));
}

// spec: S-0417 claim 17 (accepted: one draw, S-0208 claim 13; while the locomotor has movement in progress the request is not effectively forced and S-0208 claim 11's third case refuses it with no draw, the destination kept)
#[test]
fn claim_17_one_draw_then_refusals() {
    let (mut w, _k, g) = released(&ruleset());
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: true,
    };
    let mut probe = w.clone();
    let before = probe.rng.taken();
    let mut one = probe.rng.clone();
    let q = w.rules().quirks;
    // spec: S-0208 claims 12 and 13 (the one ranged draw over 0 … 4)
    one.ranged(0, 4, q.ranged_draw_half_span);
    assert!(scatter::request_resting(&mut probe, g, req));
    assert_eq!(
        probe.rng.taken() - before,
        one.taken() - before,
        "one ranged draw when accepted"
    );
    // In the game: the first turn's request, then the walk.
    step(&mut w);
    let first = w.obj(g).unwrap().dest;
    assert!(first.is_some());
    let mut walking = false;
    for _ in 0..20 {
        step(&mut w);
        if glasnost_sim::occupancy::in_progress(&w, g) {
            walking = true;
            break;
        }
    }
    assert!(walking, "it walks off");
    let mut probe = w.clone();
    let before = probe.rng.taken();
    assert!(
        !scatter::request_resting(&mut probe, g, req),
        "refused while moving"
    );
    assert_eq!(probe.rng.taken(), before, "no draw");
    assert_eq!(probe.obj(g).unwrap().dest, first, "the destination is kept");
}

// spec: S-0417 claim 17 (No path, r79: a destination different from the one the walking unit holds ends a running retry delay, S-0212 claim 21, so the request's destination is searched for at once; r68's walker that held it unsearched while the delay ran is gone)
#[test]
fn claim_17_running_delay_ended_by_the_request() {
    let (mut w, k, g) = released(&ruleset());
    let mut plain = w.clone();
    // A retry delay that runs past the next ticks.
    let wait = w.tick + 30;
    w.obj_mut(g).unwrap().path.retry_at = wait;
    step(&mut w);
    let d = w.obj(g).unwrap().dest;
    assert!(d.is_some(), "the first request gives a destination");
    assert!(
        !glasnost_sim::path::delay_runs(&w, g),
        "the different destination ended it"
    );
    // A refused request takes no draw.
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: true,
    };
    let mut probe = w.clone();
    let before = probe.rng.taken();
    assert!(!scatter::request_resting(&mut probe, g, req));
    assert_eq!(probe.rng.taken(), before, "no draw");
    // It steps long before the delay would be over.
    let mut look = w.clone();
    let at = look.obj(g).unwrap().pos;
    let mut moved = false;
    for _ in 0..5 {
        step(&mut look);
        moved |= look.obj(g).unwrap().pos != at;
    }
    assert!(moved && look.tick < wait, "it walks while the delay would run");
    // It walks off as the unit with no delay does, tick for tick.
    let n = until_free(&mut w, g, 200).expect("contact ends");
    let m = until_free(&mut plain, g, 201).expect("contact ends");
    assert_eq!(n + 1, m, "the same tick as without the delay");
    assert!(!holds(&w, cell_of(&w, g), k));
    let asked = answers(&trace::take(&w), g);
    assert!(asked[0].1);
    assert!(
        asked[1..].iter().all(|a| !a.1),
        "the later ones refused: {asked:?}"
    );
}

/// Runs until `g`'s contact with K ends, at most `limit` ticks; returns
/// the ticks taken.
fn until_free(w: &mut World, g: ObjectId, limit: i32) -> Option<i32> {
    for n in 1..=limit {
        step(w);
        if w.obj(g).unwrap().contact.is_none() {
            return Some(n);
        }
    }
    None
}

// spec: S-0417 claim 17 (When K is free again: the first standing-position arrival is claim 10's clear report; Unit after unit: the unit accepts one request for its walk-off and ends on its destination, in guard)
// spec: S-0417 claim 14 (a later release is busy only before the earlier unit's first arrival)
#[test]
fn claim_17_walk_off_frees_k() {
    let (mut w, k, g) = released(&ruleset());
    let second = new_unit(&mut w, "GRUNT");
    let mut early = w.clone();
    assert_eq!(
        release::release(&mut early, k, second),
        Outcome::Busy,
        "busy before the first arrival"
    );
    let n = until_free(&mut w, g, 200).expect("contact ends");
    assert!(!release::tethered(&w, g));
    // The rest of the walk; the request is refused on the way.
    for _ in 0..200 {
        step(&mut w);
    }
    let asked = answers(&trace::take(&w), g);
    let accepted: Vec<_> = asked.iter().filter(|a| a.1).collect();
    assert_eq!(accepted.len(), 1, "one accepted request: {asked:?}");
    let d = accepted[0].2.unwrap();
    let o = w.obj(g).unwrap();
    assert_eq!(o.pos.cell(), d, "it ends on its destination");
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert!(!holds(&w, d, k));
    assert!(n < 200);
    assert_eq!(release::release(&mut w, k, second), Outcome::Released);
}

// spec: S-0417 claim 17 (Unit after unit, Q-1850: five units queued at one factory with no rally point all come out, none busy, and end on five different cells none of which holds K)
// spec: S-0417 claim 14 (one line finishes its items at least 54 ticks apart, S-0404 claim 7)
#[test]
fn claim_17_unit_after_unit() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("GRUNT").unwrap().cost = Some(100);
    let mut w = game(&r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    for _ in 0..5 {
        assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    }
    let c0 = credits(&w, 0);
    let mut seen: Vec<(ObjectId, i32)> = Vec::new();
    for _ in 0..3000 {
        step(&mut w);
        for (&id, o) in &w.objects {
            if o.kind == Kind::Infantry && o.on_map && !seen.iter().any(|s| s.0 == id) {
                seen.push((id, w.tick));
            }
        }
        if seen.len() == 5 && line(&w, 0, grunt).is_none_or(|l| l.item.is_none()) {
            break;
        }
    }
    assert_eq!(seen.len(), 5, "all five come out: {seen:?}");
    for p in seen.windows(2) {
        assert!(p[1].1 - p[0].1 >= 54, "releases 54 or more ticks apart");
    }
    for _ in 0..300 {
        step(&mut w);
    }
    let mut cells: Vec<CellXY> = seen.iter().map(|s| cell_of(&w, s.0)).collect();
    for &(id, _) in &seen {
        let o = w.obj(id).unwrap();
        assert_eq!(o.contact, None);
        assert_eq!(o.beh.current, Some(Behaviour::Guard));
        assert!(!holds(&w, o.pos.cell(), k), "{id:?} is off K");
    }
    cells.sort();
    cells.dedup();
    assert_eq!(cells.len(), 5, "five different cells");
    assert_eq!(credits(&w, 0), c0 - 5 * 100, "no refund");
}

/// A GRUNT spawned in guard on the cell of a 1 × 1 building of type
/// `name`; one tick. Returns the answers recorded for it.
fn on_building(r: &Ruleset, name: &str, open_gate: bool) -> Vec<(i32, bool, Option<CellXY>)> {
    let mut w = game(r);
    let c = glasnost_sim::testkit::mid(SIZE, 0, 0);
    let b = building(&mut w, name, 0, 0, 0);
    if open_gate {
        let o = w.obj_mut(b).unwrap();
        o.beh.current = Some(Behaviour::Open);
        o.beh.gate.door = Door::Open;
        assert!(glasnost_sim::process::gate_open(&w, b));
    }
    let g = glasnost_sim::testkit::spawn(&mut w, Kind::Infantry, "GRUNT", 0, c, 0);
    assert!(holds(&w, c, b));
    trace::switch_on(&mut w);
    step(&mut w);
    answers(&trace::take(&w), g)
}

/// The production ruleset with 1 × 1 buildings for claim 17's exemptions.
fn exempt_ruleset() -> Ruleset {
    let mut r = ruleset();
    for (n, f) in [("GHOST", 0), ("FENCE", 1), ("FWALL", 2), ("GATE", 3)] {
        let mut b = plain(100);
        match f {
            0 => b.invisible_in_game = Some(true),
            1 => b.laser_fence = Some(true),
            2 => b.firestorm_wall = Some(true),
            _ => b.gate = Some(true),
        }
        r.registries.get_mut("buildings").unwrap().push(n.into());
        r.sim.buildings.insert(n.into(), b);
    }
    r
}

// spec: S-0417 claim 17 (the rule is not specific to factories: a resting unit on any counting building's cell is sent the request; a building of an invisible type (WP7), a laser-fence section in its down states (WP8, design), a firestorm wall with its owner's firestorm off (WP10) and an open gate (WP11) do not count; a closed gate does)
#[test]
fn claim_17_buildings_that_count() {
    let r = exempt_ruleset();
    let n = |a: &[(i32, bool, Option<CellXY>)]| a.len();
    assert_eq!(
        n(&on_building(&r, "HUT", false)),
        1,
        "a plain building counts"
    );
    assert_eq!(
        n(&on_building(&r, "GATE", false)),
        1,
        "a closed gate counts"
    );
    for name in ["GHOST", "FENCE", "FWALL"] {
        assert_eq!(n(&on_building(&r, name, false)), 0, "{name} does not count");
    }
    assert_eq!(
        n(&on_building(&r, "GATE", true)),
        0,
        "an open gate does not count"
    );
}

// spec: S-0417 claim 17 (the current behaviour is guard or area guard: area guard is sent the request too; another resting behaviour is not)
#[test]
fn claim_17_behaviours() {
    let r = ruleset();
    let stale = glasnost_sim::testkit::rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    for (b, sent) in [
        (Behaviour::Guard, true),
        (Behaviour::AreaGuard, true),
        (Behaviour::Sleep, false),
    ] {
        let mut w = game(&r);
        let c = glasnost_sim::testkit::mid(SIZE, 0, 0);
        let hut = building(&mut w, "HUT", 0, 0, 0);
        let g = glasnost_sim::testkit::spawn(&mut w, Kind::Infantry, "GRUNT", 0, c, 0);
        let tick = w.tick;
        w.obj_mut(g).unwrap().beh.force(b, tick, stale);
        assert!(holds(&w, c, hut));
        trace::switch_on(&mut w);
        step(&mut w);
        let asked = answers(&trace::take(&w), g);
        assert_eq!(!asked.is_empty(), sent, "{b:?}: {asked:?}");
    }
}

// spec: S-0417 claim 17 (During the walk-off: a teleporting unit given a search destination jumps in its request's own turn; its jump's arrival is claim 10's clear report)
#[test]
fn claim_17_teleporting_unit_jumps_in_the_request_turn() {
    let mut r = ruleset();
    let g = r.sim.infantry.get_mut("GRUNT").unwrap();
    g.teleporter = Some(true);
    g.locomotor = Some(Locomotor::Teleport);
    let (mut w, k, g) = released(&r);
    let from = cell_of(&w, g);
    assert!(holds(&w, from, k));
    step(&mut w);
    let asked = answers(&trace::take(&w), g);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(asked[0].1, "accepted");
    let d = cell_of(&w, g);
    assert_ne!(d, from, "it jumped in the request's turn");
    let o = w.obj(g).unwrap();
    assert_eq!(o.contact, None, "the jump's arrival reports clear");
    assert!(!release::tethered(&w, g));
    assert!(!holds(&w, d, k));
}

// spec: S-0417 claim 10 (a rally point set after the release does not move a unit already released; U10 b)
#[test]
fn claim_10_later_rally_point() {
    let (mut w, k, g) = released(&ruleset());
    until_free(&mut w, g, 200).expect("contact ends");
    for _ in 0..200 {
        step(&mut w);
    }
    let rest = cell_of(&w, g);
    let tl = w.obj(k).unwrap().pos.cell();
    w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(CellXY::new(tl.x + 6, tl.y + 6)));
    for _ in 0..300 {
        step(&mut w);
    }
    let o = w.obj(g).unwrap();
    assert_eq!(o.pos.cell(), rest);
    assert_eq!(o.dest, None);
    assert_eq!(o.guard_spot, None);
}

// spec: S-0417 claim 17 (row WP2: an enslaved unit is not sent the request when the building is its master and the cell is the master's unloading cell)
#[test]
fn claim_17_enslaved_unit_at_its_master() {
    for enslaved in [false, true] {
        let mut w = game(&ruleset());
        let c = glasnost_sim::testkit::mid(SIZE, 0, 0);
        let hut = building(&mut w, "HUT", 0, 0, 0);
        assert_eq!(glasnost_sim::econ::slaves::unloading_cell(&w, hut), Some(c));
        let g = glasnost_sim::testkit::spawn(&mut w, Kind::Infantry, "GRUNT", 0, c, 0);
        if enslaved {
            w.obj_mut(g).unwrap().econ.slave = Some(glasnost_sim::econ::slaves::SlaveState {
                master: Some(hut),
                ..Default::default()
            });
        }
        trace::switch_on(&mut w);
        step(&mut w);
        let asked = answers(&trace::take(&w), g);
        assert_eq!(asked.is_empty(), enslaved, "enslaved {enslaved}: {asked:?}");
    }
}
