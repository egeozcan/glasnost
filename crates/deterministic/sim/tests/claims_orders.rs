//! Claim tests for orders (S-0213) and behaviours (S-0215), on the
//! synthetic scenario.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_sim::load::new_game;
use glasnost_sim::objects;
use glasnost_sim::orders::{self, Order, PENDING_LIMIT};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::targeting::{self, KeyCommand};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

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
    // Another house has its own list.
    assert!(orders::issue(&mut w, BETA, Order::Stop { actor: id }));
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
        (Some(Behaviour::Move), Some(Target::Cell(c)))
    );
    let o = w.obj(crusher).unwrap();
    assert_eq!(
        (o.beh.queued, o.attack_move),
        (Some(Behaviour::Attack), None)
    );
    orders::issue(&mut w, ALPHA, am(cart, Some(Target::Object(enemy)), None));
    tick::step(&mut w);
    let o = w.obj(cart).unwrap();
    assert_eq!(o.attack_move, Some(Target::Object(enemy)));
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

// spec: S-0213 claim 15 (no owner check: an order in another house's name still reaches the actor)
#[test]
fn s0213_no_owner_check() {
    let mut w = ready(1);
    let enemy = of(&w, 1, Kind::Vehicle, 0)[0];
    let c = testkit::cell(4, 4);
    orders::issue(&mut w, ALPHA, mv(enemy, c));
    tick::step(&mut w);
    assert_eq!(w.obj(enemy).unwrap().dest, Some(Target::Cell(c)));
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

// spec: S-0215 claim 14 (infantry rest in guard: rows 11 and 12, no ability, no area-guard default)
// spec: S-0215 claim 15 (armed vehicles at skill 0 and threshold 0 rest in area guard, row 12; unarmed ones in guard, row 7)
#[test]
fn s0215_placement_fall_back() {
    let mut w = testkit::new_game(1);
    tick::step(&mut w);
    let inf = of(&w, 1, Kind::Infantry, 0)[0];
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    assert_eq!(w.obj(inf).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(cart).unwrap().beh.current, Some(Behaviour::AreaGuard));
    assert_eq!(w.obj(crusher).unwrap().beh.current, Some(Behaviour::Guard));
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
    w.obj_mut(id).unwrap().beh.force(Behaviour::Sleep, t);
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
    w.obj_mut(id).unwrap().beh.force(Behaviour::QueuedMove, t);
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
    w.obj_mut(id).unwrap().beh.force(Behaviour::Guard, t);
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
    w.obj_mut(id).unwrap().beh.force(Behaviour::Move, t);
    process::turn(&mut w, id);
    let b = &w.obj(id).unwrap().beh;
    assert_eq!(b.last, Some((t, 1)));
    assert_eq!(b.queued, Some(Behaviour::Guard));
    let cart = of(&w, 0, Kind::Vehicle, 0)[0];
    w.obj_mut(cart).unwrap().beh.force(Behaviour::Attack, t);
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
        o.beh.force(Behaviour::AreaGuard, t);
    }
    process::turn(&mut w, cart);
    let o = w.obj(cart).unwrap();
    assert_eq!(o.guard_spot, Some(Target::Cell(o.pos.cell())));
    let (_, d) = o.beh.last.unwrap();
    assert!((15..=19).contains(&d), "{d}");
}

// spec: S-0215 claim 17 (a zombie or paralysed behaviour is never left by falling back to rest)
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
    w.obj_mut(id).unwrap().beh.force(Behaviour::Sleep, t);
    process::fall_back(&mut w, id, false);
    assert_eq!(w.obj(id).unwrap().beh.queued, None);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Sleep));
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
