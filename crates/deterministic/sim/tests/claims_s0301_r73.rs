//! Claim tests of S-0301 revision 73, claim 9, *A building's own answers
//! above a held temporal target* (Q-2567 answered; M8 step A3 intake 98):
//! every answer of S-0405 table T1 ranks above the held-temporal-target
//! *rearming*, so a building whose temporal beam is attached to its target
//! drops that target while a row of T1 holds, at the first of its next
//! *attack* processing and its checking turn in a check tick of S-0309
//! claim 29, and the beam stays attached. Scenario F22 parts a (switched
//! off), b (short of power) and c (the control). Synthetic rules
//! (`tests/sw_common`): every name and number is invented. The ledger is
//! `docs/m8-claims/intake98.md`.
//!
//! Tick reading: after `tick::step` has run tick *k*, the tick number reads
//! *k* + 1. An order is applied after the turn pass of the tick that runs
//! next (S-0203 claim 7), so a switch order given before tick *w* runs
//! acts from tick *w* + 1's updates on.

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::temporal;
use glasnost_sim::world::{ObjectId, Target, World};
use sw_common::*;

/// F22's rules: every weapon of warhead HE is a temporal weapon (S-0301
/// Terms), so the TOWER's Cannon is *T*'s one test temporal weapon. The
/// TOWER needs power and drains 10 (S-0405 Terms); GEN, a weaponless twin,
/// puts out 100, which covers that with a margin; PLOAD, another weaponless
/// twin, drains 95, so that its drain and the TOWER's together exceed the
/// output while the TOWER's alone does not. The BUGGY, unarmed, is given
/// strength 10 000, so the erasure (S-0514 formula F3) lasts far longer
/// than every run here.
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(10_000);
    let base = r.sim.buildings["TOWER"].clone();
    {
        let t = r.sim.buildings.get_mut("TOWER").unwrap();
        t.powered = Some(true);
        t.econ.power = Some(-10);
    }
    for (name, power) in [("GEN", 100), ("PLOAD", -95)] {
        let mut b = base.clone();
        b.primary = None;
        b.econ.power = Some(power);
        if power < 0 {
            b.powered = Some(true);
        }
        r.sim.buildings.insert(name.into(), b);
        r.registries
            .entry("buildings".into())
            .or_default()
            .push(name.into());
    }
    r
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    /// Switched off (S-0405 claim 10).
    A,
    /// Short of power (S-0405 claims 4 and 7).
    B,
}

/// F22's set-up: GEN, *T* (the TOWER) and PLOAD of ALPHA, PLOAD switched
/// off, and the unarmed *V* (a BUGGY of BETA) within *T*'s range; *T* is
/// ordered to attack *V* and runs until its beam is locked on *V*.
/// Returns the world, *T*, *V*, PLOAD and the lock tick.
fn locked() -> (World, ObjectId, ObjectId, ObjectId, i32) {
    let mut w = game(&rules());
    spawn(&mut w, "GEN", ALPHA, -8, 8);
    let p = spawn(&mut w, "PLOAD", ALPHA, 8, -8);
    glasnost_sim::econ::power::switch(&mut w, p, false);
    let t = spawn(&mut w, "TOWER", ALPHA, 0, 0);
    let v = spawn(&mut w, "BUGGY", BETA, 3, 3);
    // spec: S-0301 F22 (Procedure: the human orders T to attack V without force-fire)
    let ok = orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: t,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(v)),
            dest: None,
        },
    );
    assert!(ok, "set-up: the attack order is taken");
    let lock = until(&mut w, 300, |w| temporal::holds(w, t, v)) - 1;
    assert!(
        !glasnost_sim::econ::power::low_power(&w, ALPHA),
        "set-up: full power"
    );
    (w, t, v, p, lock)
}

/// What one tick shows at its end.
#[derive(Clone, Copy, Debug)]
struct End {
    tick: i32,
    target: Option<Target>,
    beh: Option<Behaviour>,
    /// *T* processed its behaviour in this tick, and its behaviour at the
    /// end of the tick before was *attack*: an *attack* processing.
    attack_processing: bool,
    beam: bool,
    shots: usize,
}

fn tick_end(w: &mut World, t: ObjectId, v: ObjectId) -> End {
    let was_attack = w.obj(t).unwrap().beh.current == Some(Behaviour::Attack);
    step(w);
    let tick = w.tick - 1;
    let o = w.obj(t).unwrap();
    End {
        tick,
        target: o.target,
        beh: o.beh.current,
        attack_processing: was_attack && o.beh.last.is_some_and(|(at, _)| at == tick),
        beam: temporal::holds(w, t, v),
        shots: shots_by(w, t),
    }
}

/// The shots `id` has made so far.
fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

fn check_tick(t: i32) -> bool {
    glasnost_sim::combat::held::check_tick(t)
}

/// One run of part a or b with `k` ticks between the lock and the switch
/// order (part a) or PLOAD's switch-on order (part b). Asserts F22's
/// Expected and returns (*w*, the drop tick, *u*).
fn run_part(part: Part, k: i32) -> (i32, i32, i32) {
    let (mut w, t, v, p, lock) = locked();
    let mut before = Vec::new();
    for _ in 0..k {
        before.push(tick_end(&mut w, t, v));
    }
    let order = match part {
        Part::A => Order::PowerOff { actor: t },
        Part::B => Order::PowerOn { actor: p },
    };
    assert!(
        orders::issue(&mut w, ALPHA, order),
        "set-up: the order is taken"
    );
    let mut after = Vec::new();
    let wt = match part {
        // spec: S-0301 F22 part a (in a tick w after the lock the human switches T off; T has no chargers)
        Part::A => {
            let e = tick_end(&mut w, t, v);
            after.push(e);
            assert!(
                w.obj(t).unwrap().econ.building.switched_off,
                "set-up: switched off in tick {}",
                e.tick
            );
            e.tick
        }
        // spec: S-0301 F22 part b (w is the first tick at whose end the house has low power)
        Part::B => loop {
            let e = tick_end(&mut w, t, v);
            after.push(e);
            if glasnost_sim::econ::power::low_power(&w, ALPHA) {
                break e.tick;
            }
            assert!(after.len() < 40, "set-up: low power comes");
        },
    };
    assert_eq!(
        glasnost_sim::econ::power::chargers(&w, t),
        0,
        "set-up: no chargers"
    );
    // The ticks before w: T holds V in attack and its beam is attached.
    for e in before.iter().chain(after.iter().filter(|e| e.tick < wt)) {
        assert_eq!(e.target, Some(Target::Object(v)), "held before w: {e:?}");
        assert_eq!(e.beh, Some(Behaviour::Attack), "attack before w: {e:?}");
        assert!(e.beam, "attached before w: {e:?}");
    }
    let last = after.last().copied().unwrap();
    assert!(last.beam, "the beam is attached at the end of the tick w");
    assert!(lock < wt, "w after the lock");
    // From w on: the drop at the first of an attack processing and the
    // check tick, with the beam kept.
    let mut tail: Vec<End> = after.iter().copied().filter(|e| e.tick >= wt).collect();
    let is_u = |e: &End| e.tick > wt && (e.attack_processing || check_tick(e.tick));
    while (tail.iter().all(|e| e.target.is_some()) || !tail.iter().any(is_u)) && tail.len() < 40 {
        tail.push(tick_end(&mut w, t, v));
    }
    // spec: S-0301 r73 claim 9 (A building's own answers above a held temporal target: while a row of T1 holds, here not in operation, S-0405 claim 8, the building gets illegal or impossible, not rearming; it drops the target at the first of its next attack processing and its checking turn in a check tick of S-0309 claim 29) and F22 (Expected)
    let u = tail
        .iter()
        .find(|e| is_u(e))
        .map(|e| e.tick)
        .expect("u within the run");
    // spec: S-0309 claim 2 (no passive acquisition while a temporal weapon of its own is erasing a target: the guard that the drop starts does not take V back in the dropping update)
    let drop = tail
        .iter()
        .find(|e| e.target.is_none())
        .copied()
        .expect("dropped");
    // spec: S-0309 claim 2 (while its beam stays attached to V, being erased, the building takes no target by passive acquisition)
    for e in tail.iter().filter(|e| e.tick >= drop.tick && e.beam) {
        assert!(
            e.target.is_none(),
            "no target taken while the beam erases V: {e:?}"
        );
    }
    assert!(
        drop.tick >= wt && drop.tick <= u,
        "dropped from w to u (w {wt}, u {u}): {tail:?}"
    );
    assert!(
        drop.attack_processing || check_tick(drop.tick),
        "the drop is an attack processing or in a check tick: {drop:?}"
    );
    if drop.attack_processing {
        // spec: S-0215 claim 32 (the row for those answers drops the target and starts guard in the same processing)
        assert_eq!(drop.beh, Some(Behaviour::Guard), "guard at that processing");
    }
    // spec: S-0301 r73 claim 9 (dropping the target does not let the beam go, S-0514 claim 14: a building does not fall back to rest in S-0215 claim 13's sense, so the beam stays attached after the drop)
    for e in tail.iter().filter(|e| e.tick <= u.max(drop.tick)) {
        assert!(e.beam, "the beam stays attached from w to u: {e:?}");
    }
    assert!(
        temporal::holds(&w, t, v),
        "the beam stays attached after the drop"
    );
    (wt, drop.tick, u)
}

#[test]
fn r73_claim9_f22_part_a_a_switched_off_building_drops_its_beam_held_target_and_keeps_the_beam() {
    // spec: S-0301 r73 claim 9 (A building's own answers above a held temporal target, Q-2567: while it is not in operation, switched off and not charged, S-0405 claim 8, it does not keep the target its beam holds) and scenario F22 part a (w varied over 10 runs, both parities of T's attack processing ticks and different distances from the next check tick)
    let mut seen = std::collections::BTreeSet::new();
    for k in 0..10 {
        let (wt, drop, u) = run_part(Part::A, k);
        seen.insert((wt % 2, (16 - wt.rem_euclid(16)) % 16));
        assert!(drop <= u, "run {k}");
    }
    assert!(
        seen.iter().any(|s| s.0 == 0) && seen.iter().any(|s| s.0 == 1),
        "both parities: {seen:?}"
    );
    assert!(
        seen.len() >= 5,
        "different distances to the check tick: {seen:?}"
    );
}

#[test]
fn r73_claim9_f22_part_b_a_building_short_of_power_drops_its_beam_held_target_and_keeps_the_beam() {
    // spec: S-0301 r73 claim 9 (A building's own answers above a held temporal target, Q-2567: while it is not in operation, short of power and not charged, S-0405 claim 8, it does not keep the target its beam holds) and scenario F22 part b
    let mut seen = std::collections::BTreeSet::new();
    for k in 0..10 {
        let (wt, _, _) = run_part(Part::B, k);
        seen.insert(wt % 2);
    }
    assert_eq!(seen.len(), 2, "both parities");
}

#[test]
fn r73_claim9_f22_part_c_the_control_keeps_the_target_and_the_beam() {
    // spec: S-0301 F22 part c (no switch order after the lock and PLOAD left switched off: T holds V and stays in attack just after every update from the lock, its attack processings fall every other tick, it makes no second shot, and its beam stays attached) and S-0215 claim 32 (the rearming row: delay 2)
    let (mut w, t, v, _, _) = locked();
    // The effects list is never drained here, so it counts every shot.
    let first = shots_by(&w, t);
    assert!(first >= 1, "set-up: the locking shot");
    let mut ends = Vec::new();
    for _ in 0..100 {
        ends.push(tick_end(&mut w, t, v));
    }
    let mut processings = Vec::new();
    for e in &ends {
        assert_eq!(e.target, Some(Target::Object(v)), "{e:?}");
        assert_eq!(e.beh, Some(Behaviour::Attack), "{e:?}");
        assert!(e.beam, "{e:?}");
        assert_eq!(e.shots, first, "no second shot: {e:?}");
        if e.attack_processing {
            processings.push(e.tick);
        }
    }
    assert!(processings.len() >= 40, "{processings:?}");
    for p in processings.windows(2) {
        assert_eq!(p[1] - p[0], 2, "every other tick: {processings:?}");
    }
}

#[test]
fn r73_claim9_t1_answers_rank_above_the_held_temporal_rearming() {
    // spec: S-0301 r73 claim 9 (every answer of S-0405 table T1 ranks above the held-temporal-target rearming, as it does above a pending delayed shot's rearming, and no T1 answer ranks below it; so a building whose temporal beam is attached to its target gets illegal or impossible for that target, not rearming, while a row of T1 holds)
    let (mut w, t, v, _, _) = locked();
    assert_eq!(
        check(&w, t, Some(Target::Object(v)), 0, true),
        Answer::Rearming,
        "in operation: the held-temporal-target rearming"
    );
    glasnost_sim::econ::power::switch(&mut w, t, false);
    assert!(temporal::holds(&w, t, v), "set-up: still attached");
    assert_eq!(
        check(&w, t, Some(Target::Object(v)), 0, true),
        Answer::Impossible,
        "switched off: T1's impossible, not rearming"
    );
}

// spec: S-0301 r73 claim 9 (every answer of S-0405 table T1 ranks above the held-temporal-target rearming); S-0405 T1 (illegal: being drained, S-0608 Terms; building or selling, not a pulse cannon) (seam sweep 6: intake 98's other T1 rows; the type rows, a pulse cannon and an occupiable building without firing occupants, answer from the start, so such a building never locks its beam)
#[test]
fn seams6_the_other_t1_rows_rank_above_the_held_temporal_rearming() {
    let (w, t, v, p, _) = locked();
    assert_eq!(
        check(&w, t, Some(Target::Object(v)), 0, true),
        Answer::Rearming,
        "set-up: the held-temporal-target rearming"
    );
    let mut drained = w.clone();
    drained.drain.links.insert(t, p);
    assert!(glasnost_sim::econ::power::being_drained(&drained, t));
    let mut selling = w.clone();
    selling.obj_mut(t).unwrap().beh.current = Some(Behaviour::Selling);
    let mut building = w.clone();
    building.obj_mut(t).unwrap().beh.current = Some(Behaviour::Construction);
    for (row, w) in [
        ("drained", drained),
        ("selling", selling),
        ("construction", building),
    ] {
        assert!(temporal::holds(&w, t, v), "set-up: still attached ({row})");
        assert_eq!(
            check(&w, t, Some(Target::Object(v)), 0, true),
            Answer::Illegal,
            "{row}: T1's illegal, not rearming"
        );
    }
}
