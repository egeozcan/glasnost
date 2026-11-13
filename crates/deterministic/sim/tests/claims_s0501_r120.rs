//! Claim tests for S-0501 r120 (Q-2506, Q-2507, Q-2508): landing where it
//! is (claim 6) and AC4's same-cell runs, claim 8's exception tested at
//! hand-over against the flight point's cell, claim 7's forget before the
//! cell test and the hold above a refused cell, the entry report in flight
//! after a refused touchdown in contact (AC31 part h (4)), the run-away
//! notices' landing spots (AC31 part h (5)) and claim 29's unarmed
//! aircraft at cruise height. Synthetic data through the real tick path;
//! every type, weapon and number is invented.

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
    // A build-up sprite, so that a sell order sells it (S-0408 claim 4).
    pad.econ.buildup = Some(true);
    pad.econ.buildup_frames = Some(10);
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(docks);
    pad.econ.docking_offsets = offsets;
    pad
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with `docks` docking places and the displacements `offsets`, a 2 × 2
/// foundation); JET (armed, landable, not pad-bound, 2 rounds of the weapon
/// Agm, docking at APAD); LIFT (as JET, unarmed). TANK, a ground vehicle of
/// the test house, is an occupant F7 refuses.
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
    let mut lift = jet();
    lift.primary = None;
    lift.combat.ammo = None;
    push(&mut r, "aircraft", "LIFT");
    r.sim.aircraft.insert("LIFT".into(), lift);
    r
}

/// One docking place whose displacement is zero: its docking position is
/// the foundation centre (S-0404 claim 15).
fn rules() -> Ruleset {
    rules_with(1, Some(vec![[0, 0, 0]]))
}

/// `r` with claim 7's tested touchdown: r120's refusals.
// spec: S-0501 Edge cases (r132: quirks.aircraft_descent_touchdown_untested off: every touchdown is tested, so claim 7's refusal, claim 24's refused touchdown in contact and claim 45's arise wherever F7 refuses the cell; on, the untested touchdown is claims_s0501_r132's)
fn tested(mut r: Ruleset) -> Ruleset {
    r.globals.quirks.aircraft_descent_touchdown_untested = false;
    r
}

/// AC31 part h (1)'s displacement (test value): from the 2 × 2 foundation's
/// centre to the centre of the cell two cells east of its east column, in
/// its south row, at height 0.
const OFF_PAD: [i32; 3] = [640, 128, 0];

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
/// toward a cell 12 cells east of where it started. Returns (world, JET).
fn cruising() -> (World, ObjectId) {
    let mut w = game(&rules());
    ready(&mut w);
    let a = put(&mut w, "JET", -8, 0);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    order_move(&mut w, a, CellXY::new(c.x + 12, c.y));
    let one = F64::from_i64(1, w.rules().mode);
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && h == s.held && s.cur.eq_num(one) {
            return (w, a);
        }
    }
    panic!("never at full speed: {}", describe(&w, a));
}

/// [`cruising`], then a stop order; runs the order's tick. Returns (world,
/// JET, *C*: the cell it is over at the end of the order's tick).
fn stopped() -> (World, ObjectId, CellXY) {
    let (mut w, a) = cruising();
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

/// Whether the landing of `a` was refused in the tick just run: it was
/// landing at the end of the tick before and is now neither landing nor on
/// the ground, above height 0.
fn refused(was: &air::AirState, w: &World, a: ObjectId) -> bool {
    let s = st(w, a);
    was.landing && !s.landing && !s.ground && air::height(w, a) > 0
}

// ---------------------------------------------------------------------------
// Claim 6: landing where it is; AC4's same-cell runs

// spec: S-0501 claim 6 (r120, Q-2507: an aircraft in flight, moving and not landing that is given no destination gets a new flight point with no destination object: the centre of the cell claim 40's selection gives around the cell it is over, that cell itself when F7 and the flight-reservation test accept it; it flies to that point, its landing starts in that cell, its body is not turned toward the point, and it touches down no farther than claim 24's bound; r183, Q-3041: claim 4's descent move needs a flight point, not a destination, so it draws the descent toward that centre) and claim 8 (r120: setting no destination while it is in flight, moving and not landing makes it land where it is); AC4 same-cell part, run (1)
#[test]
fn ac4_run_1_landing_where_it_is_starts_in_the_cell_and_does_not_steer() {
    let (mut w, a, c) = stopped();
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, None, "no destination object: {}", describe(&w, a));
    let s = st(&w, a);
    assert_eq!(
        s.flying_to,
        Some(Target::Cell(c)),
        "the flight point: C's centre"
    );
    assert!(s.own_point && s.unfinished, "{}", describe(&w, a));
    to_landing_start(&mut w, a);
    assert_eq!(
        w.obj(a).unwrap().pos.cell(),
        c,
        "its landing starts in C: {}",
        describe(&w, a)
    );
    // From the landing start to touchdown: claim 4's forward step, then, in
    // a tick in which it descends, claim 4's descent move of 5 leptons
    // toward C's centre (r183, Q-3041: it needs a flight point, not a
    // destination).
    let mode = w.rules().mode;
    let stall = w.rules().quirks.aircraft_unit_step_stall;
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    let centre = glasnost_sim::world::Pos::centre(c, 0);
    let dist = |x: i32, y: i32| {
        (i64::from(x - centre.x).pow(2) + i64::from(y - centre.y).pow(2)).isqrt()
    };
    let mut checked = 0;
    let mut nudged = 0;
    for _ in 0..400 {
        let o = w.obj(a).unwrap().clone();
        let was = o.air.clone().unwrap();
        let h0 = air::height(&w, a);
        let turning = o.body.turning(w.tick) || o.body.turning(w.tick + 1);
        let f = o.body.value(w.tick);
        step(&mut w);
        let p = w.obj(a).unwrap().pos;
        if !turning {
            let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, was.cur, stall, mode);
            let off = (i64::from(p.x - nx).pow(2) + i64::from(p.y - ny).pow(2)).isqrt();
            if air::height(&w, a) < h0 && dist(nx, ny) > 5 {
                // spec: S-0501 Constants (descent horizontal nudge 5 leptons; table trigonometry and truncation, so at least 4)
                assert!(
                    (4..=5).contains(&off) && dist(p.x, p.y) < dist(nx, ny),
                    "the forward step and the descent move toward C's centre: {}",
                    describe(&w, a)
                );
                nudged += 1;
            } else if air::height(&w, a) >= h0 {
                assert_eq!(off, 0, "the forward step alone: {}", describe(&w, a));
            }
            checked += 1;
        }
        if st(&w, a).ground {
            break;
        }
    }
    let s = st(&w, a);
    assert!(s.ground, "a touchdown: {}", describe(&w, a));
    assert!(checked > 0 && nudged > 0, "{checked} {nudged}");
    assert!(
        near(w.obj(a).unwrap().pos.cell(), c),
        "in C or a cell next to it: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 6 (r120, Q-2507: the selection takes F7 and the flight-reservation test on the cell it is over; refused, claim 40's replacement search around it with that search's direction draws, one U(0, 7) draw per radius tried)
#[test]
fn claim_6_a_refused_cell_takes_the_replacement_search() {
    let (mut w, a) = cruising();
    let b = put(&mut w, "JET", -8, 8);
    // The order's tick must not carry it into the next cell.
    let mode = w.rules().mode;
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    for _ in 0..40 {
        let o = w.obj(a).unwrap().clone();
        let f = o.body.value(w.tick);
        let stall = w.rules().quirks.aircraft_unit_step_stall;
        let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, st(&w, a).cur, stall, mode);
        if Pos::new(nx, ny, 0).cell() == o.pos.cell() {
            break;
        }
        step(&mut w);
    }
    // A second aircraft whose destination is the cell it is over: the
    // reservation test refuses that cell (claim 39).
    let here = w.obj(a).unwrap().pos.cell();
    w.obj_mut(b).unwrap().dest = Some(Target::Cell(here));
    let rng = w.rng.clone();
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    assert_eq!(c, here, "still over that cell at the order");
    let s = st(&w, a);
    let Some(Target::Cell(fc)) = s.flying_to else {
        panic!("a flight point: {}", describe(&w, a))
    };
    assert!(s.own_point);
    assert_ne!(fc, c, "a replacement cell: {}", describe(&w, a));
    assert!(near(fc, c), "radius 1 first holds a cell: {fc:?} {c:?}");
    assert_ne!(w.rng, rng, "the search's draws");
}

// spec: S-0501 claim 8 (r120, Q-2507: the point it is landing at is its flight point; the cell compared is that of the new destination's heading point, claim 40, a mobile object's position; the exception ignores the destination: no take-off, the flight point stays as it was, and the ignored destination is still the aircraft's destination)
#[test]
fn claim_8_a_destination_object_whose_heading_point_is_in_the_flight_points_cell_is_ignored() {
    let (mut w, a, c) = stopped();
    to_landing_start(&mut w, a);
    let h0 = air::height(&w, a);
    assert!(h0 > 0);
    let fp = st(&w, a).flying_to;
    assert_eq!(fp, Some(Target::Cell(c)), "{}", describe(&w, a));
    // A vehicle standing in the flight point's cell: its heading point, its
    // position, is in that cell, but it is not the flight point.
    let v = tank(&mut w, c);
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(v)),
        }
    ));
    let mut h = h0;
    let mut ignored = false;
    for i in 0..6 {
        step(&mut w);
        let s = st(&w, a);
        let nh = air::height(&w, a);
        assert!(
            !s.taking_off && nh <= h,
            "no take-off (tick {i}): {}",
            describe(&w, a)
        );
        h = nh;
        if s.ground {
            break;
        }
        assert!(s.landing, "{}", describe(&w, a));
        assert_eq!(
            s.flying_to,
            fp,
            "the flight point kept: {}",
            describe(&w, a)
        );
        // The order applies after the turn pass; its first movement update
        // hands the destination over in the next tick.
        if i > 0 {
            assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(v)));
            assert_eq!(s.ignored, Some(Target::Object(v)), "{}", describe(&w, a));
            ignored = true;
        }
    }
    assert!(ignored, "handed over and ignored: {}", describe(&w, a));
    assert!(h < h0, "it goes on landing");
}

// spec: S-0501 claim 8 (r120, Q-2507: the point it is landing at is its flight point, not its position; the cell compared is that of the new destination's heading point, and the test is made only when the destination is handed over: a landing aircraft that is over another cell than its flight point's still ignores a destination in the flight point's cell, and an ignored destination is still its destination, so claim 4's descent move draws it toward that cell's centre; it takes off for one in the cell it is now over); AC4 same-cell part, runs (2) and (3)
#[test]
fn ac4_runs_2_and_3_the_point_compared_is_the_flight_points_cell() {
    for flight_cell in [true, false] {
        let (mut w, a, c) = stopped();
        to_landing_start(&mut w, a);
        assert!(air::height(&w, a) > 0);
        // Set-up: its flight point in the cell next to the one it is over,
        // as after a drift out of the flight point's cell (runs (2) and (3)
        // need such a drift; our runs set the flight point instead).
        let over = w.obj(a).unwrap().pos.cell();
        let fc = CellXY::new(over.x, over.y + 1);
        w.obj_mut(a).unwrap().air.as_mut().unwrap().flying_to = Some(Target::Cell(fc));
        let to = if flight_cell { fc } else { over };
        order_move(&mut w, a, to);
        let mut h = air::height(&w, a);
        let mut took_off = false;
        let mut nudged = 0;
        let mode = w.rules().mode;
        let stall = w.rules().quirks.aircraft_unit_step_stall;
        let ti = w.obj(a).unwrap().type_index;
        let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
        for _ in 0..400 {
            let o = w.obj(a).unwrap().clone();
            let was = o.air.clone().unwrap();
            let f = o.body.value(w.tick);
            step(&mut w);
            let s = st(&w, a);
            let nh = air::height(&w, a);
            if s.taking_off || nh > h {
                took_off = true;
                break;
            }
            let p = w.obj(a).unwrap().pos;
            let (nx, ny) = air::step(o.pos.x, o.pos.y, f, v, was.cur, stall, mode);
            if nh < h && (p.x, p.y) != (nx, ny) {
                let centre = Pos::centre(fc, 0);
                let d0 = (i64::from(nx - centre.x)).pow(2) + (i64::from(ny - centre.y)).pow(2);
                let d1 = (i64::from(p.x - centre.x)).pow(2) + (i64::from(p.y - centre.y)).pow(2);
                assert!(d1 < d0, "the descent move goes toward that cell's centre");
                nudged += 1;
            }
            h = nh;
            if s.ground {
                break;
            }
        }
        if flight_cell {
            assert!(!took_off, "run (2): no take-off: {}", describe(&w, a));
            assert!(nudged > 0, "run (2): the descent move: {}", describe(&w, a));
            assert_eq!(
                w.obj(a).unwrap().pos.cell().y,
                fc.y,
                "drawn into the flight point's cell: {}",
                describe(&w, a)
            );
        } else {
            assert!(
                took_off,
                "run (3): a take-off at the order: {}",
                describe(&w, a)
            );
        }
        let _ = c;
    }
}

// ---------------------------------------------------------------------------
// Claim 7: the forget before the cell test and the hold

// spec: S-0501 claim 7 (r120: before its cell is tested, the touchdown forgets the destination and the flight point when the aircraft is in the flight point's cell; a refused touchdown rises 10 leptons and starts a take-off that ends in the next movement update and leaves the landing's held height; with nothing to move it, it stays in flight, not landing, 10 leptons above the refused cell until it gets a destination, and claim 6's triggers do not apply) and F7 (an occupant other than its contact refuses the cell); AC4 occupant part
#[test]
fn claim_7_a_refused_touchdown_in_the_flight_points_cell_forgets_and_holds() {
    let mut w = game(&tested(rules()));
    ready(&mut w);
    let a = put(&mut w, "JET", -8, 0);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    let g = CellXY::new(c.x + 6, c.y);
    order_move(&mut w, a, g);
    // After the move's selection, so that the occupied cell stays the destination.
    for _ in 0..50 {
        step(&mut w);
        if st(&w, a).regive.is_none() && st(&w, a).first_moved.is_some() {
            break;
        }
    }
    tank(&mut w, g);
    let mut was = st(&w, a);
    let mut u = None;
    for _ in 0..1500 {
        step(&mut w);
        if refused(&was, &w, a) {
            u = Some(w.tick - 1);
            break;
        }
        was = st(&w, a);
    }
    assert!(u.is_some(), "a refused touchdown: {}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.pos.cell(), g, "in the flight point's cell");
    assert_eq!(
        o.dest,
        None,
        "the destination forgotten: {}",
        describe(&w, a)
    );
    let s = st(&w, a);
    assert_eq!(s.flying_to, None, "the flight point forgotten");
    assert_eq!(air::height(&w, a), 10, "the refusal's rise");
    assert!(s.taking_off && s.stranded, "{}", describe(&w, a));
    let at = o.pos;
    step(&mut w);
    assert_eq!(w.obj(a).unwrap().dest, None, "{}", describe(&w, a));
    let s = st(&w, a);
    assert!(
        !s.taking_off,
        "the take-off ends in the next update: {}",
        describe(&w, a)
    );
    assert_eq!(s.held, 0, "the landing's held height");
    let mut held = 0;
    while w.obj(a).unwrap().dest.is_none() && held < 300 {
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.pos,
            at,
            "no height or position change: {}",
            describe(&w, a)
        );
        assert!(!st(&w, a).landing, "no landing trigger");
        step(&mut w);
        held += 1;
    }
    assert!(
        held >= 30,
        "the hold lasts ({held} ticks): {}",
        describe(&w, a)
    );
    if w.obj(a).unwrap().dest.is_some() {
        // Its next destination moves it again.
        let h = air::height(&w, a);
        let rose = (0..30).any(|_| {
            step(&mut w);
            air::height(&w, a) > h || w.obj(a).is_some_and(|o| (o.pos.x, o.pos.y) != (at.x, at.y))
        });
        assert!(rose, "a destination moves it: {}", describe(&w, a));
    }
}

/// The forget cases of claim 7 away from the flight point's cell: a JET
/// landing where it is, in *sleep* (no processing rule, so nothing gives it
/// a destination), given destination `dest` as its flight point and
/// `contact`. Its touchdown cell is found on a copy of the world; a TANK
/// then stands there. Returns whether the refused touchdown forgot the
/// destination, or `None` when it touched down in the flight point's cell.
fn forget_case(dest: impl Fn(&World) -> Target, contact: bool) -> Option<bool> {
    let mut w = game(&tested(rules()));
    let k = building(&mut w, "APAD", 0, -2, 8);
    ready(&mut w);
    let a = put(&mut w, "JET", -8, 0);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    order_move(&mut w, a, CellXY::new(c.x + 12, c.y));
    let one = F64::from_i64(1, w.rules().mode);
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        if air::height(&w, a) > 0 && !s.taking_off && s.cur.eq_num(one) {
            break;
        }
    }
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    to_landing_start(&mut w, a);
    let d = dest(&w);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(Behaviour::Sleep, t, true);
        o.dest = Some(d);
        if contact {
            o.contact = Some(k);
            o.econ.exit = Some(Exit {
                factory: k,
                tethered: false,
                place: 0,
            });
        }
        let s = o.air.as_mut().unwrap();
        s.flying_to = Some(d);
        s.own_point = false;
    }
    let fc = air::flight_point(&w, a, d).cell();
    let mut copy = w.clone();
    for _ in 0..400 {
        step(&mut copy);
        if st(&copy, a).ground {
            break;
        }
    }
    assert!(st(&copy, a).ground, "{}", describe(&copy, a));
    let p = copy.obj(a).unwrap().pos.cell();
    if p == fc {
        return None;
    }
    tank(&mut w, p);
    let mut was = st(&w, a);
    for _ in 0..400 {
        step(&mut w);
        if refused(&was, &w, a) {
            assert_eq!(w.obj(a).unwrap().pos.cell(), p);
            let forgot = w.obj(a).unwrap().dest.is_none();
            assert_eq!(st(&w, a).stranded, forgot, "{}", describe(&w, a));
            assert_eq!(st(&w, a).flying_to.is_none(), forgot);
            return Some(forgot);
        }
        was = st(&w, a);
    }
    panic!("no refusal: {}", describe(&w, a));
}

// spec: S-0501 claim 7 (r120: wherever it is, the touchdown forgets the destination and the flight point when the flight point's cell holds the building it is in contact with; for an aircraft with no contact a flight point's cell with no building counts as holding it; a refused touchdown keeps the destination only outside the flight point's cell, and only when that cell's building is not its contact, with no contact when that cell holds a building)
#[test]
fn claim_7_the_forget_outside_the_flight_points_cell() {
    type DestOf = dyn Fn(&World) -> Target;
    let pad = |w: &World| {
        let k = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Building)
            .unwrap()
            .id;
        Target::Object(k)
    };
    // No contact, a flight point's cell with no building: forgotten.
    let open = |w: &World| {
        let c = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Aircraft)
            .unwrap()
            .pos
            .cell();
        Target::Cell(CellXY::new(c.x, c.y - 3))
    };
    let cases: [(&DestOf, bool, bool); 3] = [
        (&open, false, true),
        (&pad, false, false),
        (&pad, true, true),
    ];
    for (i, (dest, contact, forgets)) in cases.into_iter().enumerate() {
        let got = forget_case(dest, contact);
        assert_eq!(got, Some(forgets), "case {i}");
    }
}

// ---------------------------------------------------------------------------
// Claim 24: the entry report in flight (AC31 part h (4)) and the run-away
// notices (AC31 part h (5))

// spec: S-0501 claim 24 (r120, Q-2506: a refused touchdown in contact in the docking position's cell, its flight point's, forgets the destination and the flight point, claim 7; with nothing to move it, it is 10 leptons above the refused cell, so the second rule asks the pad to be loaded; a switched-on pad finds it settled and gives itself back as destination, and with enter current and nothing queued the processing then finds it not landing and reports entry at once, in flight: the pad starts servicing and has it queue sleep, it queues guard, and the processing returns 1 tick and leaves it ready, so guard is current by the end of that tick; it is not sent back to the approaching stage) and claim 7; AC31 part h (4)
#[test]
fn ac31h4_a_refused_touchdown_at_the_docking_position_reports_entry_in_flight() {
    let r = tested(rules_with(1, Some(vec![OFF_PAD])));
    let mut w = game(&r);
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
    let dc = glasnost_sim::econ::release::pad_position(&w, k, a).0.cell();
    assert!(!foundation(&w, k).contains(&dc));
    // W stands in the docking position's cell before the landing starts.
    tank(&mut w, dc);
    let mut was = st(&w, a);
    let mut u = None;
    for _ in 0..2000 {
        step(&mut w);
        if refused(&was, &w, a) && w.obj(a).unwrap().contact == Some(k) {
            u = Some(w.tick - 1);
            break;
        }
        assert!(
            !(st(&w, a).ground),
            "void: an accepted touchdown: {}",
            describe(&w, a)
        );
        was = st(&w, a);
    }
    let u = u.unwrap_or_else(|| panic!("no refused touchdown: {}", describe(&w, a)));
    let o = w.obj(a).unwrap();
    assert_eq!(o.pos.cell(), dc, "in the docking position's cell");
    assert_eq!(
        o.dest,
        None,
        "no destination at the end of u: {}",
        describe(&w, a)
    );
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    assert_eq!(o.beh.queued, None, "void with a behaviour queued");
    // r: its next processing after u.
    let before = o.beh.last;
    let mut r_tick = None;
    for _ in 0..10 {
        let h = air::height(&w, a);
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.beh.last != before {
            r_tick = Some(o.beh.last.unwrap());
            break;
        }
        assert_eq!(h, 10, "10 leptons above the cell: {}", describe(&w, a));
        assert_eq!(air::height(&w, a), 10, "{}", describe(&w, a));
    }
    let (r_t, d) = r_tick.unwrap_or_else(|| panic!("no processing: {}", describe(&w, a)));
    assert!(r_t <= u + 3, "r at most u + 3");
    assert_eq!(d, 1, "the processing returns 1 tick");
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Guard),
        "the entered report: {}",
        describe(&w, a)
    );
    assert!(
        matches!(o.beh.queued, None | Some(Behaviour::Sleep)),
        "{}",
        describe(&w, a)
    );
    assert_eq!(o.contact, Some(k));
    if service(&w, k).is_none() {
        step(&mut w);
    }
    assert!(service(&w, k).is_some(), "servicing at r or r + 1");
    assert_ne!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
}

/// AC31 part h (5)'s run: the pad *P*₂ with two docking places (test
/// values) and two JETs, the first given its dock order one cycle before
/// the second (`first`: which one); the pad is sold once both are in flight
/// more than 5 cells from it. Returns (each aircraft's contact place before
/// the sale, its destination cell at the end of the sale's first tick).
fn run_away_run(first: usize) -> [(u32, CellXY); 2] {
    let r = rules_with(2, Some(vec![[-128, -128, 0], [128, 128, 0]]));
    let mut w = game(&r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let jets = [put(&mut w, "JET", 12, 0), put(&mut w, "JET", 12, 1)];
    step(&mut w);
    for i in [first, 1 - first] {
        assert!(issue(
            &mut w,
            0,
            Order::Behaviour {
                actor: jets[i],
                behaviour: Behaviour::Enter,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(k)),
            }
        ));
        step(&mut w);
    }
    let pc = w.obj(k).unwrap().pos.cell();
    for _ in 0..200 {
        if jets.iter().all(|&a| air::height(&w, a) > 0) {
            break;
        }
        step(&mut w);
    }
    let places = jets.map(|a| {
        let o = w.obj(a).unwrap();
        assert_eq!(o.contact, Some(k), "a contact: {}", describe(&w, a));
        assert!(
            (o.pos.cell().x - pc.x).abs() > 5,
            "more than 5 cells away: {}",
            describe(&w, a)
        );
        o.econ.exit.unwrap().place
    });
    assert_eq!(places[first], 0, "the first granted holds place 0");
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(&mut w, 0, Order::Sell { actor: k }));
    for _ in 0..20 {
        step(&mut w);
        if w.obj(k)
            .is_none_or(|b| b.beh.current == Some(Behaviour::Selling))
        {
            break;
        }
    }
    let spots = jets.map(|a| match w.obj(a).unwrap().dest {
        Some(Target::Cell(c)) => c,
        d => panic!("a cell as destination: {d:?} {}", describe(&w, a)),
    });
    [(places[0], spots[0]), (places[1], spots[1])]
}

// spec: S-0501 claim 24 (r120, Q-2508: the pad sends its notices in contact-place order, from place 0 up, each handled in full before the next; the order of granting counts only through the places taken; no aircraft's landing spot depends on the spots given before it in the same sale: F8 takes no draws and does not consider other aircraft's destinations, so two aircraft for which F8 searches near the same building get the same spot) and F8; AC31 part h (5)
#[test]
fn ac31h5_the_run_away_spots_do_not_depend_on_each_other() {
    let a = run_away_run(0);
    let b = run_away_run(1);
    assert_eq!(a[0].1, a[1].1, "run (5a): the same cell");
    assert_eq!(b[0].1, b[1].1, "run (5b): the same cell");
    assert_eq!(a[0].1, b[0].1, "the same cell in both runs");
    assert_eq!((a[0].0, a[1].0), (0, 1));
    assert_eq!((b[0].0, b[1].0), (1, 0));
}

// ---------------------------------------------------------------------------
// Claim 29: the unarmed aircraft at cruise height (intake 92's seam)

// spec: S-0501 claim 29 (r110, Q-2437: at exactly cruise height, outside a team, an unarmed aircraft in guard takes its own cell as destination and move, so it lands, whatever its destination; r150, Q-2726: outside a team the processing at exactly cruise height ends with a delay of 1 tick, the 45 ticks being for outcomes on the ground) and claim 31 (r110, Q-2438: this processing leaves the ready mark as it is)
#[test]
fn claim_29_an_unarmed_aircraft_at_cruise_height_lands_in_its_own_cell() {
    for mark_cleared in [false, true] {
        let mut w = game(&rules());
        ready(&mut w);
        let a = put(&mut w, "LIFT", -8, 0);
        step(&mut w);
        let c = w.obj(a).unwrap().pos.cell();
        order_move(&mut w, a, CellXY::new(c.x + 12, c.y));
        let mut cruise = false;
        for _ in 0..600 {
            step(&mut w);
            let s = st(&w, a);
            let h = air::height(&w, a);
            if !s.taking_off && h > 0 && s.held == h {
                cruise = true;
                break;
            }
        }
        assert!(cruise, "{}", describe(&w, a));
        let t = w.tick;
        {
            let o = w.obj_mut(a).unwrap();
            o.beh.force(Behaviour::Guard, t, true);
            o.air.as_mut().unwrap().mark_cleared = mark_cleared;
        }
        let h = air::height(&w, a);
        let at = w.obj(a).unwrap().pos.cell();
        let (_, d) = next_processing(&mut w, a);
        assert_eq!(air::height(&w, a), h, "at cruise height");
        let o = w.obj(a).unwrap();
        assert_eq!(
            d, 1,
            "at exactly cruise height outside a team: 1 tick (r150)"
        );
        assert!(
            o.beh.current == Some(Behaviour::Move) || o.beh.queued == Some(Behaviour::Move),
            "move: {}",
            describe(&w, a)
        );
        assert_eq!(
            st(&w, a).mark_cleared,
            mark_cleared,
            "the ready mark left as it is"
        );
        let Some(Target::Cell(own)) = o.dest else {
            panic!("its own cell: {}", describe(&w, a))
        };
        assert!(near(own, at), "its own cell: {own:?} {at:?}");
        let mut landed = false;
        for _ in 0..600 {
            step(&mut w);
            if st(&w, a).ground {
                landed = true;
                break;
            }
        }
        assert!(landed, "it lands: {}", describe(&w, a));
    }
}
