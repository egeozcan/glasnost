//! Claim tests for S-0501 r132 (Q-2576 to Q-2579): claim 7's untested
//! touchdown (`quirks.aircraft_descent_touchdown_untested`), the flight on
//! the ground after a touchdown or a refusal that kept the destination
//! (`quirks.aircraft_refused_kept_drop`), the landing's check of its flight
//! point's cell, claim 6's aircraft given no destination while it lands
//! (AC4's same-cell runs (4) and (5), the F8 case) and over a cell outside
//! the usable area (`quirks.aircraft_offmap_stop_start`), claim 24's entry
//! restarted in the docking stage and AC31 part h (4). Synthetic data
//! through the real tick path; every type, weapon, map and number is
//! invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn pad_type(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> glasnost_rules::sim::BuildingType {
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(docks);
    pad.econ.docking_offsets = offsets;
    pad
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with `docks` docking places and the displacements `offsets`, a 2 × 2
/// foundation); JET (armed, landable, not pad-bound, 2 rounds of the weapon
/// Agm, docking at APAD); BOUND (as JET, pad-bound). TANK, a ground vehicle
/// of the test house, is an occupant F7 refuses.
fn rules_with(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "buildings", "APAD");
    r.sim
        .buildings
        .insert("APAD".into(), pad_type(docks, offsets));
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: econ_common::dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let jet = || {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            tech_level: Some(1),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            cost: Some(1000),
            primary: Some("Agm".into()),
            ..Default::default()
        };
        u.combat.ammo = Some(2);
        u.econ.landable = Some(true);
        u.econ.dock = Some(vec!["APAD".into()]);
        u
    };
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), jet());
    let mut bound = jet();
    bound.econ.airport_bound = Some(true);
    push(&mut r, "aircraft", "BOUND");
    r.sim.aircraft.insert("BOUND".into(), bound);
    r
}

/// One docking place whose displacement is zero: its docking position is
/// the foundation centre (S-0404 claim 15).
fn rules() -> Ruleset {
    rules_with(1, Some(vec![[0, 0, 0]]))
}

/// AC31 part h (1)'s displacement (test value): from the 2 × 2 foundation's
/// centre to the centre of the cell two cells east of its east column, in
/// its south row, at height 0.
const OFF_PAD: [i32; 3] = [640, 128, 0];

/// `r` with claim 7's two switches at `untested` and `drop`.
fn with(mut r: Ruleset, untested: bool, drop: bool) -> Ruleset {
    r.globals.quirks.aircraft_descent_touchdown_untested = untested;
    r.globals.quirks.aircraft_refused_kept_drop = drop;
    r
}

fn put(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn tank(w: &mut World, c: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "TANK", 0, c, 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} beh={:?}/{:?} last={:?} dest={:?} contact={:?} ammo={} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
        o.combat.ammo,
        o.air
    )
}

fn foundation(w: &World, k: ObjectId) -> Vec<CellXY> {
    let b = w.obj(k).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
}

/// Whether cell `c` is cell `d` or one of its eight neighbours.
fn near(c: CellXY, d: CellXY) -> bool {
    (c.x - d.x).abs() <= 1 && (c.y - d.y).abs() <= 1
}

fn service(w: &World, k: ObjectId) -> Option<i32> {
    w.obj(k).unwrap().air.as_ref().and_then(|s| s.service)
}

fn one(w: &World) -> F64 {
    F64::from_i64(1, w.rules().mode)
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

/// Steps until the next processing of `a`; returns its tick and delay.
fn next_processing(w: &mut World, a: ObjectId) -> (i32, i32) {
    let before = w.obj(a).unwrap().beh.last;
    for _ in 0..50 {
        step(w);
        let last = w.obj(a).unwrap().beh.last;
        if last != before {
            return last.unwrap();
        }
    }
    panic!("no processing: {}", describe(w, a));
}

/// A JET of house 0 flying east at full speed in flight over open ground,
/// toward a cell 12 cells east of where it started; `pad`: a pad APAD of
/// house 0 two cells west and eight south of the middle. Returns (world,
/// JET, the pad).
fn cruising(r: &Ruleset, pad: bool) -> (World, ObjectId, Option<ObjectId>) {
    let mut w = game(r);
    let k = pad.then(|| building(&mut w, "APAD", 0, -2, 8));
    ready(&mut w);
    let a = put(&mut w, "JET", -8, 0);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    order_move(&mut w, a, CellXY::new(c.x + 12, c.y));
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && h == s.held && s.cur.eq_num(one(&w)) {
            return (w, a, k);
        }
    }
    panic!("never at full speed: {}", describe(&w, a));
}

/// [`cruising`] with no pad, then a stop order; runs the order's tick.
/// Returns (world, JET, *C*: the cell it is over at the end of the order's
/// tick).
fn stopped(r: &Ruleset) -> (World, ObjectId, CellXY) {
    let (mut w, a, _) = cruising(r, false);
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    (w, a, c)
}

/// Steps until the first tick at whose end the landing of `a` is under way.
fn to_landing_start(w: &mut World, a: ObjectId) {
    for _ in 0..800 {
        step(w);
        if st(w, a).landing {
            return;
        }
    }
    panic!("no landing: {}", describe(w, a));
}

/// Steps until the tick at whose end `a` has touched down: it was landing at
/// the end of the tick before and is now at height 0 and not landing.
/// Asserts that its height never rises on the way. Returns that tick.
fn to_touchdown(w: &mut World, a: ObjectId) -> i32 {
    let mut h = air::height(w, a);
    for _ in 0..800 {
        let was = st(w, a);
        step(w);
        let nh = air::height(w, a);
        assert!(nh <= h, "no rise while it lands: {}", describe(w, a));
        h = nh;
        if was.landing && !st(w, a).landing && nh <= 0 {
            return w.tick - 1;
        }
    }
    panic!("no touchdown: {}", describe(w, a));
}

/// Whether the landing of `a` was refused in the tick just run: it was
/// landing at the end of the tick before and is now neither landing nor on
/// the ground, above height 0.
fn refused(was: &air::AirState, w: &World, a: ObjectId) -> bool {
    let s = st(w, a);
    was.landing && !s.landing && !s.ground && air::height(w, a) > 0
}

/// A JET landing where it is (a stop order at full speed), then given by
/// hand, in *sleep* so that no processing gives it another, the pad as
/// destination and flight point, with no contact. The pad's cell holds a
/// building that is not its contact, so a touchdown outside that cell
/// forgets nothing (claim 7). Returns (world, JET, pad) at the end of the
/// landing start's tick.
fn landing_toward_pad(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let (mut w, a, k) = cruising(r, true);
    let k = k.unwrap();
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    to_landing_start(&mut w, a);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.beh.force(Behaviour::Sleep, t, true);
    o.dest = Some(Target::Object(k));
    let s = o.air.as_mut().unwrap();
    s.flying_to = Some(Target::Object(k));
    s.own_point = false;
    (w, a, k)
}

/// From an aircraft on the ground by its height that holds its destination
/// `d` and its flight point, not landing: steps until its first movement
/// update above height 0. Checks that each update on the ground keeps it
/// there, not landing and not settled, with `d`, and moves it by claim 4's
/// forward step alone while its facing is not turning, and that it climbs
/// only in the update after one that left its target fraction at 1.
/// Returns the number of updates it spent on the ground.
fn slides_then_climbs(w: &mut World, a: ObjectId, d: Target) -> usize {
    let mode = w.rules().mode;
    let stall = w.rules().quirks.aircraft_unit_step_stall;
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    for on_ground in 0..200 {
        let o = w.obj(a).unwrap().clone();
        let was = o.air.clone().unwrap();
        let turning = o.body.turning(w.tick) || o.body.turning(w.tick + 1);
        let f = o.body.value(w.tick);
        step(w);
        let s = st(w, a);
        if air::height(w, a) > 0 {
            assert!(
                was.tgt.eq_num(one(w)),
                "it climbs only in the update after one that left its target fraction at 1: {}",
                describe(w, a)
            );
            assert!(!s.sliding && s.held > 0, "{}", describe(w, a));
            assert!(
                !s.landing && !s.taking_off,
                "ordinary flight: {}",
                describe(w, a)
            );
            return on_ground;
        }
        assert!(
            s.sliding && !s.landing && !s.ground && s.held == 0,
            "on the ground by its height, not landing, held height 0: {}",
            describe(w, a)
        );
        assert_eq!(w.obj(a).unwrap().dest, Some(d), "{}", describe(w, a));
        if !turning {
            let p = w.obj(a).unwrap().pos;
            let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, was.cur, stall, mode);
            assert_eq!(
                (p.x, p.y),
                (nx, ny),
                "claim 4's forward step alone: {}",
                describe(w, a)
            );
        }
    }
    panic!("it never climbs: {}", describe(w, a));
}

// ---------------------------------------------------------------------------
// Claim 7: the untested touchdown and the flight on the ground after it

// spec: S-0501 claim 7 (r132: claim 3's descent brings every landing to height 0 before its landing step, so no touchdown is tested: no cell test and none of an accepted touchdown's effects, the forget of the destination in any case included; a touchdown its own forget test does not make forget leaves it on the ground by its height with its destination and its flight point, and it goes on as after a refusal that kept the destination; every touchdown ends tracking, claim 47 (b)) and Edge cases (quirks.aircraft_descent_touchdown_untested on; quirks.aircraft_refused_kept_drop on: it slides on, its fraction following claim 5, and at the start of the update after the first that leaves its target fraction at 1 it takes the held height of ordinary flight, claim 48 (b)'s third case, and climbs in that update; off: the held height of ordinary flight, so it climbs from its next update on)
#[test]
fn claim_7_an_untested_touchdown_that_keeps_the_destination_slides_then_climbs() {
    for drop in [true, false] {
        let r = with(rules(), true, drop);
        let (mut w, a, k) = landing_toward_pad(&r);
        let u = to_touchdown(&mut w, a);
        let o = w.obj(a).unwrap();
        let fc = air::flight_point(&w, a, Target::Object(k)).cell();
        assert_ne!(o.pos.cell(), fc, "outside the flight point's cell");
        assert!(!foundation(&w, k).contains(&o.pos.cell()));
        assert_eq!(
            o.dest,
            Some(Target::Object(k)),
            "the destination kept: {}",
            describe(&w, a)
        );
        let s = st(&w, a);
        assert_eq!(
            s.flying_to,
            Some(Target::Object(k)),
            "the flight point kept"
        );
        assert!(
            !s.ground && !s.landing && !s.taking_off && s.held == 0,
            "on the ground by its height, not settled: {}",
            describe(&w, a)
        );
        assert_eq!(air::height(&w, a), 0, "no rise of 10 leptons");
        assert_eq!(s.mark, None, "no object-present flag while it moves");
        assert!(
            !glasnost_sim::combat::flying::tracked(&w, a),
            "the touchdown ends its tracking"
        );
        assert_eq!(s.sliding, drop, "{}", describe(&w, a));
        if drop {
            let n = slides_then_climbs(&mut w, a, Target::Object(k));
            assert!(n >= 1, "at least one update on the ground");
        } else {
            step(&mut w);
            assert!(
                air::height(&w, a) > 0 && st(&w, a).held > 0,
                "off: it climbs from its next update on: {}",
                describe(&w, a)
            );
        }
        // Guidance and claim 6's triggers apply again once above the terrain.
        let _ = u;
    }
}

// spec: S-0501 Edge cases (quirks.aircraft_descent_touchdown_untested off: every touchdown is tested and, when accepted, has the accepted touchdown's effects, the forget of the destination in any case included) and claim 7 (an accepted touchdown forgets the destination in that same movement update)
#[test]
fn claim_7_off_a_tested_touchdown_on_open_ground_forgets_the_destination_in_any_case() {
    let r = with(rules(), false, true);
    let (mut w, a, k) = landing_toward_pad(&r);
    to_touchdown(&mut w, a);
    let o = w.obj(a).unwrap();
    let fc = air::flight_point(&w, a, Target::Object(k)).cell();
    assert_ne!(o.pos.cell(), fc);
    assert_eq!(o.dest, None, "forgotten in any case: {}", describe(&w, a));
    let s = st(&w, a);
    assert!(s.ground && !s.sliding, "settled: {}", describe(&w, a));
    assert_eq!(s.flying_to, None);
    assert!(s.mark.is_some(), "its object-present flag at rest");
}

/// A JET given a move to a cell 6 cells east; once the move's selection is
/// made, a TANK stands in that cell. Returns (world, JET, the cell).
fn landing_on_an_occupant(r: &Ruleset) -> (World, ObjectId, CellXY) {
    let mut w = game(r);
    ready(&mut w);
    let a = put(&mut w, "JET", -8, 0);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    let g = CellXY::new(c.x + 6, c.y);
    order_move(&mut w, a, g);
    for _ in 0..50 {
        step(&mut w);
        if st(&w, a).regive.is_none() && st(&w, a).first_moved.is_some() {
            break;
        }
    }
    tank(&mut w, g);
    (w, a, g)
}

// spec: S-0501 claim 7 (r132: no touchdown is tested, so an aircraft can touch down in a cell that F7 refuses; the touchdown's own forget test, in the flight point's cell, forgets the destination and the flight point, and it rests there) and claim 45 (r132: the same at the edge cell) and Edge cases (quirks.aircraft_descent_touchdown_untested off: F7 refuses the occupied cell, and it rises by 10 leptons); AC4 (F7 is judged at claim 40's choice, not at a touchdown)
#[test]
fn claim_7_an_untested_touchdown_in_an_occupied_cell_rests_there() {
    for untested in [true, false] {
        let r = with(rules(), untested, true);
        let (mut w, a, g) = landing_on_an_occupant(&r);
        let mut was = st(&w, a);
        let mut rose = false;
        for _ in 0..1500 {
            step(&mut w);
            if refused(&was, &w, a) {
                rose = true;
                break;
            }
            if st(&w, a).ground {
                break;
            }
            was = st(&w, a);
        }
        let o = w.obj(a).unwrap();
        assert_eq!(o.pos.cell(), g, "{}", describe(&w, a));
        assert_eq!(o.dest, None, "the destination forgotten");
        if untested {
            assert!(!rose, "no refusal: {}", describe(&w, a));
            let s = st(&w, a);
            assert!(s.ground && air::height(&w, a) == 0, "{}", describe(&w, a));
            assert_eq!(s.flying_to, None);
        } else {
            assert!(rose, "off: the refusal: {}", describe(&w, a));
            assert_eq!(air::height(&w, a), 10, "the refusal's rise");
        }
    }
}

// spec: S-0501 claim 6 (r132: an aircraft that claim 7 leaves with nothing to move it 10 leptons above a refused cell, which comes only with claim 7's quirks.aircraft_descent_touchdown_untested off, is not moving: being given no destination changes nothing for it, and it goes on holding 10 leptons above that cell) and claim 7 (it stays there until it gets a destination); intake 96's seam (a clear given during the hold)
#[test]
fn claim_6_a_stop_during_claim_7s_hold_changes_nothing() {
    let r = with(rules(), false, true);
    let (mut w, a, g) = landing_on_an_occupant(&r);
    let mut was = st(&w, a);
    let mut held = false;
    for _ in 0..1500 {
        step(&mut w);
        if refused(&was, &w, a) {
            held = true;
            break;
        }
        was = st(&w, a);
    }
    assert!(held, "the refusal: {}", describe(&w, a));
    step(&mut w);
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    // The stop's own tick and the next movement update: a later processing
    // of its behaviour may give it a destination, which the spec allows.
    for _ in 0..2 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        assert_eq!(o.pos.cell(), g, "{}", describe(&w, a));
        assert_eq!(
            air::height(&w, a),
            10,
            "held 10 leptons above: {}",
            describe(&w, a)
        );
        assert_eq!(s.flying_to, None, "no flight point: {}", describe(&w, a));
        assert!(!s.landing && !s.ground && !s.sliding);
        assert_eq!(o.dest, None);
    }
}

/// [`landing_toward_pad`] with a TANK in the cell where it would touch
/// down: with `quirks.aircraft_descent_touchdown_untested` off, F7 refuses
/// the touchdown, which keeps the destination (claim 7). Runs the refusal's
/// tick. Returns (world, JET, pad).
fn refused_and_kept(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let (mut w, a, k) = landing_toward_pad(r);
    let mut copy = w.clone();
    to_touchdown(&mut copy, a);
    let p = copy.obj(a).unwrap().pos.cell();
    tank(&mut w, p);
    let mut was = st(&w, a);
    for _ in 0..800 {
        step(&mut w);
        if refused(&was, &w, a) {
            let o = w.obj(a).unwrap();
            assert_eq!(o.pos.cell(), p);
            assert_eq!(o.dest, Some(Target::Object(k)), "kept: {}", describe(&w, a));
            return (w, a, k);
        }
        was = st(&w, a);
    }
    panic!("no refusal: {}", describe(&w, a));
}

// spec: S-0501 claim 7 (r132, Q-2578: after a refusal that kept the destination, the next update's descent toward the landing's held height of 0 brings it back to height 0 before that update's take-off step ends the take-off; that drop is no touchdown; it is then on the ground by its height, not landing, with its destination, its flight point and the held height 0, makes claim 4's forward step with its fraction following claim 5 and no climb or guidance, and at the start of the update after the first that leaves its target fraction at 1 takes the held height of ordinary flight and climbs in that update) and Edge cases (quirks.aircraft_refused_kept_drop on; off: the held height of ordinary flight, so it climbs from its next update on)
#[test]
fn claim_7_a_refusal_that_kept_the_destination_drops_back_to_the_ground() {
    for drop in [true, false] {
        let r = with(rules(), false, drop);
        let (mut w, a, k) = refused_and_kept(&r);
        assert_eq!(air::height(&w, a), 10);
        assert!(st(&w, a).taking_off);
        step(&mut w);
        let s = st(&w, a);
        assert!(
            !s.taking_off && !s.landing && !s.ground,
            "{}",
            describe(&w, a)
        );
        assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(k)));
        if drop {
            assert_eq!(air::height(&w, a), 0, "the drop: {}", describe(&w, a));
            assert!(s.sliding && s.held == 0, "{}", describe(&w, a));
            slides_then_climbs(&mut w, a, Target::Object(k));
        } else {
            assert!(
                air::height(&w, a) > 10 && s.held > 0 && !s.sliding,
                "off: it climbs from the next update: {}",
                describe(&w, a)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 7: the landing's check of its flight point's cell

/// A `name` aircraft of house 0, landing by hand from 200 leptons above a
/// cell of the pad's foundation toward the pad, its contact, whose docking
/// position lies two cells east of the foundation (AC31 part h (1)'s pad),
/// in *sleep*. Returns (world, pad, aircraft).
fn landing_over_the_foundation(name: &str) -> (World, ObjectId, ObjectId) {
    let r = rules_with(1, Some(vec![OFF_PAD]));
    let mut w = game(&r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, name, 8, 0);
    step(&mut w);
    let fc = foundation(&w, k)[0];
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.pos.z += 200;
    }
    air::sync_listing(&mut w, a);
    let centre = Pos::centre(fc, 0);
    let z = w.surface_at(centre.x, centre.y) + 200;
    {
        let o = w.obj_mut(a).unwrap();
        o.pos = Pos::new(centre.x, centre.y, z);
        o.beh.force(Behaviour::Sleep, t, true);
        o.dest = Some(Target::Object(k));
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
        let s = o.air.as_mut().unwrap();
        s.ground = false;
        s.landing = true;
        s.taking_off = false;
        s.held = 0;
        s.flying_to = Some(Target::Object(k));
        s.own_point = false;
        s.mark = None;
    }
    (w, k, a)
}

// spec: S-0501 claim 7 (r132: before touchdown each movement update of a landing checks its flight point's cell: for a pad-bound aircraft it refuses a cell that holds no building it is in contact with, and a refused cell ends the landing with a take-off; for one that is not pad-bound it refuses none here) and claim 24 (r132: pad-bound and armed, a landing that starts over the foundation is ended by that check, its flight point's cell, the docking position's, holding no pad)
// design: S-0501 claim 7, Open question 28 and Q-2646 (the check at the start of the update, claim 8's take-off toward cruise height that climbs in that update, the destination and the flight point kept)
#[test]
fn claim_7_the_landing_check_refuses_a_pad_bound_flight_point_off_its_pad() {
    let (mut w, k, a) = landing_over_the_foundation("BOUND");
    let dc = air::flight_point(&w, a, Target::Object(k)).cell();
    assert!(
        !foundation(&w, k).contains(&dc),
        "the docking position off the pad"
    );
    let h = air::height(&w, a);
    step(&mut w);
    let s = st(&w, a);
    assert!(
        !s.landing && s.taking_off && s.held > 0,
        "the landing ended with a take-off: {}",
        describe(&w, a)
    );
    assert!(air::height(&w, a) > h, "it climbs in that update");
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert_eq!(o.contact, Some(k), "the contact kept");
    assert_eq!(s.flying_to, Some(Target::Object(k)));
    // Not pad-bound: the same landing goes on.
    let (mut w, _, a) = landing_over_the_foundation("JET");
    let h = air::height(&w, a);
    step(&mut w);
    let s = st(&w, a);
    assert!(s.landing && !s.taking_off, "{}", describe(&w, a));
    assert!(air::height(&w, a) < h);
}

// ---------------------------------------------------------------------------
// Claim 6: given no destination while landing (Q-2576); AC4 runs (4), (5)

// spec: S-0501 claim 6 (r132, Q-2576: a landing aircraft is always moving; given no destination it gets the same selection around the cell it is over, and when the selected cell is its flight point's cell its flight ignores the centre: its landing goes on with that flight point kept, and with no destination claim 4's descent move does not apply); AC4 same-cell part, run (4)
#[test]
fn ac4_run_4_given_no_destination_over_its_flight_points_cell_the_landing_goes_on() {
    let (mut w, a, c) = stopped(&rules());
    to_landing_start(&mut w, a);
    assert!(air::height(&w, a) > 0);
    assert_eq!(w.obj(a).unwrap().pos.cell(), c, "void: not over C");
    let fp = st(&w, a).flying_to;
    assert_eq!(fp, Some(Target::Cell(c)));
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    let mode = w.rules().mode;
    let stall = w.rules().quirks.aircraft_unit_step_stall;
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    let mut h = air::height(&w, a);
    let mut checked = 0;
    for _ in 0..400 {
        let o = w.obj(a).unwrap().clone();
        let was = o.air.clone().unwrap();
        let turning = o.body.turning(w.tick) || o.body.turning(w.tick + 1);
        let f = o.body.value(w.tick);
        step(&mut w);
        let s = st(&w, a);
        let nh = air::height(&w, a);
        assert!(!s.taking_off && nh <= h, "no take-off: {}", describe(&w, a));
        h = nh;
        if !turning {
            let p = w.obj(a).unwrap().pos;
            let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, was.cur, stall, mode);
            assert_eq!((p.x, p.y), (nx, ny), "the forward step alone");
            checked += 1;
        }
        if s.ground {
            break;
        }
        assert!(s.landing, "{}", describe(&w, a));
        assert_eq!(s.flying_to, fp, "the flight point kept");
        assert_eq!(w.obj(a).unwrap().dest, None);
    }
    assert!(st(&w, a).ground, "a touchdown: {}", describe(&w, a));
    assert!(checked > 0);
}

// spec: S-0501 claim 6 (r120, Q-2507: an aircraft in flight, moving and not landing, with attack current and no target: the cleared destination gives the flight an F8 landing spot as its flight point, with no destination object) and claim 41 (F8 always gives a cell); intake 96's seam
#[test]
fn claim_6_given_no_destination_in_flight_in_attack_with_no_target_takes_an_f8_spot() {
    let (mut w, a, _) = cruising(&rules(), false);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(Behaviour::Attack, t, true);
        o.target = None;
    }
    let spot = air::landing_spot(&mut w.clone(), a).unwrap();
    glasnost_sim::econ::assign_destination(&mut w, a, None);
    let s = st(&w, a);
    assert_eq!(s.flying_to, Some(Target::Cell(spot)), "{}", describe(&w, a));
    assert!(s.own_point && s.unfinished && !s.landing);
    assert_eq!(w.obj(a).unwrap().dest, None, "no destination object");
}

// spec: S-0501 claim 6 (r132: landing where it is, the descent does not steer and it slows along its heading, so along each map axis it touches down within claim 24's bound D of where its landing started, possibly in a cell next to the flight point's) and claim 7 (r132: that touchdown is untested; its own forget test forgets the destination and the flight point there, since for an aircraft with no contact a flight point's cell with no building counts as holding it, so it rests where it touched down); AC4 same-cell part
#[test]
fn ac4_a_touchdown_that_drifts_out_of_the_flight_points_cell_rests_there() {
    let (mut w, a, c) = stopped(&rules());
    to_landing_start(&mut w, a);
    assert_eq!(st(&w, a).flying_to, Some(Target::Cell(c)));
    assert_eq!(w.obj(a).unwrap().pos.cell(), c, "void: not over C");
    // Set-up: the landing under way at full speed fraction, at C's edge
    // along its heading, so that the descent's slowing forward steps carry
    // it out of C (here a stop's landing starts once it has slowed to 0).
    let mode = w.rules().mode;
    let stall = w.rules().quirks.aircraft_unit_step_stall;
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    let full = one(&w);
    {
        let t = w.tick;
        let o = w.obj_mut(a).unwrap();
        let f = o.body.value(t);
        let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, full, stall, mode);
        // spec: S-0204 claim 2 (a cell is 256 leptons wide)
        let edge = |p: i32, n: i32, c: i32| match (n - p).signum() {
            1 => c * 256 + 255,
            -1 => c * 256,
            _ => p,
        };
        (o.pos.x, o.pos.y) = (edge(o.pos.x, nx, c.x), edge(o.pos.y, ny, c.y));
        o.air.as_mut().unwrap().cur = full;
    }
    assert_eq!(w.obj(a).unwrap().pos.cell(), c);
    to_touchdown(&mut w, a);
    let o = w.obj(a).unwrap();
    let s = st(&w, a);
    assert_ne!(o.pos.cell(), c, "void: no drift: {}", describe(&w, a));
    assert!(s.ground && !s.sliding, "at rest: {}", describe(&w, a));
    assert_eq!(o.dest, None);
    assert_eq!(s.flying_to, None, "the flight point forgotten");
    assert_eq!(air::height(&w, a), 0);
}

// spec: S-0501 claim 6 (r132, Q-2576: given no destination while landing over a cell other than its flight point's, it takes off toward the centre of the cell the selection gives, handed to its flight only, not as a destination, and lands where it is there); AC4 same-cell part, run (5)
#[test]
fn ac4_run_5_given_no_destination_over_another_cell_it_takes_off_and_lands_there() {
    let (mut w, a, _) = stopped(&rules());
    to_landing_start(&mut w, a);
    assert!(air::height(&w, a) > 0);
    // Set-up: its flight point in the cell next to the one it is over, as
    // after a drift out of the flight point's cell (as in runs (2) and (3)).
    let over = w.obj(a).unwrap().pos.cell();
    let fc = CellXY::new(over.x, over.y + 1);
    w.obj_mut(a).unwrap().air.as_mut().unwrap().flying_to = Some(Target::Cell(fc));
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    step(&mut w);
    let s = st(&w, a);
    assert!(
        s.taking_off && !s.landing,
        "a take-off at the order: {}",
        describe(&w, a)
    );
    // The order is carried out after that tick's movement update, so the
    // climb comes at the next one.
    let h = air::height(&w, a);
    step(&mut w);
    assert!(air::height(&w, a) > h, "it climbs: {}", describe(&w, a));
    let s = st(&w, a);
    assert_eq!(s.flying_to, Some(Target::Cell(over)), "C′'s centre");
    assert!(s.own_point);
    assert_eq!(w.obj(a).unwrap().dest, None, "not given as a destination");
    to_landing_start(&mut w, a);
    assert_eq!(
        w.obj(a).unwrap().pos.cell(),
        over,
        "its next landing starts in C′: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 6 (r132, Q-2576: with attack current and no target, the F8 spot is the point handed to its flight, and its cell is the one compared with its flight point's) and claim 41 (F8 always gives a cell)
#[test]
fn claim_6_given_no_destination_while_landing_in_attack_with_no_target_takes_an_f8_spot() {
    let (mut w, a, c) = stopped(&rules());
    to_landing_start(&mut w, a);
    let t = w.tick;
    // Its flight point several cells away, with attack current and no
    // target, then the clear (S-0213's assignment, as an order gives it).
    let far = CellXY::new(c.x, c.y - 4);
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(Behaviour::Attack, t, true);
        o.target = None;
        o.dest = Some(Target::Cell(far));
        let s = o.air.as_mut().unwrap();
        s.flying_to = Some(Target::Cell(far));
        s.own_point = false;
    }
    let spot = air::landing_spot(&mut w.clone(), a).unwrap();
    glasnost_sim::econ::assign_destination(&mut w, a, None);
    let s = st(&w, a);
    if spot == far {
        assert!(s.landing && s.flying_to == Some(Target::Cell(far)));
    } else {
        assert_eq!(s.flying_to, Some(Target::Cell(spot)), "{}", describe(&w, a));
        assert!(
            s.own_point && s.taking_off && !s.landing,
            "{}",
            describe(&w, a)
        );
    }
    assert_eq!(w.obj(a).unwrap().dest, None);
}

// ---------------------------------------------------------------------------
// Claim 6: over a cell outside the usable area (Q-2577)

/// A new game like `game` whose usable area lies two cells in from the
/// legal map's edge (S-0009 claims 1 and 4).
fn banded_game(r: &Ruleset) -> World {
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<_> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(cell), Some(old)) = (cells.get_mut(c), m.cells.get(c)) {
            *cell = *old;
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic test game loads");
    ready(&mut w);
    w
}

/// S-0522 F2 written out here: the cell of local point (a, b).
fn f2(w: &World, a: i32, b: i32) -> CellXY {
    let fp = w.cells.footprint();
    let [lx, ly, _, _] = fp.local;
    let (x, y) = (a + lx, b + ly);
    CellXY::new((y + 1).div_euclid(2) + x, fp.width + y.div_euclid(2) - x)
}

/// Claim 6's start (Q-2577) for cell `c`, found by searching F2 for the
/// local point whose cell is `c` rather than inverting it.
fn expected_start(w: &World, c: CellXY) -> CellXY {
    let [_, _, lw, lh] = w.cells.footprint().local;
    let span = 4 * (SIZE + lw + lh);
    let mut found = None;
    for a in -span..=span {
        for b in -span..=span {
            if f2(w, a, b) == c {
                assert!(found.is_none(), "F2 is one-to-one");
                found = Some((a, b));
            }
        }
    }
    let (mut a, mut b) = found.expect("a local point");
    let (ca, cb) = (lw.div_euclid(2), lh.div_euclid(2));
    if (a - ca).abs() < (b - cb).abs() {
        a = if a < ca { 1 } else { lw - 1 };
    } else {
        b = if b < cb { 1 } else { lh - 1 };
    }
    f2(w, a, b)
}

// spec: S-0501 claim 6 (r132, Q-2577: over a cell outside the usable area by its height-aware test, for an aircraft that is not visiting and may not leave the map now, the selection starts from the cell of the local point that keeps one coordinate and moves the other to 1 or to one less than the local area's size along it, the coordinate nearer the centre, ties to the row, rows counted in units of Lh; F7 refuses a start outside the usable area, so the replacement search runs, and when every radius fails the flight point is the start cell's centre) and Edge cases (quirks.aircraft_offmap_stop_start on; off: the selection starts from the cell it is over); S-0522 F2
#[test]
fn claim_6_given_no_destination_outside_the_usable_area_starts_from_the_local_edge() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_offmap_stop_start = on;
        let mut w = banded_game(&r);
        let a = put(&mut w, "JET", -8, 0);
        step(&mut w);
        let c0 = w.obj(a).unwrap().pos.cell();
        order_move(&mut w, a, CellXY::new(c0.x + 6, c0.y));
        let mut flying = false;
        for _ in 0..600 {
            step(&mut w);
            if air::height(&w, a) > 0 && !st(&w, a).taking_off {
                flying = true;
                break;
            }
        }
        assert!(flying, "{}", describe(&w, a));
        let cells: Vec<CellXY> = (0..3 * SIZE)
            .flat_map(|x| (0..3 * SIZE).map(move |y| CellXY::new(x, y)))
            .filter(|&c| w.cells.cell(c).is_some() && !w.cells.usable(c))
            .step_by(3)
            .collect();
        assert!(cells.len() > 20, "a band of unusable cells");
        let mut moved = 0;
        for c in cells {
            let mut v = w.clone();
            let z = v.obj(a).unwrap().pos.z;
            let p = Pos::centre(c, 0);
            v.obj_mut(a).unwrap().pos = Pos::new(p.x, p.y, z);
            assert!(air::height(&v, a) > 0);
            let start = if on { expected_start(&v, c) } else { c };
            moved += usize::from(start != c);
            let want = if v.cells.usable(start) {
                start
            } else {
                air::replacement_search(&mut v.clone(), a, start).unwrap_or(start)
            };
            glasnost_sim::econ::assign_destination(&mut v, a, None);
            let s = st(&v, a);
            assert_eq!(
                s.flying_to,
                Some(Target::Cell(want)),
                "over {c:?}, start {start:?}: {}",
                describe(&v, a)
            );
            assert!(s.own_point);
        }
        assert_eq!(moved > 0, on, "the start moves only with the switch on");
    }
}

// ---------------------------------------------------------------------------
// Claim 24: the entry restarted in the docking stage (Q-2579); AC31 h (4)

/// AC31 part h (1)'s pad and a JET in contact with it, in *enter*,
/// approaching with its ammunition spent. `r` sets the switches. Returns
/// (world, pad, JET).
fn approaching(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, "JET", 8, 0);
    step(&mut w);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    glasnost_sim::process::fall_back(&mut w, a, false);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
    next_processing(&mut w, a);
    next_processing(&mut w, a);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    (w, k, a)
}

// spec: S-0501 claim 24 (r132, Q-2579: with a behaviour queued, the pad's answer to the second rule's request starts entry again within that processing: it makes entry's beginning with its 1-tick delay, puts the aircraft back in the approaching stage and leaves it ready to change, and it neither reports entry, nor makes a docking step, nor falls back; its next processing, 1 tick later, is the approaching stage's) and Edge cases (quirks.aircraft_descent_touchdown_untested off: the refused touchdown that forgot the destination in the docking position's cell)
#[test]
fn claim_24_an_entry_restarted_in_the_docking_stage_makes_its_beginning_at_once() {
    let r = with(rules_with(1, Some(vec![OFF_PAD])), false, true);
    let (mut w, k, a) = approaching(&r);
    let dc = glasnost_sim::econ::release::pad_position(&w, k, a).0.cell();
    tank(&mut w, dc);
    let mut was = st(&w, a);
    let mut u = false;
    for _ in 0..2000 {
        step(&mut w);
        if refused(&was, &w, a) {
            u = true;
            break;
        }
        was = st(&w, a);
    }
    assert!(u, "a refused touchdown: {}", describe(&w, a));
    assert!(
        st(&w, a).enter_docking,
        "in the docking stage: {}",
        describe(&w, a)
    );
    assert_eq!(w.obj(a).unwrap().dest, None);
    let start0 = w.obj(a).unwrap().beh.start_tick;
    // A behaviour queued while it is not ready to change (set by hand).
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.queued = Some(Behaviour::Guard);
        o.air.as_mut().unwrap().mark_cleared = true;
    }
    let (r_t, d) = next_processing(&mut w, a);
    assert_eq!(d, 1, "entry's beginning: 1 tick");
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
    assert_ne!(o.beh.start_tick, start0, "entry started again");
    assert_eq!(o.beh.queued, None, "the queued behaviour replaced");
    assert_eq!(o.dest, Some(Target::Object(k)), "the pad given back");
    let s = st(&w, a);
    assert!(!s.enter_docking, "back in the approaching stage");
    assert_eq!(s.enter_of, Some(o.beh.start_tick), "the beginning made");
    assert!(!s.mark_cleared, "left ready to change");
    assert_eq!(service(&w, k), None, "no entry report");
    let (r2, d2) = next_processing(&mut w, a);
    assert_eq!(r2, r_t + 1, "its next processing 1 tick later");
    assert_eq!(d2, 3, "the approaching stage's processing in contact");
}

// spec: S-0501 claim 24 (r132: a touchdown beside the docking position, untested, keeps the pad as destination, and a later processing in the docking stage that finds it on the ground and not at rest reports entry as in flight, Q-2649 interim) and claim 7 (r132: after it the aircraft goes on as after a refusal that kept the destination) and Edge cases (quirks.aircraft_descent_touchdown_untested on; quirks.aircraft_refused_kept_drop in both positions); AC31 part h (1): whether either position comes to rest
#[test]
fn ac31h1_after_an_untested_touchdown_beside_the_docking_position_it_comes_to_rest() {
    for drop in [true, false] {
        let (mut w, _, a) = approaching(&with(rules_with(1, Some(vec![OFF_PAD])), true, drop));
        let mut was = st(&w, a);
        let mut touchdowns = 0;
        let mut rest = None;
        for _ in 0..2000 {
            step(&mut w);
            let s = st(&w, a);
            if was.landing && !s.landing && air::height(&w, a) == 0 {
                touchdowns += 1;
            }
            if s.ground && air::height(&w, a) == 0 {
                rest = Some(w.tick);
                break;
            }
            was = s;
        }
        assert!(
            rest.is_some(),
            "drop {drop}: at rest within 2000 ticks after {touchdowns} touchdowns: {}",
            describe(&w, a)
        );
        assert!(touchdowns >= 1, "drop {drop}: {}", describe(&w, a));
    }
}

// spec: S-0501 claim 24 (r132: a landing toward a docking position off the foundation goes on to its touchdown, since the landing's check of its flight point's cell refuses no occupied cell to an aircraft that is not pad-bound; untested, no touchdown rises by 10 leptons; after one in the docking position's cell, which the vehicle holds, it has no destination by claim 7's own forget test, after one in a cell next to it the pad as destination) and claim 7; AC31 part h (4)
#[test]
fn ac31h4_an_occupant_at_the_docking_position() {
    let (mut w, k, a) = approaching(&rules_with(1, Some(vec![OFF_PAD])));
    let dc = glasnost_sim::econ::release::pad_position(&w, k, a).0.cell();
    assert!(!foundation(&w, k).contains(&dc));
    tank(&mut w, dc);
    to_landing_start(&mut w, a);
    let mut h = air::height(&w, a);
    let mut u = None;
    for _ in 0..600 {
        let was = st(&w, a);
        step(&mut w);
        let nh = air::height(&w, a);
        assert!(nh <= h, "no rise: {}", describe(&w, a));
        h = nh;
        if was.landing && !st(&w, a).landing {
            u = Some(w.tick - 1);
            break;
        }
        assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(k)));
    }
    assert!(
        u.is_some(),
        "a touchdown within 600 ticks: {}",
        describe(&w, a)
    );
    let o = w.obj(a).unwrap();
    assert_eq!(air::height(&w, a), 0, "no rise of 10 leptons");
    assert_eq!(o.contact, Some(k));
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    let c = o.pos.cell();
    assert!(
        near(c, dc) && !foundation(&w, k).contains(&c),
        "{}",
        describe(&w, a)
    );
    if c == dc {
        assert_eq!(o.dest, None, "claim 7's own forget test");
    } else {
        assert_eq!(o.dest, Some(Target::Object(k)), "kept: {}", describe(&w, a));
    }
}
