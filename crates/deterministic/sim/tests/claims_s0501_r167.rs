//! Claim tests for S-0501 r167 (Q-2802 answered) and r172 (Q-2821
//! answered), M8 intake 118: *sticky* at a pad (claim 29: the pad found
//! below cruise height and its settled start, a contact held at cruise
//! height, *On the pad*), the held height of claim 48 (b)'s third case
//! (claim 6), the flights of scenario AC33 parts c to f, and the flagged
//! aircraft type that is not a rocket type (claim 18, *Missile types*;
//! scenario AC35). Synthetic data through the real tick path; every type,
//! weapon, map and number is invented. Ledger: `docs/m8-claims/intake118.md`.

mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready, wheel};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// AC31 part h (1)'s displacement (test value): from the 2 × 2 foundation's
/// centre to the centre of the cell two cells east of its east column, in
/// its south row, at height 0, outside the foundation.
const OFF_PAD: [i32; 3] = [640, 128, 0];

/// AC33 part c's displacement (test value): from the 2 × 2 foundation's
/// centre, the corner its four cells share, to the centre of its south-east
/// cell, at height 0, so that no landing can start outside that cell.
const CELL_PAD: [i32; 3] = [128, 128, 0];

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with one docking place at the foundation centre); KITE (unarmed,
/// landable, not pad-bound); JET (armed, landable, 2 rounds, docking at
/// APAD); BOUND (JET, pad-bound); NOLAND (unarmed, not landable, fly-by, so
/// claim 5 holds it at full speed); DRIFT (NOLAND without fly-by); FLAG
/// (JET with 3 rounds and the missile-spawn flag, not a rocket type); the
/// target vehicle TGT (strength 10, one shot of Agm).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "buildings", "APAD");
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(1);
    pad.econ.docking_offsets = Some(vec![[0, 0, 0]]);
    r.sim.buildings.insert("APAD".into(), pad);
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
    let plane = |armed: bool| {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(40),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            tech_level: Some(1),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            cost: Some(1000),
            ..Default::default()
        };
        if armed {
            u.primary = Some("Agm".into());
            u.combat.ammo = Some(2);
            u.econ.dock = Some(vec!["APAD".into()]);
        }
        u.econ.landable = Some(true);
        u
    };
    let mut bound = plane(true);
    bound.econ.airport_bound = Some(true);
    let mut noland = plane(false);
    noland.econ.landable = Some(false);
    let mut drift = noland.clone();
    noland.econ.fly_by = Some(true);
    let mut flag = plane(true);
    flag.combat.ammo = Some(3);
    flag.missile_spawn = Some(true);
    for (n, u) in [
        ("KITE", plane(false)),
        ("JET", plane(true)),
        ("BOUND", bound),
        ("NOLAND", noland),
        ("DRIFT", {
            drift.econ.fly_by = Some(false);
            drift
        }),
        ("FLAG", flag),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    let mut tgt = wheel(500);
    tgt.strength = Some(10);
    push(&mut r, "vehicles", "TGT");
    r.sim.vehicles.insert("TGT".into(), tgt);
    r
}

/// `r` with aircraft type `name` changed by `f`.
fn with(mut r: Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) -> Ruleset {
    f(r.sim.aircraft.get_mut(name).unwrap());
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let Some(o) = w.obj(id) else {
        return format!("t={} gone", w.tick);
    };
    format!(
        "t={} pos={:?} h={} hp={} ammo={} beh={:?}/{:?} dest={:?} target={:?} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.health,
        o.combat.ammo,
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.target,
        o.contact,
        o.air,
    )
}

fn cruise(w: &World, a: ObjectId) -> i32 {
    air::cruise_height(w, a)
}

/// AC33's set-up, as `claims_s0501_r150`'s: a `name` aircraft of house 0 on
/// the ground at the west of the map with `beh` current, then given a move
/// order to *C*, eleven cells east of the middle, applied by the end of the
/// returned world's tick. With `pad`, an APAD of house 0 far north of the
/// line. Returns (world, aircraft, C, pad).
fn ordered(
    r: &Ruleset,
    name: &str,
    beh: Behaviour,
    pad: bool,
) -> (World, ObjectId, CellXY, Option<ObjectId>) {
    let mut w = game(r);
    let k = pad.then(|| building(&mut w, "APAD", 0, 0, -12));
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        name,
        0,
        testkit::mid(SIZE, -12, 4),
        0,
    );
    step(&mut w);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(beh, t, true);
        o.beh.queued = None;
    }
    let c = testkit::mid(SIZE, 11, 4);
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
    step(&mut w);
    (w, a, c, k)
}

/// Steps until the first processing of *sticky* at cruise height has given
/// the unarmed aircraft the cell it is over (a destination other than *C*);
/// returns whether it came within `max` ticks.
fn to_own_cell(w: &mut World, a: ObjectId, c: CellXY, max: i32) -> bool {
    for _ in 0..max {
        step(w);
        if let Some(Target::Cell(d)) = w.obj(a).unwrap().dest
            && d != c
        {
            return true;
        }
    }
    false
}

fn tenth(w: &World) -> F64 {
    let mode = w.rules().mode;
    F64::from_i64(1, mode).div(F64::from_i64(10, mode), mode)
}

// ---------------------------------------------------------------------------
// AC33 part c: the contact held at cruise height, the landing at the pad and
// the stay where it touched down

// spec: S-0501 claim 29 (r167, Q-2802: a contact held at cruise height keeps the pad as destination, the contact and enter queued with sticky current at every later tick before the touchdown; On the pad: a pad-bound aircraft whose docking position lies in the foundation lands at the pad's heading point, claim 4's descent move applies since sticky is not enter, the touchdown forgets its destination and flight point since the flight point's cell holds its contact, claim 7, and it then rests where it touched down in sticky with enter queued and the contact; it never processes enter, so the pad starts no servicing for it, claim 26) and Edge cases (quirks.aircraft_descent_touchdown_untested in both positions: with it off, F7 accepts the cell whose object is the contact, and the accepted touchdown forgets the destination too); scenario AC33 part c
#[test]
fn ac33_c_it_lands_at_the_pad_and_rests_in_sticky_with_enter_queued() {
    for untested in [true, false] {
        let mut r = with(rules(), "BOUND", |u| u.slowdown_distance = Some(1000));
        r.globals.quirks.aircraft_descent_touchdown_untested = untested;
        r.sim
            .buildings
            .get_mut("APAD")
            .unwrap()
            .econ
            .docking_offsets = Some(vec![CELL_PAD]);
        let (mut w, a, _, k) = ordered(&r, "BOUND", Behaviour::Sticky, true);
        let k = k.unwrap();
        let (dock, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
        // spec: S-0204 claim 2 (a cell is 256 leptons wide; its centre is 128 in)
        assert_eq!(
            (dock.x.rem_euclid(256), dock.y.rem_euclid(256)),
            (128, 128),
            "void: the docking position is not a cell centre: {dock:?}"
        );
        let mut contact = false;
        let mut down = None;
        for _ in 0..3000 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            let d = describe(&w, a);
            assert!(!air::crashing(&w, a), "{untested}: no crash: {d}");
            if st(&w, a).ground {
                down = Some(o.pos);
                break;
            }
            if o.contact == Some(k) {
                contact = true;
            }
            if contact {
                assert_eq!(
                    (o.dest, o.contact, o.beh.current, o.beh.queued),
                    (
                        Some(Target::Object(k)),
                        Some(k),
                        Some(Behaviour::Sticky),
                        Some(Behaviour::Enter)
                    ),
                    "{untested}: before the touchdown: {d}"
                );
            }
        }
        assert!(contact, "{untested}: the contact: {}", describe(&w, a));
        let Some(p) = down else {
            panic!("{untested}: no touchdown: {}", describe(&w, a));
        };
        for _ in 0..600 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            let d = describe(&w, a);
            assert_eq!(o.pos, p, "{untested}: it stays: {d}");
            assert_eq!(air::height(&w, a), 0, "{untested}: {d}");
            assert_eq!(
                (o.dest, o.contact, o.beh.current, o.beh.queued),
                (
                    None,
                    Some(k),
                    Some(Behaviour::Sticky),
                    Some(Behaviour::Enter)
                ),
                "{untested}: at rest: {d}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC33 part d: a slowdown distance of 0

// spec: S-0501 claim 29 (r167, Q-2802: being given its flight point again changes nothing in its speed fraction; with a slowdown distance of 0 it never lands: away from the cell's centre its target fraction is 1, its fraction is at least 0.1 after each tick's adjustment, F4, and with V of at least 20 claim 4's step is at least 2 leptons, so it moves every tick at cruise height with sticky current and move queued) and claim 5; scenario AC33 part d
#[test]
fn ac33_d_with_a_slowdown_distance_of_0_it_never_lands() {
    let r = with(rules(), "KITE", |u| u.slowdown_distance = Some(0));
    let (mut w, a, c, _) = ordered(&r, "KITE", Behaviour::Sticky, false);
    let ti = w.obj(a).unwrap().type_index;
    let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
    assert!(v >= 20, "void: V {v}");
    assert!(to_own_cell(&mut w, a, c, 1500), "{}", describe(&w, a));
    let tenth = tenth(&w);
    let mut at = w.obj(a).unwrap().pos;
    for _ in 0..2500 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        let d = describe(&w, a);
        assert_eq!(air::height(&w, a), cruise(&w, a), "cruise height: {d}");
        assert!(!air::on_ground(&w, a) && !st(&w, a).landing, "{d}");
        assert!(st(&w, a).cur.ge(tenth), "fraction at least 0.1: {d}");
        assert_ne!(o.pos, at, "it moves every tick: {d}");
        at = o.pos;
        assert_eq!(
            (o.beh.current, o.beh.queued),
            (Some(Behaviour::Sticky), Some(Behaviour::Move)),
            "{d}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC33 part e: V from 1 to 9 with a positive slowdown distance

// spec: S-0501 claim 29 (r167, Q-2802: with a positive slowdown distance and V below 20 claim 5's slow-speed cases apply to the centre of the cell it comes to rest over: it lands there unless claim 5's zero step holds it short of the halving zone, the part less than 86 leptons from that centre for a slowdown distance of at least 2560; held so, it stays at cruise height over that cell with a positive fraction for as long as sticky lasts) and claim 5 (the zero step of V below 10) and Edge cases (quirks.aircraft_unit_step_stall: run in both positions as a check that its off position, which changes only 1-lepton steps, a V from 10 to 19 matter, leaves AC33 part e's outcome; its own both-position tests are claims_s0501.rs's); scenario AC33 part e
#[test]
fn ac33_e_a_slow_aircraft_hovers_short_of_the_zone_or_lands_in_its_cell() {
    for stall in [true, false] {
        let mut r = with(rules(), "KITE", |u| {
            u.speed = Some(3);
            u.slowdown_distance = Some(3000);
        });
        r.globals.quirks.aircraft_unit_step_stall = stall;
        let (mut w, a, c, _) = ordered(&r, "KITE", Behaviour::Sticky, false);
        let ti = w.obj(a).unwrap().type_index;
        let v = w.rules().unit(Kind::Aircraft, ti).unwrap().speed;
        assert!((1..=9).contains(&v), "void: V {v}");
        assert!(
            to_own_cell(&mut w, a, c, 3000),
            "{stall}: {}",
            describe(&w, a)
        );
        // The first standstill at cruise height: neither position nor body
        // facing changes in a tick.
        let mut prev = (w.obj(a).unwrap().pos, w.obj(a).unwrap().body.value(w.tick));
        let mut still = None;
        let mut landed = None;
        let mut given = None;
        for _ in 0..2500 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            let now = (o.pos, o.body.value(w.tick));
            if air::on_ground(&w, a) {
                landed = Some(o.pos.cell());
                break;
            }
            if let Some(Target::Cell(g)) = o.dest {
                given = Some(g);
            }
            if air::height(&w, a) == cruise(&w, a) && now == prev {
                still = Some(o.pos);
                break;
            }
            prev = now;
        }
        let d = describe(&w, a);
        if let Some(cell) = landed {
            // It landed with no standstill at cruise height first: in the
            // cell it was last given.
            assert_eq!(Some(cell), given, "{stall}: {d}");
            continue;
        }
        let Some(p) = still else {
            panic!("{stall}: neither a standstill nor a landing: {d}");
        };
        let centre = glasnost_sim::world::Pos::centre(p.cell(), 0);
        let (dx, dy) = (i64::from(p.x - centre.x), i64::from(p.y - centre.y));
        let far = dx * dx + dy * dy >= 86 * 86; // spec: S-0501 claim 5 (the halving zone of a slowdown distance of at least 2560: less than 86 leptons from the centre)
        let mut ticks_still = 1;
        let mut down = false;
        for _ in 0..1000 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            let dd = describe(&w, a);
            assert_eq!(
                (o.beh.current, o.beh.queued),
                (Some(Behaviour::Sticky), Some(Behaviour::Move)),
                "{stall}: {dd}"
            );
            if far {
                assert_eq!(o.pos, p, "{stall}: it stays, far from the centre: {dd}");
                assert!(st(&w, a).cur.gt(F64::ZERO), "{stall}: {dd}");
                assert!(!air::on_ground(&w, a), "{stall}: no landing: {dd}");
            } else {
                if air::on_ground(&w, a) {
                    assert_eq!(o.pos.cell(), p.cell(), "{stall}: in that cell: {dd}");
                    down = true;
                    break;
                }
                if o.pos == p && air::height(&w, a) == cruise(&w, a) {
                    ticks_still += 1;
                }
                assert!(ticks_still <= 10, "{stall}: still at most 10 ticks: {dd}");
            }
        }
        assert!(far || down, "{stall}: it lands: {}", describe(&w, a));
    }
}

// ---------------------------------------------------------------------------
// AC33 part f: a docking position outside the foundation

// spec: S-0501 claim 29 (r167, Q-2802, On the pad: a pad-bound aircraft whose pad's docking position lies outside the pad's foundation never touches down at that pad; each landing start is refused and the aircraft falls back to rest, claim 6, which with sticky current is the fall-back of A contact held at cruise height: with that pad the only one of its dock types that accepts it, that fall-back keeps the pad, the contact and the queued enter, so it stays in the air over the docking position in sticky); scenario AC33 part f
#[test]
fn ac33_f_over_a_docking_position_outside_the_foundation_it_never_touches_down() {
    let mut r = with(rules(), "BOUND", |u| u.slowdown_distance = Some(1000));
    r.sim
        .buildings
        .get_mut("APAD")
        .unwrap()
        .econ
        .docking_offsets = Some(vec![OFF_PAD]);
    let (mut w, a, _, k) = ordered(&r, "BOUND", Behaviour::Sticky, true);
    let k = k.unwrap();
    let (dock, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    let mut over = false;
    for _ in 0..3000 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        let d = describe(&w, a);
        assert!(!air::crashing(&w, a), "no crash: {d}");
        assert!(!st(&w, a).ground, "no touchdown: {d}");
        if o.pos.cell() == dock.cell() {
            over = true;
        }
        if over {
            assert_eq!(
                (o.dest, o.contact, o.beh.current, o.beh.queued),
                (
                    Some(Target::Object(k)),
                    Some(k),
                    Some(Behaviour::Sticky),
                    Some(Behaviour::Enter)
                ),
                "over the docking position: {d}"
            );
        }
    }
    assert!(
        over,
        "it comes over the docking position: {}",
        describe(&w, a)
    );
}

// ---------------------------------------------------------------------------
// Claim 29: the pad found below cruise height

// spec: S-0501 claim 29 (r167, Q-2802: an armed aircraft below cruise height with less than full ammunition and no contact that finds a pad queues enter first and is given the pad after that, so claim 24's contact rule takes it into contact at that processing; the pad then asks whether it has settled with the destination it held before the pad: one standing landed has, and is made to start enter at once although it is not ready to change, so sticky ends) and claim 24
#[test]
fn claim_29_a_landed_aircraft_that_finds_a_pad_starts_enter_at_once() {
    for beh in [Behaviour::Sticky, Behaviour::Guard] {
        let mut w = game(&rules());
        let k = building(&mut w, "APAD", 0, 0, -8);
        ready(&mut w);
        let a = testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "JET",
            0,
            testkit::mid(SIZE, 6, 4),
            0,
        );
        step(&mut w);
        assert!(air::on_ground(&w, a), "{}", describe(&w, a));
        let t = w.tick;
        {
            let o = w.obj_mut(a).unwrap();
            o.combat.ammo = 1;
            o.contact = None;
            o.dest = None;
            o.beh.force(beh, t, true);
            o.beh.queued = None;
        }
        let mut found = false;
        for _ in 0..60 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            if o.contact == Some(k) {
                assert_eq!(
                    o.dest,
                    Some(Target::Object(k)),
                    "{beh:?}: {}",
                    describe(&w, a)
                );
                assert_eq!(
                    o.beh.current,
                    Some(Behaviour::Enter),
                    "{beh:?}: enter at once: {}",
                    describe(&w, a)
                );
                found = true;
                break;
            }
        }
        assert!(found, "{beh:?}: the pad: {}", describe(&w, a));
    }
}

// spec: S-0501 claim 29 (r167, Q-2802: one that has not settled, so one in flight with a destination, keeps sticky, with enter queued, the contact and the pad as destination, and flies to the pad) and claim 24
#[test]
fn claim_29_one_in_flight_with_a_destination_keeps_sticky_with_enter_queued() {
    // A high cruise height gives a long climb, so that a processing comes
    // below it.
    let r = with(rules(), "JET", |u| u.econ.flight_level = Some(3000));
    let (mut w, a, _, k) = ordered(&r, "JET", Behaviour::Sticky, true);
    let k = k.unwrap();
    let mut lowered = false;
    for _ in 0..400 {
        step(&mut w);
        let h = air::height(&w, a);
        if h > 0 && h < cruise(&w, a) / 4 {
            w.obj_mut(a).unwrap().combat.ammo = 1;
            lowered = true;
            break;
        }
    }
    assert!(lowered, "void: no climb: {}", describe(&w, a));
    let mut found = false;
    for _ in 0..200 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.contact == Some(k) {
            let d = describe(&w, a);
            assert!(air::height(&w, a) < cruise(&w, a), "below cruise: {d}");
            assert!(air::height(&w, a) > 0, "in flight: {d}");
            assert_eq!(
                (o.dest, o.beh.current, o.beh.queued),
                (
                    Some(Target::Object(k)),
                    Some(Behaviour::Sticky),
                    Some(Behaviour::Enter)
                ),
                "{d}"
            );
            found = true;
            break;
        }
    }
    assert!(found, "the pad: {}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// Claim 6 and claim 48 (b)'s third case: types that are not landable

// spec: S-0501 claim 29 (r167, Q-2802: a type that is not landable that claim 5 holds at full speed is given the cell it is over; held height 0 within 768 leptons of that cell's flight point, its target fraction stays 1, so each movement update begun with that held height takes claim 48 (b)'s third case, which gives it the held height of ordinary flight again before its height changes: it keeps its cruise height; one that claim 5 does not hold at full speed descends to height 0 with no landing, claim 50) and claim 6 (r167)
#[test]
fn claim_29_a_full_speed_type_that_is_not_landable_keeps_cruise_height() {
    for (name, keeps) in [("NOLAND", true), ("DRIFT", false)] {
        let (mut w, a, c, _) = ordered(&rules(), name, Behaviour::Sticky, false);
        assert!(
            to_own_cell(&mut w, a, c, 1500),
            "{name}: {}",
            describe(&w, a)
        );
        let mut lowest = air::height(&w, a);
        let mut seen = 0;
        for _ in 0..600 {
            step(&mut w);
            // Its path is not traced (claim 29): a fly-by type may leave the
            // map, and the record ends there.
            if w.obj(a).is_none_or(|o| !o.on_map) {
                break;
            }
            seen += 1;
            let d = describe(&w, a);
            assert!(
                !st(&w, a).landing && !st(&w, a).ground,
                "{name}: no landing: {d}"
            );
            lowest = lowest.min(air::height(&w, a));
            if keeps {
                assert_eq!(air::height(&w, a), cruise(&w, a), "{name}: {d}");
            }
        }
        assert!(seen >= 60, "{name}: void: off the map after {seen} ticks");
        if !keeps {
            assert_eq!(lowest, 0, "{name}: it descends: {}", describe(&w, a));
        }
    }
}

// ---------------------------------------------------------------------------
// AC35: a flagged aircraft type that is not a rocket type (r172)

/// AC35's set-up: the pad APAD of house 0, `name` of house 0 standing
/// landed 20 cells from it, and TGT of house 1 in *guard* 15 cells from
/// both; an attack order on TGT. Returns (world, aircraft, pad, target).
fn ac35(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, -14);
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        name,
        0,
        testkit::mid(SIZE, -10, 8),
        0,
    );
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TGT",
        1,
        testkit::mid(SIZE, 12, 6),
        0,
    );
    step(&mut w);
    assert!(air::on_ground(&w, a), "void: {}", describe(&w, a));
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(v)),
            dest: None,
        }
    ));
    (w, a, k, v)
}

/// Steps until TGT is gone and the aircraft's run has ended (*attack* no
/// longer current); returns whether that came.
fn to_run_end(w: &mut World, a: ObjectId, v: ObjectId) -> bool {
    for _ in 0..3000 {
        step(w);
        let dead = w.obj(v).is_none_or(|o| o.health <= 0 || !o.on_map);
        if dead && w.obj(a).unwrap().beh.current != Some(Behaviour::Attack) {
            return true;
        }
    }
    false
}

// spec: S-0501 claim 18, Missile types (r172, Q-2821: the type's missile-spawn flag alone decides that an aircraft takes the rows for an aircraft on the ground at any height, a flagged type that is not a rocket type included, placed by a map: at its run end in flight with ammunition left it forgets its target and destination and gets guard, with no contact; its dock types and pad-bound field play no part in the row, and with ammunition left the after-the-row search does not apply) and claim 29 (r172: its guard processings at exactly cruise height are the same fall-back, so it gets no destination there; a pad comes only below cruise height, with a round spent and no contact) and claim 17 (no edge cell); the unflagged control takes T2's in-flight pad row: the pad with enter and a contact; scenario AC35
#[test]
fn ac35_a_flagged_type_that_is_not_a_rocket_type_takes_the_ground_rows_in_flight() {
    let (mut w, a, k, v) = ac35(&rules(), "FLAG");
    assert!(
        to_run_end(&mut w, a, v),
        "the run ends: {}",
        describe(&w, a)
    );
    let o = w.obj(a).unwrap();
    let d = describe(&w, a);
    assert!(air::height(&w, a) > 0, "void: the run ends in flight: {d}");
    assert!(o.combat.ammo > 0, "void: ammunition left: {d}");
    assert_eq!(
        (o.target, o.dest, o.contact, o.beh.current),
        (None, None, None, Some(Behaviour::Guard)),
        "at the run end: {d}"
    );
    assert!(
        o.beh.queued != Some(Behaviour::Enter),
        "no enter queued: {d}"
    );
    for _ in 0..300 {
        let h0 = air::height(&w, a);
        let c0 = cruise(&w, a);
        let had = w.obj(a).unwrap().dest;
        step(&mut w);
        let o = w.obj(a).unwrap();
        let d = describe(&w, a);
        assert!(
            o.dest.is_none() || o.dest == Some(Target::Object(k)),
            "no edge cell: {d}"
        );
        let gets = o.dest == Some(Target::Object(k)) && had != o.dest;
        if gets && h0 == c0 {
            panic!("the pad got at a tick begun at cruise height: {d}");
        }
    }
    // Control: the unflagged type.
    let r = with(rules(), "FLAG", |u| u.missile_spawn = None);
    let (mut w, a, k, v) = ac35(&r, "FLAG");
    assert!(to_run_end(&mut w, a, v), "control: {}", describe(&w, a));
    let o = w.obj(a).unwrap();
    let d = describe(&w, a);
    assert_eq!(o.dest, Some(Target::Object(k)), "control: the pad: {d}");
    assert_eq!(o.contact, Some(k), "control: the contact: {d}");
    assert!(
        o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter),
        "control: enter: {d}"
    );
}
