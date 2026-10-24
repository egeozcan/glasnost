//! When an aircraft is a tracked flying object (S-0501 claim 47, revision
//! 45, answering Q-1845; S-0303 Terms and claim 11, F7), on synthetic data,
//! through the real order and tick path.
//!
//! Every type, weapon and number here is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready, set_credits, t};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::combat::flying;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const ALPHA: HouseId = HouseId(0);

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// An unarmed flying type docking at APAD, cruise height 500 (the
/// game-wide default of the synthetic rules).
fn flyer() -> UnitType {
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        ..Default::default()
    };
    u.econ.landable = Some(true);
    u.econ.dock = Some(vec!["APAD".into()]);
    u
}

/// The rules: the economy ruleset with
/// - the pad APAD (an aircraft factory and helipad, one docking place);
/// - SCOUT (unarmed, landable, the counts-as-aircraft key left out);
/// - HOVER (as SCOUT, not landable: it hovers at its destination);
/// - FLAGOFF (as SCOUT with the counts-as-aircraft flag set to no);
/// - LOW (as SCOUT with a cruise height of 150 leptons, below 208).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.number_of_docks = Some(1);
    push(&mut r, "buildings", "APAD");
    r.sim.buildings.insert("APAD".into(), pad);
    for (n, u) in [
        ("SCOUT", flyer()),
        ("HOVER", {
            let mut u = flyer();
            u.econ.landable = Some(false);
            u
        }),
        ("FLAGOFF", {
            let mut u = flyer();
            u.econ.considered_aircraft = Some(false);
            u
        }),
        ("LOW", {
            let mut u = flyer();
            u.econ.flight_level = Some(150);
            u
        }),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    r
}

fn put(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

/// Puts a `name` of house 0 on the map at `cell` with the visiting mark set
/// before its placement, so that it is placed at its cruise height inside
/// the usable area (S-0501 claim 1).
fn put_in_air(w: &mut World, name: &str, cell: glasnost_map::CellXY) -> ObjectId {
    let ti = w.rules().index_of(Kind::Aircraft, name).unwrap();
    let id = glasnost_sim::objects::create(w, Kind::Aircraft, ti, Some(ALPHA));
    w.obj_mut(id).unwrap().air = Some(air::AirState {
        visiting: true,
        ..Default::default()
    });
    let p = Pos::centre(cell, 0);
    let z = w.surface_at(p.x, p.y);
    glasnost_sim::objects::place(w, id, Pos::new(p.x, p.y, z), 0);
    id
}

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

/// A person's move order to `cell`.
fn move_to(w: &mut World, a: ObjectId, cell: glasnost_map::CellXY) {
    open(w);
    assert!(orders::issue(
        w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(cell)),
        }
    ));
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn tracked(w: &World, id: ObjectId) -> bool {
    flying::tracked(w, id)
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} cell={:?} h={} beh={:?} dest={:?} tracked={} sector={:?} air={:?}",
        w.tick,
        o.pos,
        o.pos.cell(),
        air::height(w, id),
        o.beh.current,
        o.dest,
        tracked(w, id),
        w.flying.sector_of.get(&id),
        o.air
    )
}

/// Steps until `done` or `max` ticks, checking `each` after every tick.
fn run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        step(w);
    }
    done(w)
}

/// Builds a SCOUT at a fresh APAD of house 0; returns (world, pad, aircraft).
fn built() -> (World, ObjectId, ObjectId) {
    let mut w = game(&rules());
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let what = t(&w, Kind::Aircraft, "SCOUT");
    assert!(issue(&mut w, 0, Order::Build { what }));
    let ti = what.index;
    let found = run_until(&mut w, 3000, |w| {
        w.objects
            .values()
            .any(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
    });
    assert!(found, "the aircraft is built");
    let a = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
        .unwrap()
        .id;
    (w, k, a)
}

// spec: S-0501 claim 47 (a) (tracking starts when the take-off starts, in the movement update that accepts the destination while on the ground, at the height it has then, before it rises) and (b) (a landed aircraft is never tracked)
#[test]
fn claim_47_tracking_starts_with_the_take_off() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", -6, 0);
    assert!(st(&w, a).ground, "placed on the ground (claim 1)");
    for _ in 0..20 {
        step(&mut w);
        assert!(!tracked(&w, a), "landed: not tracked, {}", describe(&w, a));
    }
    move_to(&mut w, a, testkit::mid(SIZE, 8, 0));
    let mut ticks = 0;
    while st(&w, a).ground {
        assert!(
            !tracked(&w, a),
            "not before the take-off: {}",
            describe(&w, a)
        );
        step(&mut w);
        ticks += 1;
        assert!(ticks < 20, "takes off: {}", describe(&w, a));
    }
    // The update that took off climbed by one step from height 0.
    // spec: S-0501 F1 (a climb step of 20 without passengers)
    assert!(st(&w, a).taking_off, "{}", describe(&w, a));
    assert_eq!(air::height(&w, a), 20, "{}", describe(&w, a));
    assert!(
        tracked(&w, a),
        "tracked from the take-off: {}",
        describe(&w, a)
    );
    assert_eq!(
        w.flying.order.iter().filter(|&&x| x == a).count(),
        1,
        "listed once"
    );
}

// spec: S-0501 claim 47 (b) (tracking ends at touchdown, not when the landing starts; nothing else ends it while it flies: the landing descent stays tracked)
#[test]
fn claim_47_tracking_ends_at_touchdown_not_at_the_landing_start() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", -6, 0);
    move_to(&mut w, a, testkit::mid(SIZE, 6, 0));
    assert!(run_until(&mut w, 50, |w| !st(w, a).ground));
    let mut landing_ticks = 0;
    let down = run_until(&mut w, 2000, |w| {
        let s = st(w, a);
        if s.ground {
            return true;
        }
        assert!(tracked(w, a), "tracked in flight: {}", describe(w, a));
        if s.landing {
            landing_ticks += 1;
        }
        false
    });
    assert!(down, "lands: {}", describe(&w, a));
    assert!(landing_ticks > 1, "a landing descent of several updates");
    assert!(
        !tracked(&w, a),
        "not tracked after touchdown: {}",
        describe(&w, a)
    );
    for _ in 0..30 {
        step(&mut w);
        assert!(!tracked(&w, a), "landed: {}", describe(&w, a));
    }
}

// spec: S-0501 claim 47 (a) (leaving a pad is a take-off) and (b) (an aircraft built at a pad stands docked and is not tracked). The return to the pad: `claim_47_tracked_from_the_pad_to_the_pad` in `claims_s0501.rs`
#[test]
fn claim_47_pad_departure() {
    let (mut w, _k, a) = built();
    for _ in 0..10 {
        step(&mut w);
        assert!(!tracked(&w, a), "docked: {}", describe(&w, a));
    }
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    assert!(run_until(&mut w, 20, |w| !st(w, a).ground));
    assert!(
        tracked(&w, a),
        "tracked from leaving the pad: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 47 (b) (hovering with a speed fraction of 0 is ordinary flight: still tracked)
#[test]
fn claim_47_hovering_stays_tracked() {
    let mut w = game(&rules());
    let a = put(&mut w, "HOVER", -6, 0);
    move_to(&mut w, a, testkit::mid(SIZE, 4, 0));
    let stopped = run_until(&mut w, 2000, |w| {
        !st(w, a).ground
            && st(w, a).cur == glasnost_num::F64::ZERO
            && w.obj(a).unwrap().pos.cell() == testkit::mid(SIZE, 4, 0)
    });
    assert!(stopped, "hovers at its destination: {}", describe(&w, a));
    for _ in 0..30 {
        step(&mut w);
        assert!(tracked(&w, a), "hovering: {}", describe(&w, a));
    }
}

// spec: S-0501 claim 47 (b) (a dead aircraft stays tracked while it falls; the crash impact ends tracking) and claim 9
#[test]
fn claim_47_a_falling_dead_aircraft_stays_tracked() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", 0, 0);
    move_to(&mut w, a, testkit::mid(SIZE, -12, -12));
    assert!(run_until(&mut w, 300, |w| air::height(w, a) >= 400));
    glasnost_sim::combat::damage::apply(
        &mut w,
        a,
        glasnost_sim::combat::damage::Hit {
            amount: 10_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert!(air::crashing(&w, a));
    let mut falls = 0;
    let gone = run_until(&mut w, 300, |w| {
        if w.obj(a).is_none() {
            return true;
        }
        falls += 1;
        assert!(tracked(w, a), "falling: {}", describe(w, a));
        false
    });
    assert!(
        gone && falls > 1,
        "falls for several ticks, then leaves play"
    );
    assert!(!tracked(&w, a), "not tracked after the impact");
}

// spec: S-0501 claim 47 (a) (placed in the air at its cruise height of at least 208 leptons with the counts-as-aircraft flag: tracked from the placement) and Import mapping (ConsideredAircraft, absent: yes for an aircraft type)
#[test]
fn claim_47_placed_in_the_air_with_the_flag_is_tracked_at_once() {
    let mut w = game(&rules());
    let a = put_in_air(&mut w, "SCOUT", testkit::mid(SIZE, 0, 0));
    // spec: S-0501 F10 and Import mapping (FlightLevel absent: cruise height 500)
    assert_eq!(air::height(&w, a), 500, "at its cruise height");
    assert!(air::in_the_air(&w, a));
    assert!(
        tracked(&w, a),
        "tracked from its placement: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 47 (a) and (c) (placed in the air below 208 leptons or without the counts-as-aircraft flag: not tracked at placement; tracked at its first movement update with a positive carried speed fraction, outside sector (0, 0)); claim 1 (placed exactly at its cruise height: the current fraction starts at 1)
#[test]
fn claim_47_placed_in_the_air_without_the_flag_or_low_joins_at_its_first_update() {
    for name in ["FLAGOFF", "LOW"] {
        let mut w = game(&rules());
        let c = testkit::mid(SIZE, 0, 0);
        assert_ne!(flying::sector(&w, c), (0, 0));
        let a = put_in_air(&mut w, name, c);
        assert!(
            !tracked(&w, a),
            "{name}: not at placement: {}",
            describe(&w, a)
        );
        assert_ne!(st(&w, a).cur, glasnost_num::F64::ZERO, "{name}");
        step(&mut w);
        assert!(
            tracked(&w, a),
            "{name}: at its first update: {}",
            describe(&w, a)
        );
    }
}

// spec: S-0501 claim 47 (c) and Edge cases (quirks.aircraft_corner_sector_untracked: on, an untracked aircraft in flight in sector (0, 0) joins no list; off, it joins sector (0, 0)'s list at its first movement update with a positive carried fraction). A map of 10 × 10 cells has B = 19, so the sector size is 0 and every cell is in sector (0, 0) (S-0303 F7 and Edge cases)
#[test]
fn claim_47_corner_sector_quirk_both_ways() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_corner_sector_untracked = quirk;
        let mut w = testkit::game_with(&r, 10, 7);
        let c = testkit::mid(10, 0, 0);
        assert_eq!(flying::sector(&w, c), (0, 0));
        let a = put_in_air(&mut w, "FLAGOFF", c);
        assert!(!tracked(&w, a));
        // spec: S-0501 Import mapping (ConsideredAircraft absent: yes for an aircraft type) and claim 47 (a): the control of AC30 h (2), tracked from its placement in sector (0, 0) whatever the switch
        let s = put_in_air(&mut w, "SCOUT", testkit::mid(10, 1, 1));
        assert!(tracked(&w, s), "quirk {quirk}: {}", describe(&w, s));
        for _ in 0..10 {
            step(&mut w);
            if w.obj(a).is_none_or(|o| !o.on_map) {
                break;
            }
            assert_eq!(tracked(&w, a), !quirk, "quirk {quirk}: {}", describe(&w, a));
        }
        if !quirk {
            assert!(tracked(&w, a));
            assert_eq!(w.flying.sector_of.get(&a), Some(&(0, 0)));
        }
    }
}

// spec: S-0501 claim 47 (c) (the sector is brought up to date at the start of each movement update, before the forward step, from the cell it is in then: it follows the position one movement update late) and S-0303 claim 11 (a crossing object joins the end of its new sector's list)
#[test]
fn claim_47_the_sector_follows_one_update_late() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", -12, 0);
    move_to(&mut w, a, testkit::mid(SIZE, 12, 0));
    assert!(run_until(&mut w, 50, |w| tracked(w, a)));
    let mut lagged = 0;
    let mut checked = 0;
    for _ in 0..300 {
        let before = w.obj(a).unwrap().pos.cell();
        let carried = st(&w, a).cur;
        step(&mut w);
        if !tracked(&w, a) || st(&w, a).ground {
            break;
        }
        let now = w.obj(a).unwrap().pos.cell();
        let listed = w.flying.sector_of[&a];
        if carried != glasnost_num::F64::ZERO {
            checked += 1;
            assert_eq!(
                listed,
                flying::sector(&w, before),
                "the sector of the cell at the update's start: {}",
                describe(&w, a)
            );
            assert!(w.flying.sectors[&listed].contains(&a));
        }
        if listed != flying::sector(&w, now) {
            lagged += 1;
        }
    }
    assert!(checked > 20, "{checked} updates checked");
    assert!(lagged > 0, "the lag shows after a crossing");
}

// spec: S-0501 claim 47 (c) (no sector update while the carried speed fraction is 0: a hovering aircraft moved into another sector stays in the old sector's list until it moves on with a positive fraction; AC30 part f)
#[test]
fn claim_47_no_sector_update_at_a_zero_fraction() {
    let mut w = game(&rules());
    let a = put(&mut w, "HOVER", -6, 0);
    let first = testkit::mid(SIZE, 0, 0);
    move_to(&mut w, a, first);
    let stopped = run_until(&mut w, 2000, |w| {
        !st(w, a).ground
            && st(w, a).cur == glasnost_num::F64::ZERO
            && w.obj(a).unwrap().pos.cell() == first
    });
    assert!(stopped, "{}", describe(&w, a));
    let old = w.flying.sector_of[&a];
    assert_eq!(old, flying::sector(&w, first));
    // Two sectors east: the aircraft is put over that cell while its
    // carried fraction is 0. Its next update makes no sector update, though
    // it is in another sector; it then starts back with a positive fraction,
    // and the update after that brings the sector up to date.
    let other = testkit::mid(SIZE, 4, 0);
    assert_ne!(flying::sector(&w, other), old);
    {
        let p = Pos::centre(other, 0);
        let o = w.obj_mut(a).unwrap();
        o.pos.x = p.x;
        o.pos.y = p.y;
    }
    step(&mut w);
    assert_eq!(w.flying.sector_of[&a], old, "{}", describe(&w, a));
    let at = w.obj(a).unwrap().pos.cell();
    assert_eq!(flying::sector(&w, at), flying::sector(&w, other));
    assert_ne!(
        st(&w, a).cur,
        glasnost_num::F64::ZERO,
        "{}",
        describe(&w, a)
    );
    step(&mut w);
    assert_eq!(
        w.flying.sector_of[&a],
        flying::sector(&w, other),
        "{}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 47 (d) (aircraft that start being tracked in one tick join their sector's list in the order of the events, the tick's update order; nothing sorts them)
#[test]
fn claim_47_same_tick_starts_keep_the_update_order() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", 0, -1);
    let b = put(&mut w, "SCOUT", 0, 1);
    let ia = w.update.order.iter().position(|&x| x == a).unwrap();
    let ib = w.update.order.iter().position(|&x| x == b).unwrap();
    assert!(ia < ib, "a is updated first");
    // Order b first: the orders' sequence does not decide.
    move_to(&mut w, b, testkit::mid(SIZE, 0, 1));
    move_to(&mut w, a, testkit::mid(SIZE, 0, -1));
    move_to(&mut w, b, testkit::mid(SIZE, 10, 1));
    move_to(&mut w, a, testkit::mid(SIZE, 10, -1));
    assert!(run_until(&mut w, 20, |w| tracked(w, a) || tracked(w, b)));
    assert!(tracked(&w, a) && tracked(&w, b), "both in the same tick");
    let pos = |x| w.flying.order.iter().position(|&y| y == x).unwrap();
    assert!(pos(a) < pos(b), "in update order");
}

// spec: S-0501 claim 47 (b) (leaving the map ends tracking: going inside a transport or building)
#[test]
fn claim_47_going_inside_ends_tracking() {
    let mut w = game(&rules());
    let a = put(&mut w, "SCOUT", 0, 0);
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    assert!(run_until(&mut w, 100, |w| air::height(w, a) > 200));
    assert!(tracked(&w, a));
    glasnost_sim::objects::take_inside(&mut w, a);
    assert!(!tracked(&w, a));
}

// spec: S-0501 claim 47 (a) and (b) (an aircraft that starts on the ground is tracked exactly from its take-off start to its accepted touchdown: never while it rests on the ground, always while it flies), through several aircraft taking off, landing, hovering and taking off again on crossing move orders
#[test]
fn claim_47_never_tracked_on_the_ground_always_in_flight() {
    let mut w = game(&rules());
    let fleet: Vec<ObjectId> = (0..6)
        .map(|i| {
            let name = if i % 3 == 2 { "HOVER" } else { "SCOUT" };
            put(&mut w, name, -10 + 4 * i, -6)
        })
        .collect();
    let mut flights = 0;
    let mut landed = 0;
    for round in 0..4usize {
        for (i, &a) in fleet.iter().enumerate() {
            let dx = if (i + round) % 2 == 0 { 10 } else { -10 };
            let dy = 6 - 4 * (round % 3) as i32;
            move_to(&mut w, a, testkit::mid(SIZE, dx, dy));
        }
        for _ in 0..400 {
            step(&mut w);
            for &a in &fleet {
                let s = st(&w, a);
                assert_eq!(tracked(&w, a), !s.ground, "{}", describe(&w, a));
                if s.ground {
                    landed += 1;
                } else {
                    flights += 1;
                }
            }
        }
    }
    assert!(flights > 1000, "{flights} aircraft-ticks in flight");
    assert!(landed > 100, "{landed} aircraft-ticks landed");
}
