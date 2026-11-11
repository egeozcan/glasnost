//! Claim tests for orders (S-0213) and behaviours (S-0215), on the
//! synthetic scenario.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_sim::behaviour::Door;
use glasnost_sim::load::new_game;
use glasnost_sim::objects;
use glasnost_sim::orders::{self, Order, PENDING_LIMIT};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::targeting::{self, KeyCommand};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{AttackMoveMemory, HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);
const BETA: HouseId = HouseId(1);

fn ready(seed: u32) -> World {
    let mut w = testkit::new_game(seed);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    w
}

fn of(w: &World, h: u8, kind: Kind, type_index: u32) -> Vec<ObjectId> {
    testkit::house_objects(w, h)
        .into_iter()
        .filter(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == kind && o.type_index == type_index)
        })
        .collect()
}

fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

fn beh(actor: ObjectId, b: Behaviour, target: Option<Target>, dest: Option<Target>) -> Order {
    Order::Behaviour {
        actor,
        behaviour: b,
        attack_move: false,
        target,
        dest,
    }
}

// ---------------------------------------------------------------------------
// S-0213

// spec: S-0213 claim 1 (orders apply between ticks, after the turn pass of the tick they were issued in)
// spec: S-0213 claim 2 (at most 128 pending orders per house; the 129th is dropped)
#[test]
fn s0213_pending_limit() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    for _ in 0..PENDING_LIMIT {
        assert!(orders::issue(&mut w, ALPHA, Order::Stop { actor: id }));
    }
    assert!(!orders::issue(&mut w, ALPHA, Order::Stop { actor: id }));
    // Another house has its own list (its order for its own vehicle, S-0213 claim 15).
    let theirs = of(&w, 1, Kind::Vehicle, 0)[0];
    assert!(orders::issue(&mut w, BETA, Order::Stop { actor: theirs }));
    tick::step(&mut w);
    assert!(w.houses.iter().all(|h| h.pending.is_empty()));
}

// spec: S-0213 claim 2 (quirks.pending_order_limit off: no limit)
#[test]
fn s0213_pending_limit_switch() {
    let mut r = testkit::ruleset();
    r.globals.quirks.pending_order_limit = false;
    let rules = Arc::new(SimRules::new(&r, "nolimit"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    for _ in 0..200 {
        assert!(orders::issue(&mut w, ALPHA, Order::Stop { actor: id }));
    }
}

// spec: S-0213 claim 3 (a stop given after a move in the same tick leaves the move in force)
#[test]
fn s0213_behaviour_orders_after_the_others() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    let c = testkit::cell(0, 7);
    orders::issue(&mut w, ALPHA, mv(id, c));
    orders::issue(&mut w, ALPHA, Order::Stop { actor: id });
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert_eq!(o.dest, Some(Target::Cell(c)));
}

// spec: S-0213 claim 4 (an order whose target is no longer valid has no effect)
#[test]
fn s0213_target_checks() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    let enemy = of(&w, 1, Kind::Vehicle, 0)[0];
    let before = w.obj(id).unwrap().clone();
    orders::issue(
        &mut w,
        ALPHA,
        beh(id, Behaviour::Attack, Some(Target::Object(enemy)), None),
    );
    objects::remove(&mut w, enemy);
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.target, before.target);
    assert_eq!(o.beh.queued, None);
}

// spec: S-0213 claim 5 (table T3 (e): area guard takes the target as destination and guard spot, target nothing)
// spec: S-0213 claim 5 (table T3 (f), (g): another behaviour sets target and destination and forgets the guard spot)
#[test]
fn s0213_table_t3() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    let c = testkit::cell(-3, 3);
    orders::issue(
        &mut w,
        ALPHA,
        beh(
            id,
            Behaviour::AreaGuard,
            Some(Target::Cell(c)),
            Some(Target::Cell(testkit::cell(5, 5))),
        ),
    );
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(
        (o.target, o.dest, o.guard_spot),
        (None, Some(Target::Cell(c)), Some(Target::Cell(c)))
    );
    let d = testkit::cell(2, 2);
    orders::issue(&mut w, ALPHA, mv(id, d));
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(
        (o.target, o.dest, o.guard_spot),
        (None, Some(Target::Cell(d)), None)
    );
}

// spec: S-0213 claim 6 (attack-move: an eligible actor gets move with the memory of the destination, or attack with the memory of the target)
#[test]
fn s0213_attack_move() {
    let mut w = ready(1);
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let enemy = of(&w, 1, Kind::Vehicle, 0)[0];
    let c = testkit::cell(0, 5);
    let am = |actor, target, dest| Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: true,
        target,
        dest,
    };
    orders::issue(&mut w, ALPHA, am(cart, None, Some(Target::Cell(c))));
    // The unarmed crusher is not eligible: a target only gives a plain attack.
    orders::issue(
        &mut w,
        ALPHA,
        am(crusher, Some(Target::Object(enemy)), None),
    );
    tick::step(&mut w);
    let o = w.obj(cart).unwrap();
    assert_eq!(
        (o.beh.queued, o.attack_move),
        (
            Some(Behaviour::Move),
            Some(AttackMoveMemory::Dest(Target::Cell(c)))
        )
    );
    let o = w.obj(crusher).unwrap();
    assert_eq!(
        (o.beh.queued, o.attack_move),
        (Some(Behaviour::Attack), None)
    );
    orders::issue(&mut w, ALPHA, am(cart, Some(Target::Object(enemy)), None));
    tick::step(&mut w);
    let o = w.obj(cart).unwrap();
    assert_eq!(
        o.attack_move,
        Some(AttackMoveMemory::Target(Target::Object(enemy)))
    );
    // Any other behaviour order removes the memory.
    orders::issue(&mut w, ALPHA, mv(cart, c));
    tick::step(&mut w);
    assert_eq!(w.obj(cart).unwrap().attack_move, None);
}

// spec: S-0213 claim 7 (a run of identical move orders to one cell is spread; the leader keeps the common cell)
#[test]
fn s0213_group_destinations() {
    let mut w = ready(1);
    let carts = of(&w, 0, Kind::Vehicle, 0);
    let c = testkit::cell(0, 8);
    for &id in &carts {
        orders::issue(&mut w, ALPHA, mv(id, c));
    }
    tick::step(&mut w);
    let dests: Vec<Option<Target>> = carts.iter().map(|&id| w.obj(id).unwrap().dest).collect();
    assert!(dests.contains(&Some(Target::Cell(c))), "{dests:?}");
    let mut distinct = dests.clone();
    distinct.sort();
    distinct.dedup();
    assert!(distinct.len() > 1, "{dests:?}");
}

// spec: S-0213 claim 8 (stop: target and destination nothing, behaviour unchanged, queued destinations gone)
#[test]
fn s0213_stop() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    orders::issue(&mut w, ALPHA, mv(id, testkit::cell(0, 9)));
    tick::step(&mut w);
    tick::step(&mut w);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Move));
    orders::issue(&mut w, ALPHA, Order::Stop { actor: id });
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!((o.target, o.dest), (None, None));
    assert!(o.queued_dests.is_empty());
    assert_eq!(o.beh.current, Some(Behaviour::Move));
}

// spec: S-0213 claim 9 (a scatter order sends a forced request; an infantry unit standing still accepts)
#[test]
fn s0213_scatter() {
    let mut w = ready(1);
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let before = w.obj(inf).unwrap().dest;
    orders::issue(&mut w, BETA, Order::Scatter { actor: inf });
    tick::step(&mut w);
    let o = w.obj(inf).unwrap();
    assert_ne!(o.dest, before);
    assert!(matches!(o.dest, Some(Target::Cell(_))));
}

// spec: S-0213 claim 10 (deploy: contact broken, target and destination nothing, unload queued)
#[test]
fn s0213_deploy() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    orders::issue(&mut w, ALPHA, Order::Deploy { actor: id });
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!((o.target, o.dest, o.contact), (None, None, None));
    assert!(o.beh.queued == Some(Behaviour::Unload) || o.beh.current == Some(Behaviour::Unload));
}

// spec: S-0213 claim 15 (no owner check at application: an actor whose owner changes between the issue and the application of an order still gets it; a house's orders act only on its own objects, so an order for another house's object is not taken)
#[test]
fn s0213_no_owner_check() {
    let mut w = ready(1);
    let own = of(&w, 0, Kind::Vehicle, 0)[0];
    let enemy = of(&w, 1, Kind::Vehicle, 0)[0];
    let c = testkit::cell(4, 4);
    assert!(!orders::issue(&mut w, ALPHA, mv(enemy, c)));
    assert!(orders::issue(&mut w, ALPHA, mv(own, c)));
    // The owner changes in the tick of the order.
    w.obj_mut(own).unwrap().owner = Some(BETA);
    tick::step(&mut w);
    assert_eq!(w.obj(own).unwrap().dest, Some(Target::Cell(c)));
    assert_ne!(w.obj(enemy).unwrap().dest, Some(Target::Cell(c)));
}

// spec: S-0213 claim 18 (a key command looks at every selected object in order; at most one order each)
#[test]
fn s0213_key_commands() {
    let w = ready(1);
    let carts = of(&w, 0, Kind::Vehicle, 0);
    let o = targeting::key_command(&w, ALPHA, &carts, KeyCommand::Stop);
    assert_eq!(
        o,
        carts
            .iter()
            .map(|&actor| Order::Stop { actor })
            .collect::<Vec<_>>()
    );
}

// spec: S-0213 claim 20 (a guard-spot order sets the guard spot and needs only a live actor)
#[test]
fn s0213_guard_spot() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    let spot = Target::Cell(testkit::cell(1, 1));
    w.obj_mut(id).unwrap().on_map = false;
    orders::issue(&mut w, ALPHA, Order::GuardSpot { actor: id, spot });
    tick::step(&mut w);
    assert_eq!(w.obj(id).unwrap().guard_spot, Some(spot));
}

// ---------------------------------------------------------------------------
// S-0215

// spec: S-0215 claim 1 (a newly created actor has none in all three slots)
#[test]
fn s0215_new_actor_slots() {
    let mut w = ready(1);
    let id = objects::create(&mut w, Kind::Vehicle, 0, Some(ALPHA));
    let b = &w.obj(id).unwrap().beh;
    assert_eq!((b.current, b.queued, b.suspended), (None, None, None));
}

/// A synthetic game from a changed copy of the synthetic ruleset.
fn game_with(r: &glasnost_rules::schema::Ruleset, seed: u32) -> World {
    new_game(
        Arc::new(SimRules::new(r, "claims")),
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(seed),
    )
    .unwrap()
}

// spec: S-0215 claim 14 (infantry rest in guard: rows 11 and 12, no ability, no area-guard default)
// spec: S-0215 claim 15 (armed vehicles at skill 0 and a threshold of 0 given by the data rest in area guard, row 12; unarmed ones in guard, row 7)
#[test]
fn s0215_placement_fall_back() {
    let mut w = testkit::new_game(1);
    assert_eq!(w.rules().general.iq_guard_area, 0);
    tick::step(&mut w);
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    assert_eq!(w.obj(inf).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(cart).unwrap().beh.current, Some(Behaviour::AreaGuard));
    assert_eq!(w.obj(crusher).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 36 (a) (the area-guard threshold given by no layer is 4: skill level 3 rests in guard, 4 in area guard)
// spec: S-0215 claim 15 (rows 10 and 12: below the threshold guard, else area guard)
#[test]
fn s0215_area_guard_threshold() {
    let mut r = testkit::ruleset();
    r.sim.general.iq_guard_area = None;
    let mut w = game_with(&r, 1);
    assert_eq!(w.rules().general.iq_guard_area, 4);
    // Skill level 0 of every house of our engine: the armed cart rests in guard.
    tick::step(&mut w);
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    assert_eq!(w.obj(cart).unwrap().beh.current, Some(Behaviour::Guard));
    for (skill, rest) in [(3, Behaviour::Guard), (4, Behaviour::AreaGuard)] {
        let mut w = game_with(&r, 1);
        while w.tick < 4 {
            tick::step(&mut w);
        }
        w.houses[0].skill = skill;
        let cart = of(&w, 0, Kind::Vehicle, 0)[0];
        let t = w.tick;
        w.obj_mut(cart).unwrap().beh.force(Behaviour::Move, t, true);
        process::fall_back(&mut w, cart, false);
        assert_eq!(w.obj(cart).unwrap().beh.queued, Some(rest), "skill {skill}");
    }
}

/// Makes house `h` the neutral house, not human-controlled (the rules of
/// S-0215 that read the owner look only at that).
fn make_neutral(w: &mut World, h: usize) {
    w.houses[h].human = false;
    w.houses[h].role = glasnost_sim::world::HouseRole::Neutral;
}

/// The queued behaviour after a fall-back to rest from *move* of the first
/// object of `kind` of house 1 in a game of `r`, with house 1 neutral or a
/// player house.
fn rest_of(r: &glasnost_rules::schema::Ruleset, kind: Kind, neutral: bool) -> Option<Behaviour> {
    let mut w = game_with(r, 1);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    if neutral {
        make_neutral(&mut w, 1);
    }
    let id = of(&w, 1, kind, 0)[0];
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(Behaviour::Move, t, true);
    o.beh.queued = None;
    o.dest = None;
    o.target = None;
    process::fall_back(&mut w, id, false);
    w.obj(id).unwrap().beh.queued
}

// spec: S-0215 claim 14 rows 7 to 10 (the neutral and special houses keep the rows for owners that are not human-controlled, at skill level 0: guard below the area-guard threshold; else area guard when armed, an engineer or a vehicle thief, and guard otherwise)
// spec: S-0215 claim 14 rows 11 and 12 (a player house's infantry: guard without the ability or the area-guard default)
#[test]
fn s0215_claim_14_non_player_infantry_rows() {
    let mut r = testkit::ruleset();
    // Row 7: threshold 4, skill 0.
    r.sim.general.iq_guard_area = Some(4);
    assert_eq!(rest_of(&r, Kind::Infantry, true), Some(Behaviour::Guard));
    // Threshold 0: the armed walker of the neutral house rests in area guard (row 9), a player's in guard (row 12).
    r.sim.general.iq_guard_area = Some(0);
    assert_eq!(
        rest_of(&r, Kind::Infantry, true),
        Some(Behaviour::AreaGuard)
    );
    assert_eq!(rest_of(&r, Kind::Infantry, false), Some(Behaviour::Guard));
    // Unarmed: guard (row 10); an unarmed engineer: area guard (row 9).
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    assert_eq!(rest_of(&r, Kind::Infantry, true), Some(Behaviour::Guard));
    r.sim.infantry.get_mut("WALKER").unwrap().engineer = Some(true);
    assert_eq!(
        rest_of(&r, Kind::Infantry, true),
        Some(Behaviour::AreaGuard)
    );
    assert_eq!(rest_of(&r, Kind::Infantry, false), Some(Behaviour::Guard));
}

// spec: S-0215 claim 15 row 4 (a human-controlled harvester off its resource's land type queues guard; a harvester of the neutral or special house queues harvest wherever it stands)
#[test]
fn s0215_claim_15_non_player_harvester_harvests_anywhere() {
    let mut r = testkit::ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().harvester = Some(true);
    let mut w = game_with(&r, 1);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    for (neutral, want) in [(false, Behaviour::Guard), (true, Behaviour::Harvest)] {
        let mut w = w.clone();
        if neutral {
            make_neutral(&mut w, 0);
        }
        let id = of(&w, 0, Kind::Vehicle, 0)[0];
        let t = w.tick;
        let o = w.obj_mut(id).unwrap();
        o.beh.force(Behaviour::Move, t, true);
        o.beh.queued = None;
        o.dest = None;
        process::fall_back(&mut w, id, false);
        assert_eq!(
            w.obj(id).unwrap().beh.queued,
            Some(want),
            "neutral {neutral}"
        );
    }
}

// spec: S-0215 Import mapping (cells to leptons: w(trunc(c × 256)); 4.5 cells give 1152 and 0.3 cells give 76)
// spec: S-0215 F9 (gate times: 0.1 game minutes give 90 ticks, 0.2 give 180; a wrapped product is negative; 0 at 2⁶³ or more)
// spec: S-0215 Constants (absent-key defaults: gate deploy time and close delay 0)
#[test]
fn s0215_guard_stray_and_gate_times() {
    let dec = |s: &str| Some(glasnost_rules::sim::Decimal(s.into()));
    let mut r = testkit::ruleset();
    r.sim.general.guard_mode_stray = dec("4.5");
    assert_eq!(SimRules::new(&r, "x").general.guard_stray, Some(1152));
    r.sim.general.guard_mode_stray = dec("0.3");
    assert_eq!(SimRules::new(&r, "x").general.guard_stray, Some(76));
    r.sim.general.guard_mode_stray = None;
    assert_eq!(SimRules::new(&r, "x").general.guard_stray, None);
    r.registries.insert("buildings".into(), vec!["DOOR".into()]);
    let gate = |td: Option<&str>, tc: Option<&str>| glasnost_rules::sim::BuildingType {
        gate: Some(true),
        deploy_time: td.and_then(dec),
        gate_close_delay: tc.and_then(dec),
        ..Default::default()
    };
    for (td, tc, want) in [
        (Some("0.1"), Some("0.2"), (90, 180)),
        (None, None, (0, 0)),
        (Some("2400000"), Some("20000000000000000"), (-2134967296, 0)),
    ] {
        r.sim.buildings.insert("DOOR".into(), gate(td, tc));
        let b = &SimRules::new(&r, "x").buildings[0];
        assert_eq!((b.gate_door_time, b.gate_close_delay), want);
    }
}

// spec: S-0215 claim 3 (a start before the processing is processed in the same tick)
// spec: S-0215 claim 12 (an order queues its behaviour; it becomes current at the next per-tick start)
// spec: S-0215 claim 22 (move with a destination: the standard delay)
#[test]
fn s0215_start_processed_at_once() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    orders::issue(&mut w, ALPHA, mv(id, testkit::cell(0, 9)));
    tick::step(&mut w);
    let t = w.tick;
    tick::step(&mut w);
    let b = &w.obj(id).unwrap().beh;
    assert_eq!(b.current, Some(Behaviour::Move));
    let (at, d) = b.last.unwrap();
    assert_eq!(at, t);
    // The default rate gives D_r = 14 (S-0215 F1); D = D_r + U(0, 2).
    assert!((14..=16).contains(&d), "{d}");
}

// spec: S-0215 claim 4 (a behaviour without a processing rule waits exactly 450 ticks, no draw)
#[test]
fn s0215_no_rule_450() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Sleep, t, true);
    let taken = w.rng.taken();
    process::turn(&mut w, id);
    assert_eq!(w.obj(id).unwrap().beh.last, Some((t, 450)));
    assert_eq!(w.rng.taken(), taken);
}

// spec: S-0215 claim 5 (queued move and attack move are processed as sleep, quirks.queued_move_sleeps)
#[test]
fn s0215_queued_move_sleeps() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    let t = w.tick;
    w.obj_mut(id)
        .unwrap()
        .beh
        .force(Behaviour::QueuedMove, t, true);
    process::turn(&mut w, id);
    assert_eq!(w.obj(id).unwrap().beh.last, Some((t, 450)));
}

// spec: S-0215 claim 6 (the standard delay D = D_r + U(0, 2), one draw)
// spec: S-0215 claim 24 (guard: the standard delay)
#[test]
fn s0215_standard_delay() {
    let mut w = ready(1);
    // The unarmed crusher rests in guard; a vehicle's guard attempts no idle action.
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Guard, t, true);
    let taken = w.rng.taken();
    process::turn(&mut w, id);
    let (_, d) = w.obj(id).unwrap().beh.last.unwrap();
    assert!((14..=16).contains(&d));
    assert!(w.rng.taken() > taken);
}

// spec: S-0215 claim 11 (infantry moving while not in guard, hunt or attack without a target is not ready)
#[test]
fn s0215_readiness() {
    let mut w = ready(1);
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    orders::issue(&mut w, BETA, mv(inf, testkit::cell(2, 9)));
    for _ in 0..6 {
        tick::step(&mut w);
    }
    assert_eq!(w.obj(inf).unwrap().beh.current, Some(Behaviour::Move));
    assert!(glasnost_sim::occupancy::in_progress(&w, inf));
    assert!(!process::ready(&w, inf));
    // A vehicle in guard is ready while it stands.
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    assert!(process::ready(&w, crusher));
}

// spec: S-0215 claim 13 (an infantry unit whose guard spot is on another cell takes it as destination, queues move and forgets it)
#[test]
fn s0215_common_effects() {
    let mut w = ready(1);
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let spot = Target::Cell(testkit::cell(6, 6));
    w.obj_mut(inf).unwrap().guard_spot = Some(spot);
    process::fall_back(&mut w, inf, false);
    let o = w.obj(inf).unwrap();
    assert_eq!(o.dest, Some(spot));
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert_eq!(o.guard_spot, None);
}

// spec: S-0215 claim 15 (row 3: a vehicle with a destination queues move)
#[test]
fn s0215_vehicle_with_destination() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    w.obj_mut(id).unwrap().dest = Some(Target::Cell(testkit::cell(1, 2)));
    process::fall_back(&mut w, id, false);
    assert_eq!(w.obj(id).unwrap().beh.queued, Some(Behaviour::Move));
}

// spec: S-0215 claim 22 (move without destination, not moving, nothing queued: falls back, 1 tick)
// spec: S-0215 claim 23 (attack without a target: falls back)
#[test]
fn s0215_move_and_attack_fall_back() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Move, t, true);
    process::turn(&mut w, id);
    let b = &w.obj(id).unwrap().beh;
    assert_eq!(b.last, Some((t, 1)));
    assert_eq!(b.queued, Some(Behaviour::Guard));
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    w.obj_mut(cart)
        .unwrap()
        .beh
        .force(Behaviour::Attack, t, true);
    process::turn(&mut w, cart);
    assert_eq!(w.obj(cart).unwrap().beh.queued, Some(Behaviour::AreaGuard));
}

// spec: S-0215 claim 26 (area guard: own cell as guard spot; D = D_r + U(1, 5))
#[test]
fn s0215_area_guard() {
    let mut w = ready(1);
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    let t = w.tick;
    {
        let o = w.obj_mut(cart).unwrap();
        o.guard_spot = None;
        o.beh.force(Behaviour::AreaGuard, t, true);
    }
    process::turn(&mut w, cart);
    let o = w.obj(cart).unwrap();
    assert_eq!(o.guard_spot, Some(Target::Cell(o.pos.cell())));
    let (_, d) = o.beh.last.unwrap();
    assert!((15..=19).contains(&d), "{d}");
}

// spec: S-0215 claim 17 (an armed vehicle in a zombie behaviour reaches row 9 of claim 15: nothing queued, the behaviour stays current)
// spec: S-0215 claim 17 (rows before row 9 take precedence: an unarmed vehicle queues guard by row 7, and a destination queues move)
#[test]
fn s0215_zombie_keeps_its_behaviour() {
    let mut r = testkit::ruleset();
    r.sim.behaviours.insert(
        "sleep".into(),
        glasnost_rules::sim::BehaviourProps {
            zombie: Some(true),
            ..Default::default()
        },
    );
    let rules = Arc::new(SimRules::new(&r, "zombie"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    let id = of(&w, 0, Kind::Vehicle, 0)[0];
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Sleep, t, true);
    process::fall_back(&mut w, id, false);
    assert_eq!(w.obj(id).unwrap().beh.queued, None);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Sleep));
    // The unarmed crusher in the same behaviour: row 7 comes first.
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    w.obj_mut(crusher)
        .unwrap()
        .beh
        .force(Behaviour::Sleep, t, true);
    process::fall_back(&mut w, crusher, false);
    assert_eq!(w.obj(crusher).unwrap().beh.queued, Some(Behaviour::Guard));
    assert_eq!(w.obj(crusher).unwrap().beh.current, Some(Behaviour::Sleep));
    // The armed cart with a destination: row 3 comes first.
    w.obj_mut(id).unwrap().dest = Some(Target::Cell(testkit::cell(2, 2)));
    process::fall_back(&mut w, id, false);
    assert_eq!(w.obj(id).unwrap().beh.queued, Some(Behaviour::Move));
}

// spec: S-0215 claim 20 (the recruitable property has no effect in our engine)
#[test]
fn s0215_recruitable_has_no_effect() {
    let mut r = testkit::ruleset();
    for b in ["guard", "area_guard", "move"] {
        r.sim.behaviours.insert(
            b.into(),
            glasnost_rules::sim::BehaviourProps {
                recruitable: Some(false),
                ..Default::default()
            },
        );
    }
    let a = Arc::new(SimRules::new(&testkit::ruleset(), "x"));
    let b = Arc::new(SimRules::new(&r, "x"));
    let mut wa = new_game(a, &testkit::map(), testkit::catalogue(), &testkit::setup(4)).unwrap();
    let mut wb = new_game(b, &testkit::map(), testkit::catalogue(), &testkit::setup(4)).unwrap();
    for _ in 0..60 {
        tick::step(&mut wa);
        tick::step(&mut wb);
    }
    assert_eq!(
        glasnost_sim::save::state_hash(&wa),
        glasnost_sim::save::state_hash(&wb)
    );
}

// spec: S-0215 claim 21 (a behaviour that gives no anti-air rate, or 0, uses its rate)
#[test]
fn s0215_anti_air_rate_default() {
    let r = testkit::rules();
    for b in Behaviour::ALL {
        let p = r.behaviour(b);
        assert_eq!(p.aa_rate_delay, p.rate_delay);
    }
}

// spec: S-0213 claim 14 (house orders apply at the tick boundary in the order of claim 3)
// spec: S-0213 table T1 (cheer: all of the issuer's infantry cheer, S-0107 claim 6 for a type with a cheer sequence)
#[test]
fn s0213_cheer() {
    let mut r = testkit::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().sequences.insert(
        "cheer".into(),
        glasnost_rules::sim::SequenceEntry {
            start: 0,
            count: 8,
            multiplier: 0,
            direction: None,
        },
    );
    let rules = Arc::new(SimRules::new(&r, "cheer"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    assert!(orders::issue(&mut w, BETA, Order::Cheer));
    tick::step(&mut w);
    for id in of(&w, 1, Kind::Infantry, 0) {
        let s = w.obj(id).unwrap().infantry.as_ref().unwrap().seq;
        assert_eq!(s, Some(glasnost_sim::infantry::Seq::Cheer));
    }
    // Alpha's infantry do not cheer.
    for id in of(&w, 0, Kind::Infantry, 0) {
        let s = w.obj(id).unwrap().infantry.as_ref().unwrap().seq;
        assert_ne!(s, Some(glasnost_sim::infantry::Seq::Cheer));
    }
}

// spec: S-0213 claim 19 (a cheer is accepted only after more than R ticks since the last; −100 at program start; the memory spans the session)
#[test]
fn s0213_cheer_rate_limit() {
    let mut r = testkit::ruleset();
    r.sim.general.maximum_cheer_rate = Some(150);
    let rules = Arc::new(SimRules::new(&r, "cheer"));
    let mut w = new_game(
        rules.clone(),
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    let mut memory = targeting::CheerMemory::default();
    while w.tick < 50 {
        tick::step(&mut w);
    }
    // 50 > −100 + 150 is false: refused.
    assert_eq!(memory.cheer(&w), None);
    while w.tick < 51 {
        tick::step(&mut w);
    }
    assert_eq!(memory.cheer(&w), Some(Order::Cheer));
    assert_eq!(memory.last, 51);
    // A new game keeps the memory (quirks.cheer_memory_spans_session).
    let w2 = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    memory.game_start(&w2);
    assert_eq!(memory.last, 51);
}

// spec: S-0215 claim 30 (unload for a deployer: deploy when undeployed, undeploy when deployed with a negative delay; guard forced, destination cleared, 450 ticks)
// spec: S-0213 claim 10 (a deploy order queues unload)
#[test]
fn s0215_infantry_unload_toggles_deploy() {
    let mut r = testkit::ruleset();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.deployer = Some(true);
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 4),
        ("ready", 1),
        ("guard", 1),
    ] {
        walker.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let rules = Arc::new(SimRules::new(&r, "deployer"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let seq = |w: &World| w.obj(inf).unwrap().infantry.as_ref().unwrap().seq;
    orders::issue(&mut w, BETA, Order::Deploy { actor: inf });
    tick::step(&mut w);
    tick::step(&mut w);
    let o = w.obj(inf).unwrap();
    assert_eq!(seq(&w), Some(glasnost_sim::infantry::Seq::Deploy));
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.last.map(|l| l.1), Some(450));
    for _ in 0..40 {
        tick::step(&mut w);
    }
    assert!(
        seq(&w).is_some_and(glasnost_sim::infantry::Seq::is_deploy),
        "{:?}",
        seq(&w)
    );
    // A second deploy order undeploys the unit (its undeploy delay is absent, so negative).
    orders::issue(&mut w, BETA, Order::Deploy { actor: inf });
    tick::step(&mut w);
    tick::step(&mut w);
    assert_eq!(seq(&w), Some(glasnost_sim::infantry::Seq::Undeploy));
}

// spec: S-0215 claim 25 (sticky is processed as guard, and an actor in sticky is never ready to change)
#[test]
fn s0215_sticky() {
    let mut w = ready(1);
    let id = of(&w, 0, Kind::Vehicle, 1)[0];
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Sticky, t, true);
    assert!(!process::ready(&w, id));
    orders::issue(&mut w, ALPHA, mv(id, testkit::cell(0, 9)));
    for _ in 0..20 {
        tick::step(&mut w);
    }
    let b = &w.obj(id).unwrap().beh;
    // The ordered move stays queued, and sticky takes guard's standard delay.
    assert_eq!(b.current, Some(Behaviour::Sticky));
    assert_eq!(b.queued, Some(Behaviour::Move));
    assert!(b.last.is_some_and(|(_, d)| (14..=16).contains(&d)));
}

// spec: S-0215 claim 31 (a deployed deployer with an undeploy delay of 0 or more undeploys in guard; the delay is that sequence's frame count)
#[test]
fn s0215_deployer_in_guard_undeploys() {
    let mut r = testkit::ruleset();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.deployer = Some(true);
    walker.undeploy_delay = Some(30);
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 6),
        ("ready", 1),
        ("guard", 1),
    ] {
        walker.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let rules = Arc::new(SimRules::new(&r, "deployer2"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    orders::issue(&mut w, BETA, Order::Deploy { actor: inf });
    tick::step(&mut w);
    tick::step(&mut w);
    // Claim 30: the undeploy delay (30) when it is 0 or more.
    assert_eq!(w.obj(inf).unwrap().beh.last.map(|l| l.1), Some(30));
    let seq = |w: &World| w.obj(inf).unwrap().infantry.as_ref().unwrap().seq;
    for _ in 0..40 {
        tick::step(&mut w);
        if seq(&w) == Some(glasnost_sim::infantry::Seq::Undeploy) {
            break;
        }
    }
    assert_eq!(seq(&w), Some(glasnost_sim::infantry::Seq::Undeploy));
    let b = &w.obj(inf).unwrap().beh;
    assert_eq!(b.current, Some(Behaviour::Guard));
    assert_eq!(b.last.map(|l| l.1), Some(6));
}

/// The tick on which the walker of house 1 starts its deploy sequence by
/// S-0215 claim 31's auto-deploy row, within 300 ticks of being put in
/// guard, and that processing's delay; the auto-deploy delay list is
/// `list`, house 1 is neutral when `neutral`; with `test_index` it stays a
/// player house marked as a test-only computer-controlled player house with
/// that difficulty index.
fn auto_deploy_tick(list: Option<Vec<i32>>, neutral: bool) -> Option<(i32, i32, i32)> {
    auto_deploy_tick_as(list, neutral, None)
}

fn auto_deploy_tick_as(
    list: Option<Vec<i32>>,
    neutral: bool,
    test_index: Option<i32>,
) -> Option<(i32, i32, i32)> {
    let mut r = testkit::ruleset();
    r.sim.general.ai_auto_deploy_frame_delay = list;
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.deployer = Some(true);
    walker.deploy_fire = Some(true);
    walker.undeploy_delay = Some(-1);
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 6),
        ("ready", 1),
    ] {
        walker.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let mut w = game_with(&r, 1);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    if neutral {
        make_neutral(&mut w, 1);
    }
    if let Some(i) = test_index {
        w.houses[1].human = false;
        w.houses[1].econ.setup.difficulty = i;
    }
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let s = w.tick;
    let o = w.obj_mut(inf).unwrap();
    o.beh.force(Behaviour::Guard, s, true);
    o.guard_spot = None;
    o.dest = None;
    for _ in 0..300 {
        tick::step(&mut w);
        let o = w.obj(inf).unwrap();
        if o.infantry.as_ref().unwrap().seq == Some(glasnost_sim::infantry::Seq::Deploy) {
            let (at, d) = o.beh.last.unwrap();
            return Some((s, at, d));
        }
    }
    None
}

// spec: S-0215 claim 31 (the auto-deploy row: an undeployed deploy-to-fire unit of the neutral or special house, in guard with no destination and no guard spot, starts its deploy sequence at the first processing with t > t_s + A, A the list's entry for difficulty index 1; the delay is the sequence's frame count)
// spec: S-0215 claim 31 (our engine: no player house auto-deploys; a list without a second entry: the row does not apply)
#[test]
fn s0215_claim_31_auto_deploy_of_non_player_houses() {
    let (s, at, d) = auto_deploy_tick(Some(vec![300, 40, 5]), true).expect("deploys");
    assert!(at > s + 40, "deployed at {at}, start {s}");
    let dr = testkit::rules().behaviour(Behaviour::Guard).rate_delay;
    assert!(at <= s + 40 + dr + 3, "deployed at {at}, start {s}");
    assert_eq!(d, 4);
    assert_eq!(auto_deploy_tick(Some(vec![300, 40, 5]), false), None);
    assert_eq!(auto_deploy_tick(Some(vec![40]), true), None);
    assert_eq!(auto_deploy_tick(None, true), None);
}

// spec: S-0215 claim 31 and S-0309 Terms (Test-only exception: a marked test-only computer-controlled player house auto-deploys, with A the list's entry for its own difficulty index)
#[test]
fn s0215_claim_31_auto_deploy_of_a_test_house_by_its_index() {
    let (s, at, _) =
        auto_deploy_tick_as(Some(vec![40, 250, 250]), false, Some(0)).expect("deploys");
    assert!(at > s + 40, "deployed at {at}, start {s}");
    let dr = testkit::rules().behaviour(Behaviour::Guard).rate_delay;
    assert!(
        at <= s + 40 + dr + 3,
        "index 0: deployed at {at}, start {s}"
    );
    // Index 0 with a one-entry list applies; index 2 with a two-entry list does not.
    assert!(auto_deploy_tick_as(Some(vec![40]), false, Some(0)).is_some());
    assert_eq!(
        auto_deploy_tick_as(Some(vec![40, 40]), false, Some(2)),
        None
    );
}

// ---------------------------------------------------------------------------
// S-0215 claim 35: gates

/// The cell of the test gate, away from the scenario's units.
fn gate_cell() -> CellXY {
    testkit::cell(12, 14)
}

/// The synthetic game with a one-cell gate GATE of house Alpha (door time
/// 0.1, close delay 0.2 game minutes) and a plain building HUT, and the
/// gate's ID, with orders open.
fn gate_world(snaps: bool) -> (World, ObjectId) {
    use glasnost_map::objects::{Placed, Structure};
    use glasnost_rules::sim::{BuildingType, Decimal};
    let mut r = testkit::ruleset();
    r.globals.quirks.gate_reverse_snaps = snaps;
    r.registries
        .insert("buildings".into(), vec!["GATE".into(), "HUT".into()]);
    r.sim.buildings.insert(
        "GATE".into(),
        BuildingType {
            strength: Some(500),
            gate: Some(true),
            deploy_time: Some(Decimal("0.1".into())),
            gate_close_delay: Some(Decimal("0.2".into())),
            ..Default::default()
        },
    );
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            ..Default::default()
        },
    );
    let mut map = testkit::map();
    for (key, ty, c) in [
        ("900", "GATE", gate_cell()),
        ("901", "HUT", testkit::cell(-12, 14)),
    ] {
        map.objects.structures.push(Structure {
            placed: Placed {
                key: key.as_bytes().to_vec(),
                owner: b"Alpha".to_vec(),
                type_name: ty.as_bytes().to_vec(),
                health: 256,
                cell: c,
                facing: 0,
                raw: Vec::new(),
            },
        });
    }
    let mut w = new_game(
        Arc::new(SimRules::new(&r, "gates")),
        &map,
        testkit::catalogue(),
        &testkit::setup(1),
    )
    .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let gate = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building && o.pos.cell() == gate_cell())
        .unwrap()
        .id;
    (w, gate)
}

fn door(w: &World, gate: ObjectId) -> Door {
    w.obj(gate).unwrap().beh.gate.door
}

// spec: S-0215 F9 (the test gate's door time 0.1 and close delay 0.2 game minutes give 90 and 180 ticks)
// spec: S-0215 claim 35 (a) (a gate starts with its door at rest closed: passage closed)
// spec: S-0215 claim 35 (b) (the closed-friendly-gate outcome drops the queue and makes open current at once, a behaviour start)
#[test]
fn s0215_gate_starts_closed_and_opens_on_demand() {
    let (mut w, gate) = gate_world(true);
    let b = &w.rules().buildings[w.obj(gate).unwrap().type_index as usize];
    assert_eq!((b.gate_door_time, b.gate_close_delay), (90, 180));
    assert_eq!(door(&w, gate), Door::Closed);
    assert!(!process::gate_open(&w, gate));
    w.obj_mut(gate).unwrap().beh.queued = Some(Behaviour::Guard);
    w.obj_mut(gate).unwrap().beh.progress = 3;
    process::gate_met(&mut w, gate);
    let b = &w.obj(gate).unwrap().beh;
    assert_eq!((b.current, b.queued), (Some(Behaviour::Open), None));
    assert!(b.expired && b.progress == 0);
    // Open current and the door opening: a second meeting changes nothing.
    let t = w.tick;
    tick::step(&mut w);
    assert!(matches!(door(&w, gate), Door::Opening { since, ticks: 90 } if since == t));
    let before = w.obj(gate).unwrap().beh.clone();
    process::gate_met(&mut w, gate);
    assert_eq!(w.obj(gate).unwrap().beh, before);
}

// spec: S-0215 claim 35 (passage state open only with open current and the door at rest open)
// spec: S-0215 claim 35 (c) (the stage table: opening, waiting with Std, closing-start, closing, finished with guard queued)
// spec: S-0215 claim 35 (d) (a moving door reaches rest once Td ticks have passed since the motion began)
// spec: S-0215 claim 35 (e) (closes no earlier than Tc ticks after it began to open)
#[test]
fn s0215_gate_open_and_close_cycle() {
    let (mut w, gate) = gate_world(true);
    process::gate_met(&mut w, gate);
    let s = w.tick;
    let mut opened = None;
    let mut closing = None;
    let mut closed = None;
    for _ in 0..600 {
        tick::step(&mut w);
        let t = w.tick - 1;
        if opened.is_none() && process::gate_open(&w, gate) {
            opened = Some(t);
        }
        if closing.is_none() && matches!(door(&w, gate), Door::Closing { .. }) {
            closing = Some(t);
            assert!(!process::gate_open(&w, gate));
        }
        if closing.is_some() && closed.is_none() && door(&w, gate) == Door::Closed {
            closed = Some(t);
        }
    }
    assert_eq!(opened, Some(s + 90));
    let c = closing.unwrap();
    // Waiting processings come one Std (D_r 14 of the default rate, + U(0, 2)) apart.
    assert!((s + 180..=s + 180 + 2 * 16).contains(&c), "{c}");
    assert_eq!(closed, Some(c + 90));
    let b = &w.obj(gate).unwrap().beh;
    // The finished stage queued guard; the zero-delay idle body range of the test gate sets the ready mark on every turn, so guard is current by now, its start resetting the progress count (S-0215 claims 3 and 11).
    assert_eq!(
        (b.current, b.queued, b.progress),
        (Some(Behaviour::Guard), None, 0)
    );
    assert!(!process::gate_open(&w, gate));
}

// spec: S-0215 claim 35 (d) and scenario B28 case g (the door reaches rest at the gate's turn before its processing: with Td = 90 the opening processing of tick t + 90 finds it at rest open and chooses 0, and the first Std processing comes in tick t + 91; request Q-0030)
#[test]
fn s0215_gate_door_rests_before_the_processing() {
    let (mut w, gate) = gate_world(true);
    process::gate_met(&mut w, gate);
    let t = w.tick;
    let mut delays = Vec::new();
    for _ in 0..92 {
        tick::step(&mut w);
        let b = &w.obj(gate).unwrap().beh;
        delays.push((w.tick - 1, b.last, b.progress, door(&w, gate)));
    }
    // Tick t: the door starts opening, delay 0.
    assert!(matches!(delays[0].3, Door::Opening { since, ticks: 90 } if since == t));
    // Ticks t + 1 to t + 89: stage opening, the door still moving, delay 0 each.
    for (tick, last, progress, d) in &delays[1..90] {
        assert_eq!(*last, Some((*tick, 0)), "tick {tick}");
        assert_eq!(*progress, 1);
        assert!(matches!(d, Door::Opening { .. }));
    }
    // Tick t + 90: the door is at rest before the processing, which moves to waiting with 0.
    let (tick, last, progress, d) = delays[90];
    assert_eq!(tick, t + 90);
    assert_eq!(d, Door::Open);
    assert_eq!((last, progress), (Some((t + 90, 0)), 2));
    // Tick t + 91: the first waiting processing, with its Std (D_r of the rate of open, + U(0, 2)).
    let (tick, last, _, _) = delays[91];
    let (at, delay) = last.unwrap();
    assert_eq!(at, tick);
    let dr = w.rules().behaviour(Behaviour::Open).rate_delay;
    assert!((dr..=dr + 2).contains(&delay), "{delay}");
}

// spec: S-0215 claim 35 (c) (waiting: an object in the ground layer of a foundation cell restarts the close countdown)
#[test]
fn s0215_gate_stays_open_while_occupied() {
    let (mut w, gate) = gate_world(true);
    let blocker = glasnost_sim::objects::create(&mut w, Kind::Infantry, 0, Some(ALPHA));
    w.cells.enter(
        gate_cell(),
        glasnost_sim::world::Layer::Ground,
        blocker,
        false,
    );
    process::gate_met(&mut w, gate);
    for _ in 0..500 {
        tick::step(&mut w);
    }
    assert!(process::gate_open(&w, gate));
    w.cells
        .leave(gate_cell(), glasnost_sim::world::Layer::Ground, blocker);
    let left = w.tick;
    let n = run_until_closing(&mut w, gate, 400);
    assert!((180..=180 + 2 * 16).contains(&(w.tick - left)), "{n}");
}

fn run_until_closing(w: &mut World, gate: ObjectId, max: i32) -> i32 {
    for i in 0..max {
        if matches!(door(w, gate), Door::Closing { .. }) {
            return i;
        }
        tick::step(w);
    }
    max
}

// spec: S-0215 claim 35 (quirks.gate_reverse_snaps: a door reversed while closing is at rest open at the gate's next update; off: it takes the ticks the closing had run)
// spec: S-0215 claim 35 (e) (a unit that reaches a closing gate makes it reverse by (b))
#[test]
fn s0215_gate_reverse_snaps() {
    for snaps in [true, false] {
        let (mut w, gate) = gate_world(snaps);
        process::gate_met(&mut w, gate);
        run_until_closing(&mut w, gate, 600);
        for _ in 0..30 {
            tick::step(&mut w);
        }
        let Door::Closing { since, ticks: td } = door(&w, gate) else {
            panic!("closing expected");
        };
        process::gate_met(&mut w, gate);
        let reversed = w.tick;
        let ran = reversed - since;
        if !snaps {
            // spec: S-0215 claim 35 d2 (off: the reopening uses the opening expression over Td with progress Td − a + e, so it is at rest after a ticks; Q-0685 answered)
            tick::step(&mut w);
            let Door::Opening { since: s2, ticks } = door(&w, gate) else {
                panic!("opening expected");
            };
            assert_eq!(ticks, td);
            assert_eq!(s2 + td - (w.tick - 1), w.tick - 1 - since);
        }
        let n = run_until(&mut w, 200, |w| process::gate_open(w, gate));
        // The reversal happens in the gate's processing of the next tick.
        if snaps {
            assert_eq!(n, 2);
        } else {
            assert_eq!(n + 1, ran + 1);
        }
        assert!(ran >= 30);
    }
}

fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> i32 {
    for i in 0..max {
        if done(w) {
            return i;
        }
        tick::step(w);
    }
    max
}

// spec: S-0215 claim 35 (c) and Constants (open for a building that is not a gate: queues guard, 1 tick, no draw)
// spec: S-0215 claim 4 (open for a mobile actor: no processing rule, 450 ticks)
#[test]
fn s0215_open_without_a_gate() {
    let (mut w, _) = gate_world(true);
    let hut = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building && o.pos.cell() == testkit::cell(-12, 14))
        .unwrap()
        .id;
    let t = w.tick;
    w.obj_mut(hut).unwrap().beh.force(Behaviour::Open, t, true);
    let taken = w.rng.taken();
    process::turn(&mut w, hut);
    let b = &w.obj(hut).unwrap().beh;
    // Guard queued; the hut's zero-delay idle body range has set the ready mark, so the second per-tick start may already have made it current (S-0215 claims 1 and 11).
    assert_eq!((b.last, b.seen()), (Some((t, 1)), Some(Behaviour::Guard)));
    assert_eq!(w.rng.taken(), taken);
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    w.obj_mut(cart).unwrap().beh.force(Behaviour::Open, t, true);
    process::turn(&mut w, cart);
    assert_eq!(w.obj(cart).unwrap().beh.last, Some((t, 450)));
}

// spec: S-0214 claim 3 (nothing over an object that is already selected: the selection stays and no order is made)
#[test]
fn s0214_nothing_over_a_selected_object() {
    let (w, gate) = gate_world(true);
    let hut = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Building && o.pos.cell() == testkit::cell(-12, 14))
        .unwrap()
        .id;
    let sel = [hut, gate];
    let on = targeting::ClickOn::Object(gate);
    let mods = targeting::Modifiers::default();
    assert_eq!(
        targeting::cursor(&w, ALPHA, &sel, on, mods),
        targeting::Action::Nothing
    );
    assert_eq!(
        targeting::click(&w, ALPHA, &sel, on, mods),
        targeting::ClickResult::default()
    );
}

/// The suspended slot after a move order is applied to an actor that holds
/// one (S-0215 claims 8 to 10; the order itself forgets the put-aside pair,
/// claim 12).
fn suspended_after_move(stale: bool) -> Option<Behaviour> {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.stale_suspended_revert = stale;
    let mut w = testkit::game_with(&r, 32, 3);
    let id = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(32, 0, 0),
        0,
    );
    w.tick = 10;
    let o = w.obj_mut(id).unwrap();
    o.beh.suspended = Some(Behaviour::Guard);
    assert!(orders::issue(&mut w, ALPHA, mv(id, testkit::mid(32, 4, 0))));
    for _ in 0..30 {
        testkit::run(&mut w, 1);
        if w.obj(id).unwrap().beh.current == Some(Behaviour::Move) {
            return w.obj(id).unwrap().beh.suspended;
        }
    }
    panic!("the move never started");
}

// spec: S-0215 claims 8 to 10 (start-next leaves the suspended behaviour in place)
// spec: S-0215 Edge cases (quirks.stale_suspended_revert off: an effective start-next empties it)
#[test]
fn s0215_stale_suspended_revert() {
    assert_eq!(suspended_after_move(true), Some(Behaviour::Guard));
    assert_eq!(suspended_after_move(false), None);
}

// ---------------------------------------------------------------------------
// S-0201 claims 8 and 17: the speed order

/// A speed order is carried out at the end of the tick it is issued in, by
/// our order path, and sets the game-speed index whoever sent it; the last
/// one carried out holds, an index outside 0 to 6 is ignored, and speed-
/// normalised delays started from then on use the new index (S-0201 claims
/// 8 and 17; S-0107 F3).
#[test]
fn s0201_speed_order_sets_the_index_at_the_end_of_the_tick() {
    let mut w = ready(7);
    let before = w.game_speed;
    assert_ne!(before, 1, "the fixture starts at another index");
    // spec: S-0201 claim 17 (carried out in one tick on every machine; an order of either player)
    assert!(orders::issue(&mut w, BETA, Order::GameSpeed { speed: 1 }));
    assert_eq!(w.game_speed, before, "not before the order is carried out");
    tick::step(&mut w);
    assert_eq!(w.game_speed, 1);
    // spec: S-0107 F3 and S-0201 claim 17 (delays started from then on use the new index)
    assert_ne!(
        glasnost_sim::infantry::normalised(4, w.game_speed),
        glasnost_sim::infantry::normalised(4, before)
    );
    // spec: S-0201 claim 17 (the last change carried out holds)
    assert!(orders::issue(&mut w, ALPHA, Order::GameSpeed { speed: 5 }));
    assert!(orders::issue(&mut w, BETA, Order::GameSpeed { speed: 2 }));
    tick::step(&mut w);
    assert_eq!(w.game_speed, 2);
    // spec: S-0201 claim 8 (indices 0 to 6 only; an index outside, which no menu sends, is ignored)
    orders::issue(&mut w, ALPHA, Order::GameSpeed { speed: 9 });
    tick::step(&mut w);
    assert_eq!(w.game_speed, 2);
    // The index is hashed state and survives a save/load round trip.
    let back = glasnost_sim::save::load(
        &glasnost_sim::save::save(&w),
        testkit::rules(),
        testkit::catalogue(),
    )
    .expect("loads");
    assert_eq!(back.game_speed, 2);
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

/// The order block of the first ticks refuses input but not a speed change
/// (S-0213 claim 1; S-0201 claim 8: the change takes effect from the next
/// cycle; claim 14's block is stated for a single-player game).
#[test]
fn s0201_speed_order_is_taken_during_the_order_block() {
    let mut w = testkit::new_game(7);
    assert!(orders::blocked(&w));
    let before = w.game_speed;
    let want = if before == 0 { 3 } else { 0 };
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::GameSpeed { speed: want }
    ));
    tick::step(&mut w);
    assert_eq!(w.game_speed, want);
}
