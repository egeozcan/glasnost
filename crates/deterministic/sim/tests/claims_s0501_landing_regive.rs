//! Claim test for S-0501 claims 8 and 40 (#159): a *move*'s next
//! processing gives the flight point to the flight once more, and an
//! aircraft already landing at that point does not take off (claim 8's
//! exception, tested whenever a destination is handed to the flight).
//! Synthetic data through the real tick path; every type, weapon and number
//! is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// JET: an armed, landable aircraft that is not pad-bound (invented values).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
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
    let mut jet = UnitType {
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
    jet.combat.ammo = Some(2);
    jet.econ.landable = Some(true);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("JET".into());
    r.sim.aircraft.insert("JET".into(), jet);
    r
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn line(w: &World, a: ObjectId) -> String {
    let o = w.obj(a).unwrap();
    let s = st(w, a);
    format!(
        "t={} h={} cell={:?} beh={:?} last={:?} dest={:?} flying_to={:?} landing={} taking_off={}",
        w.tick,
        air::height(w, a),
        o.pos.cell(),
        o.beh.current,
        o.beh.last,
        o.dest,
        s.flying_to,
        s.landing,
        s.taking_off
    )
}

/// Flies JET in *attack* toward a far target until it holds cruise height,
/// then orders it to move `ahead` cells ahead; returns the world, the
/// aircraft and the destination cell.
fn moved_in_flight(ahead: i32) -> (World, ObjectId, CellXY) {
    let mut w = game(&rules());
    ready(&mut w);
    let a = testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, testkit::mid(SIZE, -10, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "TANK", 1, testkit::mid(SIZE, 14, 0), 0);
    step(&mut w);
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
    assert!(cruise, "{}", line(&w, a));
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Attack));
    let c = w.obj(a).unwrap().pos.cell();
    let g = CellXY::new(c.x + ahead, c.y);
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(g)),
        }
    ));
    (w, a, g)
}

// spec: S-0501 claim 40 (a move's next processing gives its flight point to the flight once more; a landing aircraft takes off, except one whose destination lies in the cell of its flight point) and claim 8 (the exception: no take-off, its flight point stays as it was and it goes on landing there; the test is made at each processing of move that gives one)
#[test]
fn a_moves_next_processing_does_not_take_off_an_aircraft_landing_at_its_destination() {
    for ahead in [1, 2] {
        let (mut w, a, g) = moved_in_flight(ahead);
        let mut landed = false;
        let mut landing_seen = false;
        for _ in 0..900 {
            let was = st(&w, a);
            let before = line(&w, a);
            let h0 = air::height(&w, a);
            step(&mut w);
            let s = st(&w, a);
            if was.landing {
                landing_seen = true;
                assert!(
                    !s.taking_off && air::height(&w, a) <= h0,
                    "{ahead} ahead: a take-off while landing at its destination:\n  before: {before}\n  after:  {}",
                    line(&w, a)
                );
            }
            if s.ground {
                landed = true;
                break;
            }
        }
        assert!(landing_seen, "{ahead} ahead: {}", line(&w, a));
        assert!(landed, "{ahead} ahead: {}", line(&w, a));
        assert_eq!(w.obj(a).unwrap().pos.cell(), g, "{ahead} ahead");
    }
}
