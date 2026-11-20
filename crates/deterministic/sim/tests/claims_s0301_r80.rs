//! S-0301 r80 claim 9, *Targets being erased within rank 4* (Q-3314, issue
//! #458), and scenario F24: a target that a temporal beam is erasing
//! answers *illegal* for every weapon that is not temporal, and the holders
//! and searchers drop or skip it by the rules that act on *illegal*.
//! Synthetic rules (`temporal_common`, plus a landed aircraft PLANE, the
//! ordinary firers SHOOTER, GUNMAN and SLOW, and strong targets); every name
//! and number is invented.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::check::{Answer, check, illegal_at_any_rank};
use glasnost_sim::combat::weapon::would_use;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::temporal;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use temporal_common::*;

/// S-0309 claim 29 (Phase: the check ticks are those whose number is a
/// multiple of 16).
const CHECK: i32 = 16;

/// The targets of claim 9's answer: an infantry unit, a vehicle, an
/// aircraft landed on flat ground and a building.
const TARGETS: [(Kind, &str); 4] = [
    (Kind::Infantry, "BYSTANDER"),
    (Kind::Vehicle, "VICT"),
    (Kind::Aircraft, "PLANE"),
    (Kind::Building, "TOWER"),
];

/// The ruleset: every target strong enough that its erasure lasts well past
/// the runs here, PLANE (a landed aircraft), SHOOTER (a vehicle with the
/// Gun), GUNMAN (an infantry unit with the Rifle) and SLOW (a vehicle whose
/// weapon has a rate of fire of 300 ticks).
fn rules() -> Ruleset {
    let mut r = temporal_common::ruleset();
    r.sim.aircraft.insert(
        "PLANE".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(100_000),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        },
    );
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
    r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000);
    r.sim.buildings.get_mut("TOWER").unwrap().strength = Some(100_000);
    vehicle(&mut r, "SHOOTER", |t| t.primary = Some("Gun".into()));
    infantry(&mut r, "GUNMAN", |t| t.primary = Some("Rifle".into()));
    let mut slow = r.sim.weapons["Gun"].clone();
    slow.rof = Some(300);
    r.sim.weapons.insert("SlowGun".into(), slow);
    vehicle(&mut r, "SLOW", |t| t.primary = Some("SlowGun".into()));
    r
}

/// A game under [`rules`], run to the tick from which orders are open.
fn new_game() -> World {
    let mut w = game(&rules());
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    w
}

/// Puts `id` in *sleep*, so that it acts only as the test makes it.
fn sleep(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id)
        .unwrap()
        .beh
        .force(Behaviour::Sleep, tick, stale);
}

/// The firing check without the range test, for the weapon `firer` would
/// use against `t`.
fn ask(w: &World, firer: ObjectId, t: Target) -> Answer {
    let (slot, _) = would_use(w, firer, Some(t));
    check(w, firer, Some(t), slot, false)
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

/// Orders `actor` of house 0 to attack `t` (no force-fire).
fn order_attack(w: &mut World, actor: ObjectId, t: ObjectId) -> bool {
    orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(t)),
            dest: None,
        },
    )
}

/// Fires `e`'s temporal weapon at `v` and runs ticks until the beam locks
/// (S-0514 claims 1 and 4); returns *L*, the number of the tick at whose
/// end the beam is first attached. The eraser then keeps its target, as in
/// play.
fn lock(w: &mut World, e: ObjectId, v: ObjectId) -> i32 {
    assert!(glasnost_sim::combat::fire::shot(w, e, 0, Target::Object(v)));
    for _ in 0..60 {
        let k = w.tick;
        glasnost_sim::tick::step(w);
        if temporal::erased(w, v) {
            let tick = w.tick;
            let stale = w.rules().quirks.stale_suspended_revert;
            let o = w.obj_mut(e).unwrap();
            o.target = Some(Target::Object(v));
            o.beh.force(Behaviour::Attack, tick, stale);
            return k;
        }
    }
    panic!("the beam did not lock");
}

/// The number of shots `id` has made so far, read from its rearm countdown
/// (S-0301 claim 1: each shot restarts it).
fn shot_mark(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(i32::MIN, |o| o.combat.rearm_until)
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4, Q-3314: a target being erased answers illegal for every weapon that is not a temporal weapon, for an infantry unit, vehicle, aircraft or building target, from the lock that starts its erasure); S-0514 claims 1 and 4 (the lock); S-0214 claim 6 (e) (the illegal-only test sees it too)
#[test]
fn claim_9_every_kind_of_target_being_erased_is_illegal_for_a_weapon_that_is_not_temporal() {
    for (kind, name) in TARGETS {
        let mut w = new_game();
        let x = spawn(&mut w, kind, name, 1, at(0, 0));
        let s = attacker(&mut w, "GUNMAN", 0, at(-2, 0));
        let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
        let t = Target::Object(x);
        let (slot, _) = would_use(&w, s, Some(t));
        assert_ne!(
            ask(&w, s, t),
            Answer::Illegal,
            "{name}: void unless the firer may fire at it before the lock"
        );
        assert!(!illegal_at_any_rank(&w, s, Some(t), slot), "{name}");
        lock(&mut w, e, x);
        assert!(temporal::erased(&w, x), "{name}: being erased");
        assert_eq!(ask(&w, s, t), Answer::Illegal, "{name}: illegal");
        assert!(
            illegal_at_any_rank(&w, s, Some(t), slot),
            "{name}: and for the illegal-only test"
        );
    }
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4: a cell target gets no such answer)
#[test]
fn claim_9_a_cell_target_gets_no_such_answer() {
    let mut w = new_game();
    let x = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let s = attacker(&mut w, "GUNMAN", 0, at(-2, 0));
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let cell = Target::Cell(w.obj(x).unwrap().pos.cell());
    let before = ask(&w, s, cell);
    assert_ne!(
        before,
        Answer::Illegal,
        "void unless the cell may be fired at"
    );
    lock(&mut w, e, x);
    assert!(temporal::erased(&w, x));
    assert_eq!(ask(&w, s, cell), before, "the cell's answer is unchanged");
    assert_eq!(ask(&w, s, Target::Object(x)), Answer::Illegal);
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4: the answer does not refuse a temporal weapon; a temporal firer whose beam is not on the target gets the answers of the other reasons and its shot joins the erasure, S-0514 claims 1 to 4; one whose beam is already on it gets the held-temporal-target rearming, S-0514 claim 19); scenario F24 part e
#[test]
fn claim_9_temporal_firers_are_not_refused_and_f24_e() {
    let mut w = new_game();
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let e2 = attacker(&mut w, "ERASER", 0, at(-6, 6));
    let l = lock(&mut w, e, v);
    run_to(&mut w, l + 21);
    assert!(temporal::erased(&w, v));
    let t = Target::Object(v);
    assert_eq!(ask(&w, e, t), Answer::Rearming, "the holder: rearming");
    assert_ne!(ask(&w, e2, t), Answer::Illegal, "another temporal firer");
    assert!(order_attack(&mut w, e2, v), "E2's order");
    let order = w.tick;
    let mut joined = false;
    while w.tick <= order + 100 {
        glasnost_sim::testkit::run(&mut w, 1);
        if temporal::holds(&w, e2, v) {
            joined = true;
            break;
        }
        assert_eq!(
            target(&w, e2),
            Some(t),
            "E2 holds V until its beam is on it (tick {})",
            w.tick - 1
        );
    }
    assert!(joined, "E2's beam is attached by the 100th tick (F24 e)");
    assert!(temporal::holds(&w, e, v), "E's beam stays attached");
    assert_eq!(ask(&w, e2, t), Answer::Rearming, "E2 now holds it too");
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4: the answer outranks every reason of ranks 5 and 7 to 16, so a firer whose rearm countdown is running, or that is out of ammunition, gets illegal for such a target)
#[test]
fn claim_9_it_outranks_the_rearm_countdown_and_ammunition() {
    let mut w = new_game();
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let s = spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, at(-3, 0));
    sleep(&mut w, s);
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let t = Target::Object(v);
    let mut countdown = w.clone();
    let tick = countdown.tick;
    countdown.obj_mut(s).unwrap().combat.rearm_until = tick + 500;
    assert_eq!(ask(&countdown, s, t), Answer::Rearming, "void: countdown");
    let mut empty = w.clone();
    empty.obj_mut(s).unwrap().combat.ammo = 0;
    assert_eq!(ask(&empty, s, t), Answer::OutOfAmmo, "void: no ammunition");
    lock(&mut w, e, v);
    let tick = w.tick;
    let mut countdown = w.clone();
    countdown.obj_mut(s).unwrap().combat.rearm_until = tick + 500;
    assert_eq!(ask(&countdown, s, t), Answer::Illegal, "countdown running");
    let mut empty = w.clone();
    empty.obj_mut(s).unwrap().combat.ammo = 0;
    assert_eq!(ask(&empty, s, t), Answer::Illegal, "out of ammunition");
}

// spec: S-0301 claims 9 and 14 (r80: a building's pending delayed shot answers rearming at the top of rank 4, so a building in attack keeps a target being erased until the delay ends, and the check made then answers illegal)
#[test]
fn claim_9_a_buildings_pending_delayed_shot_comes_first() {
    let mut w = new_game();
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let b = spawn(&mut w, Kind::Building, "TOWER", 0, at(4, 0));
    sleep(&mut w, b);
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    lock(&mut w, e, v);
    let t = Target::Object(v);
    let tick = w.tick;
    let mut pending = w.clone();
    pending.obj_mut(b).unwrap().combat.delayed_shot = Some(tick + 5);
    assert_eq!(ask(&pending, b, t), Answer::Rearming, "pending: rearming");
    assert_eq!(ask(&w, b, t), Answer::Illegal, "none pending: illegal");
}

/// F24's run of parts a to d and f: V of house 1 at the middle; E, A1
/// (SHOOTER), A2 (GUNMAN), B (TOWER) and A1f (SLOW) of house 0 around it.
/// `wait` ticks are run first, so that *L* falls at another place in the
/// 16-tick cycle.
struct F24 {
    w: World,
    v: ObjectId,
    e: ObjectId,
    holders: [ObjectId; 3],
    b: ObjectId,
    a1f: ObjectId,
    l: i32,
    marks: Vec<i32>,
}

fn f24(wait: i32) -> F24 {
    let mut w = new_game();
    glasnost_sim::testkit::run(&mut w, wait);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let a1 = spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, at(-3, 0));
    let a2 = spawn(&mut w, Kind::Infantry, "GUNMAN", 0, at(0, -3));
    let b = spawn(&mut w, Kind::Building, "TOWER", 0, at(4, 0));
    let a1f = spawn(&mut w, Kind::Vehicle, "SLOW", 0, at(0, 3));
    for x in [a1, a2, b, a1f] {
        assert!(order_attack(&mut w, x, v), "the attack order");
    }
    let start = shot_mark(&w, a1f);
    let mut n = 0;
    while shot_mark(&w, a1f) == start {
        glasnost_sim::testkit::run(&mut w, 1);
        n += 1;
        assert!(n < 120, "void unless A1f fires before the lock (F24 f)");
    }
    let l = lock(&mut w, e, v);
    let marks = [a1, a2, b, a1f].map(|x| shot_mark(&w, x)).to_vec();
    F24 {
        w,
        v,
        e,
        holders: [a1, a2, a1f],
        b,
        a1f,
        l,
        marks,
    }
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4: from the lock on no attempt that asks the firing check makes a shot with a weapon that is not temporal; an infantry unit or vehicle that is not a healer keeps the target until its checking turn in the first check tick of S-0309 claim 29 after the lock and drops it then, without the wait of S-0304 claim 2 while the countdown answers; a building in attack drops it at its next attack processing, S-0215 claim 32; a search rejects it, S-0309 table C firing check row); scenario F24 parts a, b, c and f
#[test]
fn f24_a_b_c_f_no_shot_and_the_drops() {
    let mut judged = 0;
    for wait in [0, 5, 11] {
        let mut s = f24(wait);
        let l = s.l;
        if l % CHECK == 0 {
            continue; // F24: part b judges nothing when L is a multiple of 16
        }
        judged += 1;
        let c = (l / CHECK + 1) * CHECK;
        let a1f = s.a1f;
        assert!(
            s.w.obj(a1f).unwrap().combat.rearm_until > c,
            "void unless A1f's countdown still runs at c (F24 f)"
        );
        let watched = [s.holders[0], s.holders[1], s.b, a1f];
        // The tick numbers run from L + 1; check the end of each.
        while s.w.tick <= l + 120 {
            let k = s.w.tick;
            glasnost_sim::testkit::run(&mut s.w, 1);
            assert!(temporal::erased(&s.w, s.v), "the erasure lasts (tick {k})");
            for (i, x) in watched.into_iter().enumerate() {
                assert_eq!(
                    shot_mark(&s.w, x),
                    s.marks[i],
                    "no shot at V from L + 1 (part a, object {i}, tick {k})"
                );
            }
            for x in s.holders {
                let held = target(&s.w, x);
                if k < c {
                    assert_eq!(
                        held,
                        Some(Target::Object(s.v)),
                        "held until the check tick (part b, tick {k}, L {l})"
                    );
                } else {
                    assert_eq!(held, None, "dropped at c and not taken again (tick {k})");
                }
            }
            if k >= l + 2 {
                assert_eq!(target(&s.w, s.b), None, "B dropped it (part c, tick {k})");
            }
        }
        assert!(temporal::holds(&s.w, s.e, s.v), "E's beam stays");
    }
    assert!(judged >= 2, "void unless two runs were judged");
}

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4: a search whose check of the target gives this answer does not pick it, S-0309 table C firing check row and claim 12, whatever the searcher's house; until the last beam lets go, S-0514 claim 10); S-0309 table C (firing check row: the scenario is F24 part d); scenario F24 part d
#[test]
fn f24_d_a_searcher_skips_it_until_the_beam_lets_go() {
    let mut w = new_game();
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let e = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let l = lock(&mut w, e, v);
    run_to(&mut w, l + 2);
    // A3 arrives after the lock (placed in its cell), in its rest behaviour.
    let a3 = spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, at(2, 2));
    let arrived = w.tick;
    while w.tick < arrived + 120 {
        let k = w.tick;
        glasnost_sim::testkit::run(&mut w, 1);
        assert!(temporal::erased(&w, v));
        assert_eq!(target(&w, a3), None, "A3 does not take V (tick {k})");
    }
    // The beam lets go (S-0514 claim 10); E is put back in sleep with no target.
    temporal::let_go(&mut w, e);
    w.obj_mut(e).unwrap().target = None;
    sleep(&mut w, e);
    assert!(!temporal::erased(&w, v), "void unless no beam is attached");
    let r = w.tick;
    let mut taken = false;
    while w.tick <= r + 100 {
        glasnost_sim::testkit::run(&mut w, 1);
        if target(&w, a3) == Some(Target::Object(v)) {
            taken = true;
            break;
        }
    }
    assert!(taken, "A3 holds V by R + 100 (the control)");
}
