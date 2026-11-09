//! Claim tests for S-0501 r150 (Q-2726 answered, M8 intake 112): an
//! aircraft in *sticky* is processed as in *guard* (claim 29), with every
//! queued behaviour waiting (claim 31) and every destination, contact and
//! crash taking effect at once (scenario AC33 parts a to c); claim 29's
//! delay of 1 tick at cruise height; claim 29's pad search on the ground;
//! claim 33's *move* at the movement update for *guard* itself only.
//! Synthetic data through the real tick path; every type, weapon, map and
//! number is invented. Ledger: `docs/m8-claims/intake112.md`.

mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
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

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with one docking place); KITE (unarmed, landable, not pad-bound); BOUND
/// (armed, landable, pad-bound, 2 rounds, docking at APAD); JET (as BOUND,
/// not pad-bound).
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
    push(&mut r, "aircraft", "KITE");
    r.sim.aircraft.insert("KITE".into(), plane(false));
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), plane(true));
    let mut bound = plane(true);
    bound.econ.airport_bound = Some(true);
    push(&mut r, "aircraft", "BOUND");
    r.sim.aircraft.insert("BOUND".into(), bound);
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn describe(w: &World, id: ObjectId) -> String {
    let Some(o) = w.obj(id) else {
        return format!("t={} gone", w.tick);
    };
    format!(
        "t={} cell={:?} h={} hp={} ammo={} beh={:?}/{:?} last={:?} dest={:?} contact={:?}",
        w.tick,
        o.pos.cell(),
        air::height(w, id),
        o.health,
        o.combat.ammo,
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
    )
}

fn cruise(w: &World, a: ObjectId) -> i32 {
    air::cruise_height(w, a)
}

fn alive(w: &World, a: ObjectId) -> bool {
    w.obj(a).is_some_and(|o| o.health > 0)
}

/// AC33's set-up: a `name` aircraft of house 0 on the ground at the west
/// of the map with `beh` current, then given a move order to *C*, eleven
/// cells east of the middle, applied by the end of the returned world's
/// tick. With `pad`, an APAD of house 0 far north of the line. Returns
/// (world, aircraft, C, pad).
fn ordered(name: &str, beh: Behaviour, pad: bool) -> (World, ObjectId, CellXY, Option<ObjectId>) {
    let mut w = game(&rules());
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

// spec: S-0501 claim 29 (r150, Q-2726: a processing of sticky has guard's outcomes; an unarmed aircraft outside a team is given the cell it is over at each processing at cruise height, handed to the flight at once, claim 8, and a landable type with a positive slowdown distance slows to a stop and lands there; its move stays queued) and claim 31 (r150: an aircraft in sticky is never ready to change, so the order's move stays queued); scenario AC33 part a
#[test]
fn ac33_a_an_unarmed_aircraft_in_sticky_lands_short_of_its_order() {
    let (mut w, a, c, _) = ordered("KITE", Behaviour::Sticky, false);
    let o = w.obj(a).unwrap();
    assert_eq!(
        (o.beh.current, o.beh.queued, o.dest),
        (
            Some(Behaviour::Sticky),
            Some(Behaviour::Move),
            Some(Target::Cell(c))
        ),
        "at the end of the order's tick: {}",
        describe(&w, a)
    );
    let mut own_cell = false;
    let mut reached = false;
    let mut airborne = false;
    let mut landed_at = None;
    for _ in 0..1500 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert_eq!(
            (o.beh.current, o.beh.queued),
            (Some(Behaviour::Sticky), Some(Behaviour::Move)),
            "move never current: {}",
            describe(&w, a)
        );
        if air::height(&w, a) > 0 {
            airborne = true;
            assert!(
                landed_at.is_none(),
                "no second take-off: {}",
                describe(&w, a)
            );
        }
        if o.pos.cell() == c {
            reached = true;
        }
        if let Some(Target::Cell(d)) = o.dest
            && d != c
        {
            own_cell = true;
        }
        if airborne && landed_at.is_none() && air::on_ground(&w, a) {
            landed_at = Some(o.pos.cell());
        }
    }
    assert!(airborne, "it takes off: {}", describe(&w, a));
    assert!(own_cell, "given the cell it is over: {}", describe(&w, a));
    assert!(!reached, "it never reaches C: {}", describe(&w, a));
    assert!(landed_at.is_some(), "it lands: {}", describe(&w, a));
    // Control: in guard it lands at C.
    let (mut w, a, c, _) = ordered("KITE", Behaviour::Guard, false);
    let mut at_c = false;
    for _ in 0..1500 {
        step(&mut w);
        if w.obj(a).unwrap().pos.cell() == c {
            at_c = true;
        }
    }
    assert!(at_c, "control: {}", describe(&w, a));
}

// spec: S-0501 claim 29 (r150: outside a team, a processing at exactly cruise height ends with a delay of 1 tick, for sticky as for guard)
#[test]
fn claim_29_a_processing_at_cruise_height_chooses_1_tick() {
    let (mut w, a, _, _) = ordered("KITE", Behaviour::Sticky, false);
    let mut seen = false;
    for _ in 0..600 {
        // A processing in a tick that starts and ends at cruise height.
        let (h0, last0) = (air::height(&w, a), w.obj(a).unwrap().beh.last);
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.beh.last != last0
            && h0 == cruise(&w, a)
            && air::height(&w, a) == cruise(&w, a)
            && !air::on_ground(&w, a)
            && let Some((_, d)) = o.beh.last
        {
            assert_eq!(d, 1, "{}", describe(&w, a));
            seen = true;
            break;
        }
    }
    assert!(seen, "a processing at cruise height: {}", describe(&w, a));
}

// spec: S-0501 claim 29 (r150: for an armed aircraft outside a team at exactly cruise height the fall-back's own effects take place at each such processing in sticky: a pad-bound aircraft with dock types that no pad grants contact crashes, claim 46, at its first processing at that height); scenario AC33 part b
#[test]
fn ac33_b_a_pad_bound_aircraft_in_sticky_with_no_pad_crashes() {
    let (mut w, a, c, _) = ordered("BOUND", Behaviour::Sticky, false);
    assert_eq!(
        w.obj(a).unwrap().beh.queued,
        Some(Behaviour::Move),
        "{}",
        describe(&w, a)
    );
    let mut first_cruise = None;
    let mut gone = None;
    for i in 0..1500 {
        step(&mut w);
        if !alive(&w, a) || w.obj(a).is_none_or(|o| !o.on_map) {
            gone = Some(i);
            break;
        }
        let o = w.obj(a).unwrap();
        assert_ne!(o.pos.cell(), c, "it never reaches C");
        if first_cruise.is_none() && air::height(&w, a) == cruise(&w, a) {
            first_cruise = Some(i);
        }
        if air::crashing(&w, a) {
            gone = Some(i);
            break;
        }
    }
    let (Some(f), Some(g)) = (first_cruise, gone) else {
        panic!(
            "cruise {first_cruise:?} crash {gone:?}: {}",
            describe(&w, a)
        );
    };
    assert!(g - f <= 45, "within 45 ticks of cruise height: {f} {g}");
}

// spec: S-0501 claim 29 (r150: one that a pad grants contact at that processing turns toward that pad, with enter queued, sticky still current); scenario AC33 part c; claim 29 (r167, Q-2802: a contact held at cruise height, each later processing at that height is the fall-back again, which keeps the contact and the pad as destination with enter queued)
#[test]
fn ac33_c_with_a_pad_it_gets_the_pad_and_its_contact() {
    let (mut w, a, _, k) = ordered("BOUND", Behaviour::Sticky, true);
    let k = k.unwrap();
    let mut done = false;
    for _ in 0..1500 {
        step(&mut w);
        let Some(o) = w.obj(a) else { break };
        if o.contact == Some(k) {
            assert_eq!(
                (o.beh.current, o.beh.queued, o.dest),
                (
                    Some(Behaviour::Sticky),
                    Some(Behaviour::Enter),
                    Some(Target::Object(k))
                ),
                "{}",
                describe(&w, a)
            );
            done = true;
            break;
        }
    }
    assert!(done, "a contact with the pad: {}", describe(&w, a));
    assert!(!air::crashing(&w, a), "no crash");
    // spec: S-0501 claim 29 (r167, Q-2802: with that pad the only one of its dock types that accepts it, the pad confirms: the aircraft keeps the contact and the pad as destination, with enter queued; the moment with no destination leaves nothing but its draws)
    let mut later = 0;
    for _ in 0..20 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        if air::height(&w, a) != cruise(&w, a) {
            break;
        }
        later += 1;
        assert_eq!(
            (o.contact, o.dest, o.beh.current, o.beh.queued),
            (
                Some(k),
                Some(Target::Object(k)),
                Some(Behaviour::Sticky),
                Some(Behaviour::Enter)
            ),
            "{}",
            describe(&w, a)
        );
    }
    assert!(
        later > 1,
        "processings at cruise height follow: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 29 (r150: an armed aircraft below cruise height, less than full ammunition and no contact, that finds a pad takes off for it if it is on the ground, with enter queued and its target forgotten)
#[test]
fn claim_29_the_pad_search_is_made_on_the_ground_too() {
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
        o.beh.force(Behaviour::Guard, t, true);
        o.beh.queued = None;
    }
    let mut found = false;
    for _ in 0..60 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.dest == Some(Target::Object(k)) {
            assert_eq!(
                o.beh.queued.or(o.beh.current),
                Some(Behaviour::Enter),
                "{}",
                describe(&w, a)
            );
            assert_eq!(o.target, None);
            found = true;
            break;
        }
    }
    assert!(found, "the pad found from the ground: {}", describe(&w, a));
}

// spec: S-0501 claim 33 (r150, Q-2726: the movement update's test is for guard itself as current; an aircraft in sticky with a destination and nothing queued gets no move there and keeps sticky with its destination)
#[test]
fn claim_33_sticky_gets_no_move_at_a_movement_update() {
    let mut w = game(&rules());
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "KITE",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    let to = CellXY::new(c.x + 6, c.y);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(Behaviour::Sticky, t, true);
        o.beh.queued = None;
        o.dest = Some(Target::Cell(to));
    }
    step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Sticky),
        "{}",
        describe(&w, a)
    );
    assert_ne!(o.beh.queued, Some(Behaviour::Move), "{}", describe(&w, a));
}
