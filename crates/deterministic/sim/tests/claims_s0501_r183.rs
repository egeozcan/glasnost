//! Claim tests for S-0501 r183, sim issues S11 (#225, #226, #227): the own
//! landing point a stop order chooses is chosen once (claims 6 and 40), an
//! aircraft placed again starts with no flight of its own (claims 1, 8 and
//! 40), and a type's flight level of −1 is the unset value (Import
//! mapping). Synthetic data through the real tick path; every type, map and
//! number is invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S11*.

mod common;
mod econ_common;

use econ_common::{SIZE, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// LIFT: an unarmed, landable aircraft (invented values); its flight level
/// is `level`.
fn rules(level: Option<i32>) -> Ruleset {
    let mut r = econ_common::ruleset();
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        ..Default::default()
    };
    u.econ.landable = Some(true);
    u.econ.flight_level = level;
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("LIFT".into());
    r.sim.aircraft.insert("LIFT".into(), u);
    r
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn order_move(w: &mut World, a: ObjectId, c: CellXY) {
    assert!(issue(
        w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
}

/// A LIFT cruising in *move* is put in *guard*; *guard*'s *move* becomes
/// current in flight (claim 33), and `k` ticks after the tick of that
/// move's first processing it gets a stop order. Returns whether the
/// flight point chosen at the stop order was replaced before the
/// touchdown.
fn stop_in_the_window(k: usize) -> bool {
    let mut w = game(&rules(None));
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "LIFT",
        0,
        testkit::mid(SIZE, -10, 0),
        0,
    );
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    order_move(&mut w, a, CellXY::new(c.x + 24, c.y));
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && h == s.held && s.regive.is_none() && w.tick > 40 {
            break;
        }
    }
    let t = w.tick;
    w.obj_mut(a).unwrap().beh.force(Behaviour::Guard, t, true);
    let mut first = false;
    for _ in 0..60 {
        step(&mut w);
        if st(&w, a).regive.is_some() {
            first = true;
            break;
        }
    }
    assert!(first, "no new move");
    for _ in 0..k {
        step(&mut w);
    }
    if st(&w, a).regive.is_none() {
        return false; // the window is over
    }
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    step(&mut w);
    let s = st(&w, a);
    if !s.own_point {
        return false;
    }
    let p0 = s.flying_to;
    for _ in 0..400 {
        step(&mut w);
        let s = st(&w, a);
        if s.ground || s.flying_to.is_none() {
            break;
        }
        if s.flying_to != p0 {
            return true;
        }
    }
    false
}

// spec: S-0501 claim 6 (the new flight point is chosen at the moment the aircraft is given no destination, and it flies to that point) and claim 40 (the move's next processing gives its flight point to the flight once more)
#[test]
fn issue_225_a_stop_in_a_moves_first_window_chooses_the_own_point_once() {
    let replaced: Vec<usize> = (0..40).filter(|&k| stop_in_the_window(k)).collect();
    assert!(
        replaced.is_empty(),
        "the flight point chosen at the stop order was replaced for k = {replaced:?}"
    );
}

// spec: S-0501 claim 1 (a placement puts the aircraft on the ground or at cruise height) and claim 40 (a freshly placed aircraft has no unfinished flight); claim 8 (an aircraft on the ground and not landing takes off whatever the cell)
#[test]
fn issue_226_an_aircraft_placed_again_has_no_flight_of_its_own() {
    let mut w = game(&rules(None));
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "LIFT",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    step(&mut w);
    let here = w.obj(a).unwrap().pos.cell();
    // The flight it was in when it went inside: landing on an unfinished
    // flight toward a point, with the marks of that flight.
    if let Some(s) = w.obj_mut(a).unwrap().air.as_mut() {
        s.landing = true;
        s.taking_off = true;
        s.unfinished = true;
        s.flying_to = Some(Target::Cell(CellXY::new(here.x, here.y - 1)));
        s.handed = Some(Pos::centre(CellXY::new(here.x, here.y - 1), 0));
        s.own_point = true;
        s.raised = true;
        s.ignored = Some(Target::Cell(here));
        s.lost = true;
        s.sliding = true;
        s.stranded = true;
        s.origin = true;
        s.released = true;
        // Marks that outlast a flight (claim 1, S-0522 Terms).
        s.passenger_carrier = true;
        s.passes = 3;
    }
    glasnost_sim::objects::take_inside(&mut w, a);
    let p = w.obj(a).unwrap().pos;
    glasnost_sim::objects::place(&mut w, a, Pos::new(p.x, p.y, p.z + 50), 0);
    let s = st(&w, a);
    assert!(s.ground, "placed on the ground (claim 1)");
    assert_eq!(
        (s.landing, s.taking_off, s.unfinished, s.flying_to, s.handed),
        (false, false, false, None, None),
        "no flight of its own after the placement"
    );
    assert_eq!(
        (s.own_point, s.raised, s.ignored, s.lost),
        (false, false, None, false),
        "no marks of an old flight point"
    );
    assert_eq!(
        (s.sliding, s.stranded, s.origin, s.released),
        (false, false, false, false),
        "no state of an old touchdown or release"
    );
    assert!(
        s.passenger_carrier,
        "the passenger-carrier mark lasts (claim 1)"
    );
    assert_eq!(s.passes, 3, "the pass counter is not a flight's");
    // Given a destination in the cell of the old flight point, it takes
    // off (claim 8: on the ground and not landing).
    order_move(&mut w, a, CellXY::new(here.x, here.y - 1));
    for _ in 0..5 {
        step(&mut w);
    }
    assert!(air::height(&w, a) > 0, "it took off");
}

// spec: S-0501 Import mapping (an aircraft type's FlightLevel that is absent stays unset, -1, which means the game-wide value, not a height of -1)
#[test]
fn issue_227_a_flight_level_of_minus_one_is_the_game_wide_value() {
    for level in [None, Some(-1)] {
        let mut w = game(&rules(level));
        ready(&mut w);
        let wide = w.rules().air.flight_level;
        let a = testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "LIFT",
            0,
            testkit::mid(SIZE, -10, 0),
            0,
        );
        step(&mut w);
        let c = w.obj(a).unwrap().pos.cell();
        order_move(&mut w, a, CellXY::new(c.x + 20, c.y));
        for _ in 0..30 {
            step(&mut w);
        }
        assert_eq!(
            st(&w, a).held,
            wide,
            "FlightLevel {level:?}: the held height in cruise is the game-wide flight level"
        );
    }
}
