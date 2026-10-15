//! S-0206 claims 8 and 9: a vehicle's turret without a target.

use std::sync::Arc;

use glasnost_num::geom::direction;
use glasnost_sim::load::new_game;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Pos, Target};

// spec: S-0206 claim 8 (the turret facing is absolute: the body turns, the turret keeps or follows its own target)
// spec: S-0206 claim 9 (no target: the turret turns to the direction of the movement destination, else to the body)
#[test]
fn turret_rests_on_the_destination_then_the_body() {
    let mut r = testkit::ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(true);
    let rules = Arc::new(SimRules::new(&r, "turret"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(2),
    )
    .unwrap();
    // The rest rows need G + 5 ticks since the last shot (never fired: −100).
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let id = testkit::house_objects(&w, 0)
        .into_iter()
        .find(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == Kind::Vehicle && o.type_index == 0)
        })
        .unwrap();
    let dest = testkit::cell(-6, 12);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        },
    );
    for _ in 0..3 {
        tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    let p = Pos::centre(dest, 0);
    let want = direction(i64::from(p.x - o.pos.x), i64::from(p.y - o.pos.y), true);
    assert_eq!(o.turret.target, want);
    for _ in 0..800 {
        tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None, "the cart arrived");
    assert_eq!(o.turret.value(w.tick), o.body.value(w.tick));
}

fn turret_game(
    spins: bool,
    rot: i32,
) -> (glasnost_sim::world::World, glasnost_sim::world::ObjectId) {
    turret_game_q(spins, rot, true)
}

fn turret_game_q(
    spins: bool,
    rot: i32,
    quantised: bool,
) -> (glasnost_sim::world::World, glasnost_sim::world::ObjectId) {
    let mut r = testkit::ruleset();
    r.globals.quirks.turret_spin_quantised = quantised;
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.turret = Some(true);
    cart.rot = Some(rot);
    if spins {
        cart.turret_spins = Some(true);
    }
    let rules = Arc::new(SimRules::new(&r, "turret-sample"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(2),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let id = testkit::house_objects(&w, 0)
        .into_iter()
        .find(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == Kind::Vehicle && o.type_index == 0)
        })
        .unwrap();
    (w, id)
}

// spec: S-0206 claim 17 (a spinning turret's rotation sample is turning, though it never counts for the re-aim gate)
#[test]
fn rotation_sample_spinning_turret() {
    let (mut w, id) = turret_game(true, 1);
    for _ in 0..3 {
        tick::step(&mut w);
        assert!(w.obj(id).unwrap().rotation_sample);
    }
}

// spec: S-0206 claim 17 (a turret at rest samples not turning; a turn of positive duration begun by the rows samples turning)
#[test]
fn rotation_sample_follows_the_rows() {
    let (mut w, id) = turret_game(false, 1);
    for _ in 0..40 {
        tick::step(&mut w);
    }
    assert!(!w.obj(id).unwrap().rotation_sample, "at rest");
    let o = w.obj(id).unwrap();
    // A destination behind the vehicle: a turn of positive duration.
    let back = glasnost_map::CellXY::new(o.pos.cell().x - 6, o.pos.cell().y + 6);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(back)),
        },
    );
    let mut seen = false;
    for _ in 0..6 {
        tick::step(&mut w);
        let o = w.obj(id).unwrap();
        seen |= o.rotation_sample;
    }
    assert!(seen, "the turn was sampled");
}

/// The facing step a spinning turret makes per tick once it spins.
fn spin_step(quantised: bool, rot: i32) -> i32 {
    let (mut w, id) = turret_game_q(true, rot, quantised);
    for _ in 0..20 {
        tick::step(&mut w);
    }
    let a = w.obj(id).unwrap().turret.value(w.tick);
    tick::step(&mut w);
    let b = w.obj(id).unwrap().turret.value(w.tick);
    (i32::from(b) - i32::from(a)).rem_euclid(65536)
}

// spec: S-0206 claim 11 and Edge cases (quirks.turret_spin_quantised on: ω = 3 spins by 1024 per tick; off: by 256 × min(ω, 8) unrounded, capped at 2048, and not at all at ω = 0)
#[test]
fn turret_spin_quantised_both_settings() {
    assert_eq!(spin_step(true, 3), 1024);
    assert_eq!(spin_step(false, 3), 768);
    assert_eq!(spin_step(false, 7), 1792);
    assert_eq!(spin_step(false, 12), 2048);
    assert_eq!(spin_step(false, 0), 0);
}
