//! Claim tests for S-0314 claims 12 to 15 (automatic building repair and
//! sale) on synthetic data: invented building and vehicle types, no stock
//! values. Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied; the neutral (2) and special (3) houses where a test needs them.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn building() -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(1000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b
}

/// HUT and SHED (plain buildings), YARD (a factory for buildings). Both
/// automatic thresholds 0, the reserve 100, the game's tech level 51 (so
/// every sale draw succeeds) unless a test says otherwise.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["HUT", "SHED", "YARD"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    let s = &mut r.sim;
    s.buildings.insert("HUT".into(), building());
    s.buildings.insert("SHED".into(), building());
    let mut yard = building();
    yard.factory = Some("building".into());
    s.buildings.insert("YARD".into(), yard);
    s.general.iq_repair_sell = Some(0);
    s.general.iq_sell_back = Some(0);
    s.general.credit_reserve = Some(100);
    s.general.multiplayer_tech_level = Some(51);
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with_non_players(r, SIZE, 13);
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, name: &str, house: u8, cell: CellXY, health: i32) -> ObjectId {
    let id = testkit::spawn(w, Kind::Building, name, house, cell, 0);
    w.obj_mut(id).unwrap().health = health;
    id
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// Steps to the tick before a multiple of the repair period, so that the
/// next step's turn is not a claim-8 step tick.
fn off_period(w: &mut World) {
    let pb = w.rules().combat.general.repair_period;
    while (w.tick + 1) % pb == 0 || w.tick % pb == 0 {
        step(w);
    }
}

fn repairing(w: &World, id: ObjectId) -> bool {
    w.obj(id).unwrap().econ.building.repairing
}

fn selling(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_none_or(|o| o.beh.seen() == Some(Behaviour::Selling))
}

/// A hit of `amount` that ignores defences, from `attacker`.
fn hit_from(amount: i32, attacker: ObjectId) -> Hit {
    Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker: Some(attacker),
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

// spec: S-0314 claim 12 (credits at least the reserve start repair; equality belongs to repair; below it no start)
#[test]
fn claim_12_reserve_split() {
    for (credits, starts) in [(99, false), (100, true), (101, true)] {
        let mut r = rules();
        // The sale branch is closed: the sale threshold 1 fails for sale skill 0.
        r.sim.general.iq_sell_back = Some(1);
        let mut w = game(&r);
        off_period(&mut w);
        let hut = put(&mut w, "HUT", 0, c(0, 0), 200);
        w.houses[0].credits = credits;
        step(&mut w);
        assert_eq!(repairing(&w, hut), starts, "credits {credits}");
    }
}

// spec: S-0314 claim 12 (a full-health building is ineligible; a building being sold is ineligible)
#[test]
fn claim_12_eligibility() {
    let mut w = game(&rules());
    off_period(&mut w);
    let full = put(&mut w, "HUT", 0, c(0, 0), 400);
    step(&mut w);
    assert!(!repairing(&w, full), "full health");
    let mut w = game(&rules());
    off_period(&mut w);
    // Full health while the order is applied (orders follow the turn pass),
    // then damaged while selling.
    let sold = put(&mut w, "HUT", 0, c(0, 0), 400);
    assert!(glasnost_sim::orders::issue(
        &mut w,
        A,
        glasnost_sim::orders::Order::Sell { actor: sold }
    ));
    step(&mut w);
    assert!(selling(&w, sold));
    if let Some(o) = w.obj_mut(sold) {
        o.health = 200;
    }
    step(&mut w);
    assert!(w.obj(sold).is_none_or(|o| !o.econ.building.repairing));
}

// spec: S-0314 claim 8 (the gate: the house skill level 0 must be at least the repair-and-sell threshold)
#[test]
fn claim_12_skill_gate() {
    let mut r = rules();
    r.sim.general.iq_repair_sell = Some(1);
    let mut w = game(&r);
    off_period(&mut w);
    let hut = put(&mut w, "HUT", 0, c(0, 0), 200);
    step(&mut w);
    assert!(!repairing(&w, hut));
}

// spec: S-0314 claim 13 (a start takes no payment and changes no health by itself; it sets the house's hold)
#[test]
fn claim_13_start_takes_no_payment() {
    let mut w = game(&rules());
    off_period(&mut w);
    let hut = put(&mut w, "HUT", 0, c(0, 0), 200);
    let before = w.houses[0].credits;
    // Look at the hold between the turn pass and the house update: stop
    // after one step and check that the start left health and money alone.
    step(&mut w);
    assert!(repairing(&w, hut));
    assert_eq!(w.obj(hut).unwrap().health, 200);
    assert_eq!(w.houses[0].credits, before);
}

// spec: S-0314 claim 13 (neutral and special houses need the computer-repair flag or the captured mark)
#[test]
fn claim_13_neutral_needs_flag_or_capture() {
    let mut r = rules();
    r.sim.general.credit_reserve = Some(0);
    let mut w = game(&r);
    off_period(&mut w);
    let plain = put(&mut w, "HUT", 2, c(0, 0), 200);
    let flagged = put(&mut w, "HUT", 3, c(3, 0), 200);
    w.obj_mut(flagged).unwrap().combat.computer_repair = true;
    let captured = put(&mut w, "SHED", 2, c(6, 0), 200);
    step(&mut w);
    assert!(!repairing(&w, plain));
    assert!(repairing(&w, flagged));
    assert!(!repairing(&w, captured));
    // spec: S-0314 Terms (captured mark: a successful ownership change sets it)
    w.obj_mut(captured).unwrap().owner = Some(HouseId(3));
    glasnost_sim::econ::events::owner_changed(&mut w, captured, Some(HouseId(2)));
    assert!(w.obj(captured).unwrap().combat.captured);
    off_period(&mut w);
    step(&mut w);
    assert!(repairing(&w, captured));
    assert!(!repairing(&w, plain));
}

// spec: S-0314 claim 15 (one automatic start per house per tick; the house update releases the hold for the next tick)
#[test]
fn claim_15_one_start_per_house_per_tick() {
    let mut w = game(&rules());
    off_period(&mut w);
    let a1 = put(&mut w, "HUT", 0, c(0, 0), 200);
    let a2 = put(&mut w, "SHED", 0, c(3, 0), 200);
    let b1 = put(&mut w, "HUT", 1, c(6, 0), 200);
    step(&mut w);
    assert_eq!(
        [repairing(&w, a1), repairing(&w, a2)],
        [true, false],
        "the first in object-turn order starts; the hold blocks the second"
    );
    assert!(repairing(&w, b1), "another house is not blocked");
    assert!(
        !w.houses[0].combat.repair_hold,
        "released by the house update"
    );
    step(&mut w);
    assert!(repairing(&w, a2), "starts in the following tick");
}

// spec: S-0314 claim 15 (repair turned off by order is started again on a later eligible turn)
#[test]
fn claim_15_restart_after_order_off() {
    let mut w = game(&rules());
    off_period(&mut w);
    let hut = put(&mut w, "HUT", 0, c(0, 0), 200);
    step(&mut w);
    assert!(repairing(&w, hut));
    w.obj_mut(hut).unwrap().econ.building.repairing = false;
    step(&mut w);
    assert!(repairing(&w, hut));
}

// spec: S-0314 Terms (hostile-hit mark: a surviving hit with a known attacker not allied; healing does not clear it; an allied hit does not set it)
#[test]
fn hostile_hit_mark() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0), 400);
    let friend = put(&mut w, "SHED", 0, c(3, 0), 400);
    let enemy = put(&mut w, "SHED", 1, c(6, 0), 400);
    damage::apply(&mut w, hut, hit_from(10, friend));
    assert!(!w.obj(hut).unwrap().combat.hostile_hit, "allied attacker");
    let mut anon = hit_from(10, enemy);
    anon.attacker = None;
    damage::apply(&mut w, hut, anon);
    assert!(!w.obj(hut).unwrap().combat.hostile_hit, "no known attacker");
    damage::apply(&mut w, hut, hit_from(10, enemy));
    assert!(w.obj(hut).unwrap().combat.hostile_hit);
    w.obj_mut(hut).unwrap().health = 400;
    assert!(w.obj(hut).unwrap().combat.hostile_hit, "healing keeps it");
}

/// A late-test setup applied to the building.
type Prep = fn(&mut World, ObjectId);

/// A damaged HUT of Alpha below the reserve, with the hostile-hit mark.
fn below_reserve(r: &Ruleset, health: i32) -> (World, ObjectId) {
    let mut w = game(r);
    off_period(&mut w);
    let hut = put(&mut w, "HUT", 0, c(0, 0), health);
    w.obj_mut(hut).unwrap().combat.hostile_hit = true;
    w.houses[0].credits = 0;
    (w, hut)
}

// spec: S-0314 claim 14 (below the reserve a building with the hostile-hit mark is sold when the draw is below the tech level and the ratio below red)
#[test]
fn claim_14_sale() {
    let (mut w, hut) = below_reserve(&rules(), 100);
    step(&mut w);
    assert!(selling(&w, hut));
    assert!(!repairing(&w, hut), "the sale branch never tries repair");
}

// spec: S-0314 claim 14 (no hostile-hit mark: no draw, no sale)
#[test]
fn claim_14_needs_hostile_mark() {
    let (mut w, hut) = below_reserve(&rules(), 100);
    w.obj_mut(hut).unwrap().combat.hostile_hit = false;
    step(&mut w);
    assert!(!selling(&w, hut));
    assert!(
        !repairing(&w, hut),
        "failing the sale tests does not try repair"
    );
}

// spec: S-0314 claim 14 (the draw is taken even when a late test rejects the sale: ratio equal to red, a tag, a building factory, a delayed kill)
#[test]
fn claim_14_late_rejections_still_draw() {
    let cases: [(&str, i32, Prep); 4] = [
        ("ratio equal to red", 200, |_, _| {}),
        ("tag", 100, |w, id| {
            w.triggers.objects.insert(id, 0);
        }),
        ("delayed kill", 100, |w, id| {
            w.obj_mut(id).unwrap().combat.delay_kill_until = Some(i32::MAX);
        }),
        ("building factory", -1, |_, _| {}),
    ];
    for (name, health, prep) in cases {
        let (mut w, id) = if health < 0 {
            let mut w = game(&rules());
            off_period(&mut w);
            let y = put(&mut w, "YARD", 0, c(0, 0), 100);
            w.obj_mut(y).unwrap().combat.hostile_hit = true;
            w.houses[0].credits = 0;
            (w, y)
        } else {
            below_reserve(&rules(), health)
        };
        prep(&mut w, id);
        let mut unmarked = w.clone();
        unmarked.obj_mut(id).unwrap().combat.hostile_hit = false;
        step(&mut w);
        step(&mut unmarked);
        assert!(!selling(&w, id), "{name}: no sale");
        assert_ne!(w.rng, unmarked.rng, "{name}: the sale draw is still taken");
    }
}

// spec: S-0314 claim 14 (tech level 0: the draw is taken and never succeeds)
#[test]
fn claim_14_tech_level_zero_draws() {
    let mut r = rules();
    r.sim.general.multiplayer_tech_level = Some(0);
    let (mut w, hut) = below_reserve(&r, 100);
    let mut unmarked = w.clone();
    unmarked.obj_mut(hut).unwrap().combat.hostile_hit = false;
    for _ in 0..5 {
        step(&mut w);
        step(&mut unmarked);
    }
    assert!(!selling(&w, hut));
    assert_ne!(w.rng, unmarked.rng);
}

// spec: S-0314 claim 14 (the sale threshold read as unsigned: 1 and −1 both fail for sale skill 0, with no draw)
#[test]
fn claim_14_unsigned_threshold() {
    for t in [1, -1] {
        let mut r = rules();
        r.sim.general.iq_sell_back = Some(t);
        let (mut w, hut) = below_reserve(&r, 100);
        let mut unmarked = w.clone();
        unmarked.obj_mut(hut).unwrap().combat.hostile_hit = false;
        step(&mut w);
        step(&mut unmarked);
        assert!(!selling(&w, hut), "threshold {t}");
        assert_eq!(w.rng, unmarked.rng, "threshold {t}: no draw");
    }
}

// spec: S-0314 claim 15 (a step paid by an earlier building changes the credits a later building's split sees)
#[test]
fn claim_15_credit_order() {
    let mut r = rules();
    r.sim.general.repair_step = Some(10);
    let mut w = game(&r);
    let pb = w.rules().combat.general.repair_period;
    while w.tick % pb != 0 {
        step(&mut w);
    }
    let a = put(&mut w, "HUT", 0, c(0, 0), 200);
    let b = put(&mut w, "SHED", 0, c(3, 0), 100);
    w.obj_mut(a).unwrap().econ.building.repairing = true;
    w.obj_mut(b).unwrap().combat.hostile_hit = true;
    let cost = {
        let before = w.clone();
        let mut probe = before.clone();
        // B at full health is ineligible, so only A's step is paid.
        probe.obj_mut(b).unwrap().health = 400;
        probe.houses[0].credits = 10_000;
        step(&mut probe);
        10_000 - probe.houses[0].credits
    };
    assert!(cost > 0, "A's step costs something");
    w.houses[0].credits = 100;
    step(&mut w);
    assert_eq!(w.houses[0].credits, 100 - cost);
    assert!(
        selling(&w, b),
        "B saw credits below the reserve after A paid"
    );
}
