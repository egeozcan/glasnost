//! Claim tests for S-0311 revision 13, scenario X3 part f (absorbed
//! passengers of a building destroyed with no crew), on synthetic data:
//! invented types, costs and game-wide values, no stock numbers.
//!
//! The scenario destroys the building with the trigger action that
//! destroys attached objects (S-0011 claim 27). That action is not built
//! yet (`triggers.rs` notes it as a seam), so these tests deliver the hit
//! that claim 27 names through damage resolution: the building's current
//! health, no attacker and no attacking house, defences ignored, the rules'
//! demolition warhead. Everything after the hit is the engine's own path.

use glasnost_map::CellXY;
use glasnost_num::F32;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WarheadType};
use glasnost_sim::bomb;
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{testkit, tick};

const SIZE: i32 = 24;
const ZERO: u32 = 0; // 0.0
const QUARTER: u32 = 0x3E80_0000; // 0.25

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The combat ruleset with:
/// - BIOX: a 2 × 2 infantry absorber of cost 500, capacity 3, not eligible
///   for a delayed kill (the field left absent);
/// - PASS: an infantry type of cost 500 (the passengers);
/// - PLANTER: an infantry type of cost 1000 that can plant explosive
///   charges;
/// - the demolition warhead DEMO, a charge delay long enough that the
///   countdown is still running when the hit comes, veteran ratio 4 and
///   cap 2.5 (X3's values).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut b = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        infantry_absorb: Some(true),
        passengers: Some(3),
        size_limit: dec("1"),
        cost: Some(500),
        ..Default::default()
    };
    b.econ.power = Some(10);
    register(&mut r, "buildings", "BIOX");
    r.sim.buildings.insert("BIOX".into(), b);
    let walker = r.sim.infantry["WALKER"].clone();
    let mut pass = walker.clone();
    pass.cost = Some(500);
    register(&mut r, "infantry", "PASS");
    r.sim.infantry.insert("PASS".into(), pass);
    let mut planter = walker;
    planter.cost = Some(1000);
    planter.c4 = Some(true);
    register(&mut r, "infantry", "PLANTER");
    r.sim.infantry.insert("PLANTER".into(), planter);
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.c4_warhead = Some("DEMO".into());
    g.c4_delay = dec("1");
    g.veteran_ratio = dec("4");
    g.veteran_cap = dec("2.5");
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, house, c, 0)
}

fn until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

/// Sends a fresh passenger of house 1 into `b` (the enter order) and runs
/// until it is inside.
fn absorb_one(w: &mut World, b: ObjectId, dx: i32) -> ObjectId {
    let u = put(w, Kind::Infantry, "PASS", 1, at(dx, 3));
    orders::issue(
        w,
        glasnost_sim::world::HouseId(1),
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    assert!(
        until(w, 600, |w| absorb::absorbed_in(w, u) == Some(b)),
        "absorbed"
    );
    u
}

/// The hit of S-0011 claim 27 on `id`: damage equal to its current health,
/// no attacker and no attacking house, defences ignored, the demolition
/// warhead.
fn trigger_kill(w: &World, id: ObjectId) -> Hit {
    Hit {
        amount: w.obj(id).unwrap().health,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn exp(w: &World, id: ObjectId) -> F32 {
    w.obj(id).map_or(F32(ZERO), |o| o.combat.experience)
}

/// Builds X3 part f's scene: an enemy absorber holding two enemy
/// passengers, and a bystander of the human house. With `plant`, an
/// explosive-charge infantry unit of the human house plants a charge on the
/// building through the arrival path. Returns the world, the building, the
/// passengers, the planter (if any) and the bystander.
fn scene(plant: bool) -> (World, ObjectId, [ObjectId; 2], Option<ObjectId>, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let b = put(&mut w, Kind::Building, "BIOX", 1, at(0, 0));
    testkit::run(&mut w, 3);
    let p1 = absorb_one(&mut w, b, -2);
    let p2 = absorb_one(&mut w, b, 2);
    let bystander = put(&mut w, Kind::Infantry, "PLANTER", 0, at(-6, -6));
    let planter = plant.then(|| {
        let s = put(&mut w, Kind::Infantry, "PLANTER", 0, at(0, 0));
        let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
        let o = w.obj_mut(s).unwrap();
        o.beh.force(Behaviour::Sabotage, tick, stale);
        o.dest = Some(Target::Object(b));
        // spec: S-0519 claims 13 and 14 (the arrival plants the charge: a running countdown, the walker recorded as the planter)
        assert!(bomb::charge_arrival(&mut w, s));
        assert_eq!(w.obj(b).unwrap().combat.charge_planter, Some(s));
        s
    });
    (w, b, [p1, p2], planter, bystander)
}

// spec: S-0311 claim 9 (exception: the absorbed passengers of a building destroyed with no crew are credited to its charge planter, or to no one) and scenario X3 part f run 1
// spec: S-0312 claims 18 and 24 (damage that ignores defences and has no attacker leaves the building with no crew; every absorbed passenger dies instead of coming out)
#[test]
fn x3_f_run_1_no_planter_no_experience() {
    let (mut w, b, passengers, _, bystander) = scene(false);
    let before: Vec<F32> = w.objects.values().map(|o| o.combat.experience).collect();
    let h = trigger_kill(&w, b);
    assert_eq!(damage::apply(&mut w, b, h), HitResult::NowDead);
    testkit::run(&mut w, 3);
    // spec: S-0312 claim 24 (no crew: no passenger comes out)
    for p in passengers {
        assert!(
            w.obj(p).is_none_or(|o| !o.on_map && o.health <= 0),
            "the passenger dies inside"
        );
    }
    // spec: S-0311 claim 9 (no planter: nobody gains experience for the passengers, and the building's destruction has no attacker)
    assert_eq!(exp(&w, bystander), F32(ZERO));
    let after: Vec<F32> = w
        .objects
        .values()
        .filter(|o| !passengers.contains(&o.id) && o.id != b)
        .map(|o| o.combat.experience)
        .collect();
    assert!(after.iter().all(|e| *e == F32(ZERO)), "{after:?}");
    assert!(before.iter().all(|e| *e == F32(ZERO)));
}

// spec: S-0311 claims 8, 9 and 10 and F3 (the charge planter gains 500 / (1000 × 4) per passenger, 0.25 for two, and nothing for the building, whose destruction has no attacker) and scenario X3 part f run 2
// spec: S-0312 claim 24 (credited to the building's charge planter) and S-0302 claim 16 (a building not eligible for a delayed kill is destroyed by the hit while the charge's countdown runs)
#[test]
fn x3_f_run_2_the_planter_gains_for_the_passengers_only() {
    let (mut w, b, passengers, planter, bystander) = scene(true);
    let planter = planter.unwrap();
    // The countdown is still running: run 2 is valid.
    let until_tick = w.obj(b).unwrap().combat.delay_kill_until.unwrap();
    assert!(until_tick > w.tick + 10);
    assert_eq!(exp(&w, planter), F32(ZERO));
    let h = trigger_kill(&w, b);
    assert_eq!(damage::apply(&mut w, b, h), HitResult::NowDead);
    assert!(
        w.tick < until_tick,
        "the hit comes while the countdown runs"
    );
    testkit::run(&mut w, 3);
    for p in passengers {
        assert!(
            w.obj(p).is_none_or(|o| !o.on_map && o.health <= 0),
            "the passenger dies inside"
        );
    }
    // spec: S-0311 claim 9 and F3 (0.125 per passenger, 0.25 in all: veteran ratio 4, planter cost 1000, passenger cost 500)
    assert_eq!(exp(&w, planter), F32(QUARTER));
    // spec: S-0311 claim 9 (no other object's experience changes)
    assert_eq!(exp(&w, bystander), F32(ZERO));
    let others: Vec<F32> = w
        .objects
        .values()
        .filter(|o| o.id != planter && !passengers.contains(&o.id) && o.id != b)
        .map(|o| o.combat.experience)
        .collect();
    assert!(others.iter().all(|e| *e == F32(ZERO)), "{others:?}");
}
