//! Claim tests of S-0513 revision 89, claim 39a (Q-2316 and Q-2317
//! answered), with the matching clauses of claim 51, S-0514 claims 17 and
//! 18, S-0309 claim 27 and S-0203 claim 4 (M8 step A3 intake 81): a warp
//! member's sparkles and drops in its warp turns, with a beam and without
//! one, and the firer whose temporal beam holds a member while it waits to
//! land. Synthetic rules (`tests/sw_common`): every name and number is
//! invented. The ledger is `docs/m8-claims/intake81.md`. Revision 91's
//! building firer, firers beyond range (Q-2376) and a member removed by its
//! erasure update in a sparkle tick (Q-2377) are the last sections (M8 step
//! A3 intake 84, ledger `docs/m8-claims/intake84.md`).
//!
//! Tick reading: after `tick::step` has run tick *k*, the tick number reads
//! *k* + 1. The warp order is applied after the turn pass of tick *t*_L
//! (S-0203 claim 7). With a clean landing the member enters its leaving
//! stage in *t*_L + 1, departs in *t*_L + 62, appears in *t*_L + 63, has
//! its arrival effects in *t*_L + 64 and ends phasing in in *t*_L + 65
//! (S-0513 claims 39 and 40, SW12 part d).

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::temporal;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use sw_common::*;

/// S-0514 Constants: the sparkle period, on the global tick number.
const SPARKLE: i32 = 24;
/// S-0309 claim 29's check period, written out for the chosen ticks.
const PERIOD: i32 = 16;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(7);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    // The imported sparkle animation exists (S-0514 claim 18).
    r.sim.general.chrono_sparkle = Some("SPARK".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA", "SPARK"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    // Every weapon of warhead HE is a temporal weapon (S-0301 Terms): the
    // CART's Gun and the TOWER's Cannon. Every type of the members is given
    // strength 10 000, so no erasure here completes.
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    for t in [
        r.sim.vehicles.get_mut("BUGGY"),
        r.sim.infantry.get_mut("JUMPER"),
    ]
    .into_iter()
    .flatten()
    {
        t.strength = Some(10_000);
    }
    r
}

fn keeps_beam(on: bool) -> Ruleset {
    let mut r = rules();
    r.globals.quirks.warp_departure_keeps_beam = on;
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

/// A game with ALPHA's teleport source ready.
fn ready(r: &Ruleset) -> World {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    w
}

/// Launches ALPHA's warp of the block at `at(0, 0)` to `at(5, 5)` in the
/// first tick from now whose number leaves `rem` when divided by
/// `modulus` and in which orders are taken; returns t_L, the tick that
/// applied the warp order, which has run.
fn launch(w: &mut World, modulus: i32, rem: i32) -> i32 {
    fire_now(w, ALPHA, SOURCE, at(0, 0));
    while orders::blocked(w) || w.tick.rem_euclid(modulus) != rem {
        step(w);
    }
    fire(w, ALPHA, WARP, at(5, 5));
    let tl = w.tick;
    step(w);
    tl
}

/// Runs until tick `t` has run (the next tick to run is `t + 1`).
fn run_through(w: &mut World, t: i32) {
    while w.tick <= t {
        step(w);
    }
}

/// S-0514 claim 18's placement for an infantry unit or vehicle: 120
/// leptons farther along each horizontal axis, same height.
fn placed(p: Pos) -> Pos {
    Pos::new(p.x + 120, p.y + 120, p.z) // spec: S-0514 Constants (mobile sparkle displacement 120 leptons)
}

/// One run of the sparkle record: each tick's sparkles and the member's
/// stage after it.
struct Record {
    sparkles: Vec<(i32, Pos)>,
    stages: Vec<(i32, Option<Stage>)>,
}

/// Runs through tick `last`, recording the sparkle animations created in
/// each tick, with their positions, and `m`'s stage after each tick.
fn record(w: &mut World, m: ObjectId, last: i32) -> Record {
    let mut rec = Record {
        sparkles: Vec::new(),
        stages: Vec::new(),
    };
    while w.tick <= last {
        w.effects.clear();
        let t = w.tick;
        step(w);
        for e in &w.effects {
            if let Effect::Anim { name, pos, .. } = e
                && name == "SPARK"
            {
                rec.sparkles.push((t, *pos));
            }
        }
        rec.stages.push((t, stage(w, m)));
    }
    rec
}

fn stage_after(rec: &Record, t: i32) -> Option<Stage> {
    rec.stages
        .iter()
        .find(|(k, _)| *k == t)
        .and_then(|(_, s)| *s)
}

/// The clean timeline of claim 39 (SW12 part d), read from the record.
fn assert_clean_timeline(rec: &Record, tl: i32) {
    assert_eq!(stage_after(rec, tl + 61), Some(Stage::Expired));
    assert_eq!(
        stage_after(rec, tl + 62),
        Some(Stage::Away),
        "departs in t_L + 62"
    );
    assert_eq!(
        stage_after(rec, tl + 63),
        Some(Stage::Placed),
        "appears in t_L + 63"
    );
    assert!(
        matches!(stage_after(rec, tl + 64), Some(Stage::Hold(_))),
        "arrival effects in t_L + 64"
    );
    assert_eq!(
        stage_after(rec, tl + 65),
        None,
        "phasing in ends in t_L + 65"
    );
}

/// The expected sparkles from `from` to `to`, both included: one in each
/// tick divisible by 24, from the old position up to the appearance tick
/// t_L + 63 and from the landing point from the arrival-effects tick on.
fn expected(tl: i32, from: i32, to: i32, old: Pos, landing: Pos) -> Vec<(i32, Pos)> {
    (from..=to)
        .filter(|t| t.rem_euclid(SPARKLE) == 0)
        .map(|t| {
            let p = if t <= tl + 63 { old } else { landing };
            (t, placed(p))
        })
        .collect()
}

/// The launch residues modulo 24 that put t_L + 1, the departure, the
/// appearance, the arrival effects and the phasing end on a sparkle tick
/// (SW12 part l6c).
const RESIDUES: [(i32, i32); 5] = [(23, 1), (10, 62), (9, 63), (8, 64), (7, 65)];

// --- Sparkles in the warp's turns (SW12 parts l6 and l6c) -------------------------------

#[test]
fn claim39a_l6c_a_member_with_no_beam_sparkles_in_its_warp_turns_from_tl_plus_2_to_its_phasing_end()
{
    // spec: S-0203 r43 claim 4 (S-0514 claim 18's sparkle is no suppressed ordinary work: made before the warp work in each warp turn in which the member is being erased or phasing in)
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns, Q-2316: each turn of a member at every stage is one of its turns for S-0514 claim 18, with or without a beam; in its leaving stage it counts as being erased and from its departure it phases in; a sparkle in each turn whose tick is divisible by 24 from its update of t_L + 2 to the update that ends its phasing in, both included; its state before the turn's warp step decides, so t_L + 1 makes none; placed from its position before the warp step: the old position up to its appearance update, the landing point from its arrival-effects update; none after its phasing in ends) and claim 51 (the sparkle is an intrinsic exception, made before the warp work); S-0514 claim 18; S-0203 claim 4
    for name in ["BUGGY", "JUMPER"] {
        for (rem, off) in RESIDUES {
            let mut w = ready(&rules());
            let m = spawn(&mut w, name, BETA, 0, 0);
            let old = w.obj(m).unwrap().pos;
            let tl = launch(&mut w, SPARKLE, rem);
            assert_eq!(stage(&w, m), Some(Stage::Pending));
            assert_eq!((tl + off).rem_euclid(SPARKLE), 0, "set-up");
            let rec = record(&mut w, m, tl + 90);
            assert_clean_timeline(&rec, tl);
            let landing = w.obj(m).unwrap().pos;
            assert_ne!(landing.cell(), old.cell());
            assert_eq!(
                rec.sparkles,
                expected(tl, tl + 2, tl + 65, old, landing),
                "{name}, launch residue {rem}"
            );
            if off != 1 {
                assert!(
                    rec.sparkles.iter().any(|(t, _)| *t == tl + off),
                    "{name}: a sparkle in t_L + {off}"
                );
            }
        }
    }
}

#[test]
fn claim39a_l6_a_member_with_a_beam_sparkles_once_per_turn_and_not_after_its_phasing_end() {
    // spec: S-0203 r43 claim 4 (the sparkle before the warp work in the member's warp turns)
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns: the same sparkles with a beam attached, one per turn; after its phasing in has ended a member whose beam was locked before its departure sparkles only while it is being erased, so none while that beam stays attached; Switch: these rules hold in both settings of quirks.warp_departure_keeps_beam, for the states that arise in each: off, the lock is refused and the sparkles are those of a member with no beam) and S-0514 claims 3 and 18
    for on in [true, false] {
        for (rem, off) in RESIDUES {
            let mut w = ready(&keeps_beam(on));
            let f = spawn(&mut w, "CART", ALPHA, -8, 8);
            let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
            let old = w.obj(m).unwrap().pos;
            let tl = launch(&mut w, SPARKLE, rem);
            step(&mut w);
            assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
            // The detonation of f's temporal projectile at the member, in flight at the launch,
            // after the member's turn of t_L + 1 (claim 39a, The lock routes and The lock).
            temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
            assert_eq!(temporal::holds(&w, f, m), on, "switch {on}");
            let rec = record(&mut w, m, tl + 100);
            assert_clean_timeline(&rec, tl);
            assert_eq!(temporal::attached(&w, m), on, "the beam stays attached");
            assert!(!temporal::erased(&w, m));
            let landing = w.obj(m).unwrap().pos;
            assert_eq!(
                rec.sparkles,
                expected(tl, tl + 2, tl + 65, old, landing),
                "switch {on}, launch residue {rem}"
            );
            if off != 1 {
                assert!(rec.sparkles.iter().any(|(t, _)| *t == tl + off));
            }
        }
    }
}

#[test]
fn claim39a_a_lock_before_the_members_turn_of_tl_plus_1_makes_that_turn_sparkle() {
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns: its update of t_L + 1, in which it enters the leaving stage, makes none unless a beam was locked on it before that turn; its state before the turn's warp step decides) and S-0514 claim 18
    let mut w = ready(&rules());
    let f = spawn(&mut w, "CART", ALPHA, -8, 8);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let old = w.obj(m).unwrap().pos;
    let tl = launch(&mut w, SPARKLE, 23);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    let rec = record(&mut w, m, tl + 30);
    assert_eq!(
        rec.sparkles,
        vec![(tl + 1, placed(old)), (tl + 25, placed(old))],
        "a sparkle in t_L + 1, being erased before the warp step"
    );
}

#[test]
fn claim39a_a_later_lock_after_the_phasing_end_sparkles_again_once_per_turn() {
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns: after its phasing in has ended the member sparkles only while it is being erased, that is from a later lock on; A lock after the departure) and S-0514 claim 18 (one sparkle per turn of an object being erased)
    let mut w = ready(&rules());
    let f = spawn(&mut w, "CART", ALPHA, -8, 8);
    let g = spawn(&mut w, "CART", ALPHA, -8, 6);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, SPARKLE, 0);
    step(&mut w);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    run_through(&mut w, tl + 66);
    assert_eq!(stage(&w, m), None);
    let rec = record(&mut w, m, tl + 71);
    assert!(rec.sparkles.is_empty(), "attached, not being erased");
    temporal::deliver(&mut w, Some(g), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m), "being erased again");
    let p = w.obj(m).unwrap().pos;
    let from = w.tick;
    let rec = record(&mut w, m, from + 50);
    let want: Vec<(i32, Pos)> = (from..=from + 50)
        .filter(|t| t.rem_euclid(SPARKLE) == 0)
        .map(|t| (t, placed(p)))
        .collect();
    assert_eq!(rec.sparkles, want);
}

#[test]
fn claim39a_a_member_phasing_in_and_being_erased_in_its_hold_sparkles_once() {
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns: a member makes S-0514 claim 18's sparkle in each of its turns, one per turn, whether it is being erased, phasing in or both) and S-0514 claim 18
    // A redirected landing gives a hold of the teleport delay, 7 (claims 37 and 40); the lock
    // comes in the hold, after the departure, and the hold covers a sparkle tick.
    let set_up = |rem: i32| {
        let mut w = ready(&rules());
        let f = spawn(&mut w, "CART", ALPHA, -8, 8);
        let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
        spawn(&mut w, "TOWER", BETA, 5, 5);
        let tl = launch(&mut w, SPARKLE, rem);
        until(&mut w, 120, |w| matches!(stage(w, m), Some(Stage::Hold(_))));
        let Some(Stage::Hold(ta)) = stage(&w, m) else {
            unreachable!()
        };
        (w, f, m, tl, ta)
    };
    // A probe run gives the hold's start after the launch; the run proper
    // puts a sparkle tick three ticks into the hold.
    let (_, _, _, tl0, ta0) = set_up(0);
    let s = ta0 - tl0 + 3;
    let (mut w, f, m, tl, ta) = set_up((-s).rem_euclid(SPARKLE));
    assert_eq!(ta - tl, s - 3, "set-up");
    assert!(
        w.tick < ta + 3,
        "set-up: the lock comes before the sparkle tick"
    );
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    let rec = record(&mut w, m, ta + 5);
    assert!(
        matches!(stage_after(&rec, ta + 3), Some(Stage::Hold(_))),
        "set-up: still in the hold"
    );
    let at: Vec<i32> = rec.sparkles.iter().map(|(t, _)| *t).collect();
    assert_eq!(at, vec![ta + 3], "one sparkle, not one per cause");
}

#[test]
fn s0514_r14_claim18_the_sparkle_is_an_ordinary_animation_whose_creation_draw_precedes_the_warp_step()
 {
    // spec: S-0514 r14 claim 18 (the sparkle is an ordinary animation with the simulation effects S-0109 gives one, its playback draws included; a type whose creation takes a draw puts it in the scenario random stream's order before the warp step of a warp member's turn) and S-0513 claim 39a (the sparkle's draws come before the departure's warp-out animation); S-0109 claims 3 and 29
    let mut r = rules();
    // A random rate pair whose range holds more than one delay: one ranged
    // creation draw each (S-0109 claim 3).
    for n in ["SPARK", "WARPA"] {
        r.sim.animations.get_mut(n).unwrap().random_rate = Some([450, 225]);
    }
    let mut w = ready(&r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, SPARKLE, 10);
    run_through(&mut w, tl + 61);
    assert_eq!(
        (tl + 62).rem_euclid(SPARKLE),
        0,
        "set-up: the departure on a sparkle tick"
    );
    w.effects.clear();
    step(&mut w);
    assert!(superweapons::warp::departed(&w, m));
    let made: Vec<(&str, bool, bool)> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim {
                name, delay, id, ..
            } if name == "SPARK" || name == "WARPA" => {
                Some((name.as_str(), delay.is_some(), id.is_some()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        made,
        vec![("SPARK", true, true), ("WARPA", true, true)],
        "the sparkle, an animation in play with its creation draw, before the warp-out animation"
    );
}

/// The member's targeting timer after its expiry update in t_L + 61, a
/// sparkle tick, with the sparkle animation taking a creation draw or not.
fn expiry_timer(drawing_sparkle: bool) -> i32 {
    let mut r = rules();
    if drawing_sparkle {
        r.sim.animations.get_mut("SPARK").unwrap().random_rate = Some([450, 225]);
    }
    let mut w = ready(&r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, SPARKLE, 11);
    assert_eq!(
        (tl + 61).rem_euclid(SPARKLE),
        0,
        "set-up: the expiry on a sparkle tick"
    );
    run_through(&mut w, tl + 60);
    // No target and more than 10 ticks left: the expiry search draws.
    let t = w.tick;
    w.obj_mut(m).unwrap().targeting_timer = t + 400;
    w.effects.clear();
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    let sparkled = w
        .effects
        .iter()
        .any(|e| matches!(e, Effect::Anim { name, delay, .. } if name == "SPARK" && delay.is_some() == drawing_sparkle));
    assert!(sparkled, "the sparkle of the expiry update");
    w.obj(m).unwrap().targeting_timer
}

#[test]
fn s0203_r43_the_sparkles_creation_draw_comes_before_the_expiry_updates_target_search() {
    // spec: S-0203 r43 claim 4 (the sparkle is made before the turn's warp work) and S-0513 claim 39a (Sparkles: any draw that creating it takes comes before the draws of the turn's warp step, the expiry update's target search among them); S-0207 claim 20 (f) (the search's one draw over 4 to 8 ticks); S-0109 claim 3
    // The sparkle's draw moves the search's draw one place on in the
    // stream, so the search's result differs from the run whose sparkle
    // takes no draw. Were the search first, it would take the same value in
    // both runs.
    assert_ne!(
        expiry_timer(true),
        expiry_timer(false),
        "the search's draw follows the sparkle's"
    );
}

// --- Drops in the warp's turns -----------------------------------------------------------

/// Gives `m` a target and a destination directly, as an order would.
fn give(w: &mut World, m: ObjectId, x: ObjectId) {
    let o = w.obj_mut(m).unwrap();
    o.target = Some(Target::Object(x));
    o.dest = Some(Target::Cell(at(3, 9)));
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().target
}

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

#[test]
fn claim39a_a_member_with_no_beam_drops_a_given_target_in_each_leaving_turn_the_expiry_included() {
    // spec: S-0203 r43 claim 4 (S-0514 claim 17's clearing is no suppressed ordinary work: made after the warp work in each turn in which the member is being erased)
    // spec: S-0513 claim 39a (Drops: in each turn in which the member is still being erased after the turn's warp step, it clears its target and destination as S-0514 claim 17 gives: its updates from t_L + 1 to its expiry update in t_L + 61, with or without a beam) and claim 48 (a turn still in the leaving stage after its warp update, the wait expiry included, drops target and destination); claim 51
    let mut w = ready(&rules());
    let x = spawn(&mut w, "WALKER", ALPHA, -8, -8);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, 1, 0);
    for t in [tl + 10, tl + 40, tl + 60] {
        run_through(&mut w, t);
        give(&mut w, m, x);
        step(&mut w);
        assert!(superweapons::being_erased(&w, m));
        assert_eq!(
            (target(&w, m), dest(&w, m)),
            (None, None),
            "dropped in t_L + {}",
            t + 1 - tl
        );
    }
    assert_eq!(stage(&w, m), Some(Stage::Expired), "the expiry update");
}

#[test]
fn claim39a_a_member_with_no_beam_keeps_a_given_target_from_its_departure_update() {
    // spec: S-0513 claim 39a (Drops: only a turn in which the member is still being erased after the warp step, or whose turns are cut short by a beam, drops; the departure update ends its being erased) and claim 51 (the warp work is the whole turn: no other clearing)
    let mut w = ready(&rules());
    let x = spawn(&mut w, "WALKER", ALPHA, -8, -8);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, 1, 0);
    for t in [tl + 61, tl + 62, tl + 63] {
        run_through(&mut w, t);
        give(&mut w, m, x);
        step(&mut w);
        assert_eq!(
            target(&w, m),
            Some(Target::Object(x)),
            "kept in t_L + {}",
            t + 1 - tl
        );
    }
}

/// A member being erased in its leaving stage by `f`'s beam, locked after
/// its turn of t_L + 1; returns the world after tick `upto`.
fn locked(r: &Ruleset, name: &str, upto: i32) -> (World, ObjectId, ObjectId, ObjectId, i32) {
    let mut w = ready(r);
    let f = spawn(&mut w, "CART", ALPHA, -8, 8);
    let x = spawn(&mut w, "WALKER", ALPHA, -8, -8);
    let m = spawn(&mut w, name, BETA, 0, 0);
    let tl = launch(&mut w, 1, 0);
    step(&mut w);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    run_through(&mut w, tl + upto);
    (w, f, x, m, tl)
}

#[test]
fn claim39a_a_vehicle_member_with_a_beam_drops_in_its_departure_and_off_map_turns() {
    // spec: S-0203 r43 claim 4 (the clearing after the warp work for a vehicle phasing in with a beam attached)
    // spec: S-0513 claim 39a (Drops: the departure update ends its being erased, so it makes this drop there as a vehicle or aircraft with a beam attached, which Departure gives; Departure: while it phases in, a vehicle with a beam attached has its turns cut short) and claim 51 (S-0514 claim 17's clearing after the warp work in each turn in which the member is a vehicle or aircraft phasing in with a beam attached); S-0514 claim 17
    let (mut w, _, x, m, tl) = locked(&rules(), "BUGGY", 61);
    for t in [tl + 61, tl + 62, tl + 63] {
        run_through(&mut w, t);
        give(&mut w, m, x);
        step(&mut w);
        assert!(!temporal::erased(&w, m));
        assert_eq!(
            (target(&w, m), dest(&w, m)),
            (None, None),
            "dropped in t_L + {}",
            t + 1 - tl
        );
    }
}

#[test]
fn claim39a_an_infantry_member_with_a_beam_drops_off_the_map_only_with_the_switch_off() {
    // spec: S-0203 r43 claim 4 (the clearing after the warp work for an infantry member phasing in with a beam attached, with quirks.phase_in_beam_spares_infantry off)
    // spec: S-0513 claim 39a and Edge cases (quirks.phase_in_beam_spares_infantry: on, an infantry member phasing in with a beam attached and not being erased is not cut short and makes no drop; off, it is cut short and drops) and claim 51 (the clearing after the warp work for an infantry member in that state with the switch off); S-0514 claim 17
    for spares in [true, false] {
        let mut r = rules();
        r.globals.quirks.phase_in_beam_spares_infantry = spares;
        let (mut w, _, x, m, tl) = locked(&r, "JUMPER", 62);
        assert!(superweapons::warp::departed(&w, m));
        give(&mut w, m, x);
        step(&mut w);
        assert_eq!(stage(&w, m), Some(Stage::Placed));
        assert_eq!(
            target(&w, m).is_some(),
            spares,
            "switch {spares}: kept only when not cut short (t_L + 63 = {})",
            tl + 63
        );
    }
}

#[test]
fn claim39a_a_lock_while_the_member_waits_to_land_makes_it_drop_whatever_its_kind() {
    // spec: S-0513 claim 39a (Sparkles and drops in the warp's turns: a lock while it waits to land, or in its placement or arrival-effects update, makes it being erased again, with the drop in each turn from then on, for every kind of member) and S-0514 r13 claim 1 (a departed member off the map is an ordinary object target for the lock)
    for name in ["JUMPER", "BUGGY"] {
        let mut w = ready(&rules());
        let f = spawn(&mut w, "CART", ALPHA, -8, 8);
        let x = spawn(&mut w, "WALKER", ALPHA, -8, -8);
        let m = spawn(&mut w, name, BETA, 0, 0);
        let tl = launch(&mut w, 1, 0);
        run_through(&mut w, tl + 62);
        assert!(superweapons::warp::departed(&w, m));
        assert!(!temporal::attached(&w, m), "no beam before");
        temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
        assert!(temporal::erased(&w, m), "{name}: being erased");
        for k in [63, 64] {
            give(&mut w, m, x);
            step(&mut w);
            assert_eq!(
                (target(&w, m), dest(&w, m)),
                (None, None),
                "{name}: dropped in t_L + {k}"
            );
        }
        assert!(matches!(stage(&w, m), Some(Stage::Hold(_))));
        assert_eq!(w.tick, tl + 65);
    }
}

// --- The firer while the member waits to land (SW12 part l3, Q-2317) --------------------

/// SW12 part l3's set-up on our engine: a firer `f` of type `name`, placed
/// after the launch within its weapon's range of the member's old position,
/// whose beam is locked on the member after the member's turn of t_L + 1
/// and which is then given *attack* on it. An unprotected TOWER of the
/// member's owner on the landing cell redirects the landing (claim 37), so
/// the member waits to land for more than one tick.
struct Waiting {
    w: World,
    f: ObjectId,
    m: ObjectId,
    /// t_L.
    tl: i32,
    /// The departure tick.
    dep: i32,
    /// The appearance tick.
    app: i32,
}

fn waiting_at(r: &Ruleset, name: &str, fx: i32, fy: i32, rem: i32) -> Waiting {
    let mut w = ready(r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    spawn(&mut w, "TOWER", BETA, 5, 5);
    let tl = launch(&mut w, PERIOD, rem);
    let f = spawn(&mut w, name, ALPHA, fx, fy);
    step(&mut w);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::holds(&w, f, m));
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(m));
    o.combat.opportunity = false;
    o.targeting_timer = t + 400;
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    // The departure and appearance ticks, read from a copy.
    let mut p = w.clone();
    let dep = until(&mut p, 5, |p| superweapons::warp::departed(p, m)) - 1;
    let app = until(&mut p, 30, |p| stage(p, m) == Some(Stage::Placed)) - 1;
    assert_eq!(dep, tl + 62, "departs in t_L + 62");
    assert!(
        app > dep + 1,
        "set-up: it waits to land for more than one tick"
    );
    Waiting {
        w,
        f,
        m,
        tl,
        dep,
        app,
    }
}

/// As [`waiting_at`], with the launch chosen so that the tick after the
/// departure, in which the member is off the map at the start of its
/// firer's turn, is a check tick of S-0309 claim 29.
fn waiting(r: &Ruleset, name: &str, fx: i32, fy: i32) -> Waiting {
    let probe = waiting_at(r, name, fx, fy, 0);
    let rem = (probe.tl - probe.dep - 1).rem_euclid(PERIOD);
    let run = waiting_at(r, name, fx, fy, rem);
    assert!(held_check(run.dep + 1), "set-up: a check tick off the map");
    run
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

/// The firer's record of SW12 part l3 for Q-2317: target, behaviour, cell
/// and body and turret facing, and an infantry firer's attack state and
/// sequence (S-0107 claim 11).
fn firer_state(
    w: &World,
    f: ObjectId,
) -> (
    Option<Target>,
    Option<Behaviour>,
    glasnost_map::CellXY,
    impl PartialEq + std::fmt::Debug + use<>,
) {
    let o = w.obj(f).unwrap();
    (
        o.target,
        o.beh.current,
        o.pos.cell(),
        (
            o.body.target,
            o.turret.target,
            o.infantry.as_ref().map(|s| (s.attack, s.seq)),
        ),
    )
}

/// The rank-4 answers that S-0301 claim 9 ranks above S-0514 claim 19's
/// *rearming* and that read the target, asked about the departed member:
/// none holds for it off the map (claim 39a, *The firer*: rearming unless
/// an outranking answer holds; the member is not airborne, not protected and
/// not hidden, so its being off the map makes none of them answer).
fn assert_no_outranking_answer(w: &World, f: ObjectId, m: ObjectId) {
    assert!(!w.obj(m).unwrap().on_map, "off the map");
    assert!(
        !glasnost_sim::combat::weapon::airborne_by_height(w, m),
        "S-0306 claim 23: not airborne off the map"
    );
    assert!(
        !superweapons::protected(w, m),
        "S-0513 claim 27: not protected"
    );
    let owner = w.obj(f).unwrap().owner;
    assert!(
        !glasnost_sim::cloak::hidden_from(w, m, owner),
        "S-0516 claim 24: not hidden from the firer's house"
    );
}

/// The record from the departure update to the appearance update, both
/// included: in each tick the firer keeps the member, stays in *attack*,
/// makes no shot and keeps its cell and facing; its firing check about the
/// member answers *rearming*; the beam is attached after the appearance.
/// Returns the firer's behaviour processings in that span, as the tick of
/// each and the delay it chose.
fn assert_kept_while_waiting(s: Waiting) -> Vec<(i32, i32)> {
    let mut processings = Vec::new();
    let Waiting {
        mut w,
        f,
        m,
        tl,
        dep,
        app,
    } = s;
    let before = firer_state(&w, f);
    assert_eq!(before.0, Some(Target::Object(m)));
    assert_eq!(before.1, Some(Behaviour::Attack));
    for t in dep..=app {
        assert_eq!(w.tick, t);
        w.effects.clear();
        step(&mut w);
        assert_eq!(shots_by(&w, f), 0, "no shot in t_L + {}", t - tl);
        if let Some((k, d)) = w.obj(f).unwrap().beh.last
            && k == t
        {
            processings.push((k, d));
        }
        let now = firer_state(&w, f);
        if t < app {
            assert_eq!(now, before, "unchanged in t_L + {}", t - tl);
        } else {
            // The appearance tick: the record is taken just after the
            // member's update, and the firer's own turn later in the tick
            // may already aim at the landing point (claim 39a: the first
            // change comes at the appearance). Target, behaviour and cell
            // are unchanged.
            assert_eq!(
                (now.0, now.1, now.2),
                (before.0, before.1, before.2),
                "unchanged in the appearance tick t_L + {}",
                t - tl
            );
        }
        if superweapons::warp::departed(&w, m) {
            assert_no_outranking_answer(&w, f, m);
            assert_eq!(
                check(&w, f, Some(Target::Object(m)), 0, true),
                Answer::Rearming,
                "S-0514 claim 19's rearming while off the map"
            );
            // The firing step's target is the member, so an infantry
            // firer's no-target row (S-0107 claim 11) is not reached.
            assert_eq!(
                glasnost_sim::combat::held::firing_target(&w, w.obj(f).unwrap()),
                Some(Target::Object(m)),
                "the firing step's target while off the map"
            );
        }
    }
    assert_eq!(stage(&w, m), Some(Stage::Placed), "appeared");
    assert!(
        temporal::holds(&w, f, m),
        "the beam is attached after the appearance"
    );
    processings
}

fn held_check(t: i32) -> bool {
    glasnost_sim::combat::held::check_tick(t)
}

#[test]
fn claim39a_l3_an_infantry_firer_keeps_a_member_that_waits_to_land_with_its_turns_unchanged() {
    // spec: S-0513 claim 39a (The firer, Q-2317: while the member waits to land, from its departure update to its appearance, an infantry or vehicle firer whose beam is attached to it, within its weapon's range of its old position, whose firing check answers rearming, keeps it as its target and its turns go on as before: no shot, S-0309 claim 29's check keeps it, its range and aim read the old position, so it neither moves nor turns, its behaviour stays) and Changes for S-0309 claim 27 (the departure is not claim 27's drop); S-0514 claim 19
    // spec: S-0309 r95 claim 27 (a firer whose temporal beam is attached is the exception: rearming; while the member waits to land, an infantry firer within range keeps it, makes no shot and neither moves nor turns; measurable, SW12 part l3)
    let mut r = rules();
    // The WALKER's Rifle (warhead SA) is a temporal weapon here.
    r.sim.warheads.get_mut("SA").unwrap().temporal = Some(true);
    assert_kept_while_waiting(waiting(&r, "WALKER", -2, -2));
}

#[test]
fn claim39a_l3_an_infantry_firer_in_its_attack_state_stays_in_it_while_the_member_waits_to_land() {
    // spec: S-0513 claim 39a (The firer, Q-2317: while the member waits to land the infantry firer keeps it as its target and its turns go on as before the departure) and S-0309 r95 claim 27 (the departure is not claim 27's drop for a firer whose temporal beam is attached); S-0107 claim 11 (the attack state ends when the unit has no target, when its target changes, at a frozen stage and when it is not ready at the fire frame; none of these comes here)
    let mut r = rules();
    r.sim.warheads.get_mut("SA").unwrap().temporal = Some(true);
    let Waiting {
        mut w,
        f,
        m,
        tl,
        dep,
        app,
    } = waiting(&r, "WALKER", -2, -2);
    assert_eq!(w.tick, dep);
    // The firer in its attack state at the member, at a stage that is not
    // its fire frame and is not frozen, as after a shot.
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.combat.last_shot_target = Some(Target::Object(m));
    let s = o.infantry.as_mut().unwrap();
    s.attack = true;
    s.seq_delay = 10_000;
    s.seq_t0 = t - 10_000;
    for t in dep..=app {
        step(&mut w);
        let o = w.obj(f).unwrap();
        assert_eq!(o.target, Some(Target::Object(m)), "t_L + {}", t - tl);
        assert!(
            o.infantry.as_ref().unwrap().attack,
            "still in the attack state after t_L + {}",
            t - tl
        );
    }
    assert!(temporal::holds(&w, f, m));
}

#[test]
fn claim39a_a_vehicle_firer_keeps_a_member_that_waits_to_land_with_its_turns_unchanged() {
    // spec: S-0513 claim 39a (The firer, Q-2317: the same for a vehicle firer, unmeasured) and S-0514 claim 19
    // spec: S-0309 r95 claim 27 (the same for a vehicle firer, unmeasured)
    assert_kept_while_waiting(waiting(&rules(), "CART", -3, -2));
}

#[test]
fn claim39a_a_deactivated_vehicle_firer_drops_a_member_that_waits_to_land_at_the_check() {
    // spec: S-0513 claim 39a (The firer: with an outranking illegal, such as a deactivated firer's, S-0309 claim 29's check drops the member as its target, as before the departure; dropping the target does not itself release the beam) and S-0301 claim 9 (deactivated firers within rank 4); S-0514 claim 14
    // spec: S-0309 r95 claim 27 (a deactivated firer's illegal ranks above the rearming, and with it claim 29's check drops the member)
    let Waiting {
        mut w, f, m, dep, ..
    } = waiting(&rules(), "CART", -3, -2);
    run_through(&mut w, dep);
    assert!(superweapons::warp::departed(&w, m));
    assert_eq!(target(&w, f), Some(Target::Object(m)));
    w.obj_mut(f).unwrap().econ.building.unit_disabled = true;
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, false),
        Answer::Illegal,
        "the deactivated firer's illegal outranks rearming"
    );
    run_through(&mut w, dep + 1);
    assert!(superweapons::warp::departed(&w, m));
    assert_eq!(target(&w, f), None, "dropped at the check tick");
    assert!(temporal::holds(&w, f, m), "the drop releases no beam");
    assert_eq!(
        w.obj(f).unwrap().beh.current,
        Some(Behaviour::Attack),
        "S-0309 claim 29: the drop changes no behaviour"
    );
    // spec: S-0514 claim 14 (in the ordinary stationary attack case, the attack processing with no target falls back to rest and then releases) and S-0513 claim 39a (Release: the beam lets go only at S-0514's causes)
    // The attack processing is made when it is due, so the fall-back can
    // come some turns later; until then the beam stays attached.
    let mut fell_back = false;
    for _ in 0..200 {
        step(&mut w);
        if w.obj(f).unwrap().beh.current != Some(Behaviour::Attack) {
            fell_back = true;
            break;
        }
        assert!(
            temporal::holds(&w, f, m),
            "still attached before the fall-back"
        );
    }
    assert!(
        fell_back,
        "a later attack processing with no target falls back to rest"
    );
    assert!(
        !temporal::holds(&w, f, m),
        "the fall-back to rest, not the drop, lets the beam go"
    );
}

// --- Revision 91: a building firer and a firer beyond range (Q-2376) ---------------------

/// The ruleset with SAMT, a twin of the TOWER with the SAM flag, and STILL,
/// a twin of the CART whose type speed is 0.
fn rules_r91() -> Ruleset {
    let mut r = rules();
    let mut sam = r.sim.buildings["TOWER"].clone();
    sam.sam = Some(true);
    r.sim.buildings.insert("SAMT".into(), sam);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("SAMT".into());
    let mut still = r.sim.vehicles["CART"].clone();
    still.speed = Some(0);
    r.sim.vehicles.insert("STILL".into(), still);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("STILL".into());
    r
}

#[test]
fn r91_claim39a_l7_a_building_firer_keeps_a_member_that_waits_to_land_taking_the_rearming_row() {
    // spec: S-0513 r91 claim 39a (The firer, Q-2376: a building firer without the SAM flag whose beam is attached keeps the member as its target while it waits to land; its attack processing takes S-0215 claim 32's rearming row: no shot, that row's delay, a rotation towards the old position, the direction it fired in, so no facing change; no guard for the departure; the beam still attached at the appearance; SW12 part l7) and S-0215 claim 32 (the rearming row: delay 2, every other tick)
    // spec: S-0309 claim 27 (r91's Changes: a building firer of that kind without the SAM flag keeps the member too, takes S-0215 claim 32's rearming row, makes no shot and rotates towards the old position)
    let processings = assert_kept_while_waiting(waiting(&rules_r91(), "TOWER", -3, -3));
    assert!(processings.len() >= 2, "{processings:?}");
    for (k, d) in &processings {
        assert_eq!(*d, 2, "the rearming row's delay in tick {k}");
    }
    for p in processings.windows(2) {
        assert_eq!(p[1].0 - p[0].0, 2, "processings every other tick");
    }
}

/// A firer `name` of ALPHA at `at(fx, fy)` whose beam is locked on the
/// member BUGGY after the member's turn of t_L + 1, and which is given
/// *attack* on it only at the departure tick (until then it keeps still:
/// no target and its target search put off). An unprotected TOWER of the
/// member's owner on the landing cell redirects the landing, so the member
/// waits to land for more than one tick. `prep` runs on the world just
/// after the firer is placed. Returns the set-up at the departure tick.
fn given_at_departure(
    r: &Ruleset,
    name: &str,
    fx: i32,
    fy: i32,
    prep: impl Fn(&mut World, ObjectId),
) -> Waiting {
    given_at_departure_rem(r, name, fx, fy, 0, prep)
}

/// As [`given_at_departure`] with the launch in a tick whose number leaves
/// `rem` when divided by 16. With `rem` 1 the tick after the departure is
/// a check tick of S-0309 claim 29, in which the member is still off the
/// map (as [`waiting`] gives).
fn given_at_departure_rem(
    r: &Ruleset,
    name: &str,
    fx: i32,
    fy: i32,
    rem: i32,
    prep: impl Fn(&mut World, ObjectId),
) -> Waiting {
    let mut w = ready(r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    spawn(&mut w, "TOWER", BETA, 5, 5);
    let tl = launch(&mut w, PERIOD, rem);
    let f = spawn(&mut w, name, ALPHA, fx, fy);
    prep(&mut w, f);
    step(&mut w);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::holds(&w, f, m), "set-up: locked");
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.combat.opportunity = false;
    o.targeting_timer = t + 400;
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    assert!(temporal::holds(&w, f, m), "set-up: still locked");
    let mut p = w.clone();
    let dep = until(&mut p, 5, |p| superweapons::warp::departed(p, m)) - 1;
    let app = until(&mut p, 30, |p| stage(p, m) == Some(Stage::Placed)) - 1;
    assert_eq!(dep, tl + 62);
    assert!(app > dep + 1, "set-up: it waits to land");
    let t = w.tick;
    assert_eq!(t, dep);
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(m));
    Waiting {
        w,
        f,
        m,
        tl,
        dep,
        app,
    }
}

/// A SAM-flag firer given the member at its departure: the tick after the
/// departure from which it holds no target, if any, up to the appearance.
fn sam_drop_tick(s: &mut Waiting) -> Option<i32> {
    let Waiting {
        w, f, m, dep, app, ..
    } = s;
    while w.tick <= *app {
        let t = w.tick;
        step(w);
        if t > *dep {
            assert!(superweapons::warp::departed(w, *m) || t == *app);
        }
        if w.obj(*f).unwrap().target.is_none() {
            return Some(t);
        }
    }
    None
}

#[test]
fn r91_claim39a_a_sam_building_firer_drops_a_ground_member_and_starts_guard() {
    // spec: S-0513 r91 claim 39a (The firer, Q-2376: a building whose type has the SAM flag takes S-0215 claim 32's SAM rows, as for a target on the map at the member's old position, at the world height it had there; for a member at world height 0 those rows drop the member and then start guard) and S-0215 claim 32 (SAM aiming stage: a target that is not a suitable aircraft is dropped and guard starts)
    let mut s = given_at_departure(&rules_r91(), "SAMT", -3, -3, |_, _| {});
    let dropped = sam_drop_tick(&mut s);
    assert!(dropped.is_some(), "dropped while the member waits to land");
    let o = s.w.obj(s.f).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "guard starts");
}

/// The ruleset of revision 91 with SAMT a type that needs and drains power
/// (S-0405 Terms), so that its owner, with no power plant, is short of
/// power from the start.
fn rules_sam_drain() -> Ruleset {
    let mut r = rules_r91();
    let t = r.sim.buildings.get_mut("SAMT").unwrap();
    t.econ.power = Some(-10);
    t.powered = Some(true);
    r
}

#[test]
fn r108_claim39a_a_sam_building_firer_short_of_power_and_not_charged_drops_the_member_at_the_check()
{
    // spec: S-0513 r108 claim 39a (The firer, Q-2418: while S-0215 claim 32's power row applies, the aiming stage's rows keep any member; a building that is not charged, S-0405 claim 8, is then not in operation, so its check answers S-0405 table T1's impossible, and it drops the member, as a target on the map, at the first of an attack processing in the aimed stage and its checking turn in the first check tick of S-0309 claim 29 in that state) and S-0309 r113 claim 27 (the same for a SAM-flag building)
    // The launch leaves 1 so that the first check tick falls while the
    // member is off the map, after a processing in the aimed stage.
    let mut s = given_at_departure_rem(&rules_sam_drain(), "SAMT", -3, -3, 1, |_, _| {});
    assert!(
        glasnost_sim::seams::sam_held_by_low_power(&s.w, s.f),
        "set-up: the power row applies"
    );
    assert_eq!(
        glasnost_sim::econ::power::chargers(&s.w, s.f),
        0,
        "set-up: not charged"
    );
    let first_check = (s.dep..).find(|&t| held_check(t)).unwrap();
    assert_eq!(
        first_check,
        s.dep + 1,
        "set-up: a processing before the check"
    );
    assert!(first_check < s.app, "set-up: the check while off the map");
    let dropped = sam_drop_tick(&mut s);
    assert_eq!(
        dropped,
        Some(first_check),
        "kept by its own rows until the check"
    );
    assert_eq!(
        check(&s.w, s.f, Some(Target::Object(s.m)), 0, true),
        Answer::Impossible,
        "the building's own impossible"
    );
    let o = s.w.obj(s.f).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Attack),
        "the check starts no guard"
    );
    assert!(
        temporal::holds(&s.w, s.f, s.m),
        "S-0514 claim 14: the drop releases no beam"
    );
}

#[test]
fn r108_claim39a_a_charged_sam_building_firer_short_of_power_keeps_the_member() {
    // spec: S-0513 r108 claim 39a (The firer, Q-2418: a charged building's check answers rearming, for which the aimed row for any other answer does nothing, so it keeps the member) and S-0405 claim 8 and Constants (2 chargers keep a building in operation)
    let mut s = given_at_departure_rem(&rules_sam_drain(), "SAMT", -3, -3, 1, |w, f| {
        let a = spawn(w, "CART", ALPHA, -7, 7);
        let b = spawn(w, "CART", ALPHA, -7, 8);
        w.obj_mut(f).unwrap().combat.chargers = vec![a, b];
    });
    assert!(
        glasnost_sim::seams::sam_held_by_low_power(&s.w, s.f),
        "set-up: the power row applies"
    );
    assert_eq!(
        glasnost_sim::econ::power::chargers(&s.w, s.f),
        2,
        "set-up: charged"
    );
    assert!(glasnost_sim::econ::power::in_operation(&s.w, s.f));
    // The first check tick falls while the member is off the map, as in the
    // uncharged run, whose drop comes at it.
    let first_check = (s.dep..).find(|&t| held_check(t)).unwrap();
    assert_eq!(
        first_check,
        s.dep + 1,
        "set-up: a check after the departure"
    );
    assert!(first_check < s.app, "set-up: the check while off the map");
    assert_eq!(sam_drop_tick(&mut s), None, "kept through the check");
    assert_eq!(
        s.w.obj(s.f).unwrap().target,
        Some(Target::Object(s.m)),
        "still held at the appearance"
    );
}

/// The distance in leptons between two points on the ground.
fn flat(a: Pos, b: Pos) -> i64 {
    let (x, y) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    (x * x + y * y).isqrt()
}

#[test]
fn r91_claim39a_a_vehicle_firer_beyond_range_keeps_the_member_approaches_its_old_position_and_lets_go_at_a_new_cell()
 {
    // spec: S-0513 r91 claim 39a (The firer, Q-2376: an infantry or vehicle firer whose beam is attached and that is beyond its weapon's range of the old position at the departure keeps the member as its target; its firing check answers rearming whatever the range, so no shot; it takes S-0215 claim 37's approach step as for a target on the map at the old position, so a firer in attack that moves goes towards the old position; it lets the beam go at its first arrival in a new cell, S-0514 claim 14)
    // spec: S-0309 claim 27 (r91's Changes: a firer of that kind beyond range of the old position keeps the member and takes S-0215 claim 37's approach step towards it, letting the beam go at its first new cell)
    let Waiting {
        mut w,
        f,
        m,
        tl,
        dep,
        ..
    } = given_at_departure(&rules_r91(), "CART", -8, -8, |_, _| {});
    let old = w.obj(m).unwrap().pos;
    let start = w.obj(f).unwrap().pos;
    let cell = start.cell();
    assert!(
        !glasnost_sim::approach::in_range(&w, f, Target::Object(m)),
        "set-up: beyond range"
    );
    let mut moved_at = None;
    for _ in 0..400 {
        let t = w.tick;
        w.effects.clear();
        step(&mut w);
        assert_eq!(shots_by(&w, f), 0, "no shot in t_L + {}", t - tl);
        let o = w.obj(f).unwrap();
        if o.pos.cell() != cell {
            moved_at = Some(t);
            break;
        }
        assert!(temporal::holds(&w, f, m), "attached before a new cell");
        if superweapons::warp::departed(&w, m) {
            assert_eq!(o.target, Some(Target::Object(m)), "kept off the map");
            assert_eq!(
                check(&w, f, Some(Target::Object(m)), 0, true),
                Answer::Rearming,
                "rearming whatever the range"
            );
        }
    }
    let t = moved_at.expect("the firer moves");
    assert!(t > dep, "it moves only after it is given the member");
    let now = w.obj(f).unwrap().pos;
    assert!(
        flat(now, old) < flat(start, old),
        "it moved towards the old position"
    );
    // The arrival is seen at the holder's next turn (`temporal::holder_turn`,
    // before its movement), with the firer still in the new cell.
    let new = now.cell();
    step(&mut w);
    assert_eq!(w.obj(f).unwrap().pos.cell(), new);
    assert!(
        !temporal::holds(&w, f, m),
        "the beam lets go at the first new cell"
    );
}

#[test]
fn r108_claim39a_a_deployed_infantry_firer_beyond_range_drops_the_member_and_stays_put() {
    // spec: S-0513 r115 claim 39a (The firer: in guard, S-0215 claim 31, its beam stays attached; in attack the fall-back to rest after a drop with no target found lets it go, Q-2468)
    // spec: S-0513 r108 claim 39a (The firer, Q-2418: a deployed infantry unit of a human-controlled owner takes no approach step, S-0215 claim 37 g, and neither moves nor turns; at each processing at which S-0215 claim 23 or 31 runs S-0517 claim 16's deployed-firing rule, that rule applies to the member at its old position, as to a target on the map there: beyond the range of the weapon that rule selects it drops the member, or takes the target that rule's search finds instead; its beam stays attached)
    // spec: S-0309 r113 claim 27 (a deployed infantry firer of a human-controlled owner neither moves nor turns, and S-0517 claim 16's deployed-firing rule applies to the member at the old position, while its beam stays attached)
    let mut r = rules_r91();
    // The WALKER's Rifle (warhead SA) is a temporal weapon here, and the
    // WALKER a deploy-to-fire type (SW12 part l9) whose deploy weapon is the
    // Rifle too (S-0308 claim 8: no deploy weapon index given, slot 0).
    r.sim.warheads.get_mut("SA").unwrap().temporal = Some(true);
    r.sim.infantry.get_mut("WALKER").unwrap().deploy_fire = Some(true);
    assert!(
        r.globals.quirks.warp_departure_keeps_beam,
        "the default switch"
    );
    // In guard (S-0215 claim 31 runs the rule) and in attack (claim 23).
    for beh in [Behaviour::Guard, Behaviour::Attack] {
        let s = given_at_departure(&r, "WALKER", -6, -6, |w, f| {
            w.obj_mut(f).unwrap().infantry.as_mut().unwrap().seq =
                Some(glasnost_sim::infantry::Seq::Deployed);
        });
        let Waiting {
            mut w,
            f,
            m,
            tl,
            dep,
            app,
        } = s;
        let t = w.tick;
        let o = w.obj_mut(f).unwrap();
        o.beh.force(beh, t, false);
        o.target = Some(Target::Object(m));
        assert!(
            w.houses[usize::from(ALPHA.0)].human,
            "set-up: a human owner"
        );
        assert!(
            !glasnost_sim::approach::in_range(&w, f, Target::Object(m)),
            "set-up: beyond range"
        );
        let before = firer_state(&w, f);
        let mut dropped = None;
        for t in dep..=app {
            step(&mut w);
            let o = w.obj(f).unwrap();
            assert_eq!(o.dest, None, "{beh:?}: no approach in t_L + {}", t - tl);
            let now = firer_state(&w, f);
            assert_eq!(
                (&now.2, &now.3),
                (&before.2, &before.3),
                "{beh:?}: neither moves nor turns in t_L + {}",
                t - tl
            );
            if dropped.is_none() && o.target.is_none() {
                dropped = Some(t);
            }
        }
        // spec: S-0517 claim 16 (beyond the selected weapon's range, with no other target found, the target is dropped)
        if beh == Behaviour::Attack {
            assert_eq!(
                dropped,
                Some(dep),
                "attack: dropped in the first processing"
            );
        } else {
            // The forced guard's first processing comes later.
            assert!(dropped.is_some(), "guard: dropped while it waits to land");
        }
        if beh == Behaviour::Guard {
            assert!(
                temporal::holds(&w, f, m),
                "guard: its beam stays attached (no fall-back, S-0517 claim 16)"
            );
        } else {
            // spec: S-0513 r115 claim 39a (The firer, Q-2468: in attack, S-0215 claim 23, a drop with no target found makes a unit whose type is not a jumpjet type fall back to rest, S-0517 claim 16, and that fall-back lets the beam go as one of its common effects, S-0215 claim 13, S-0514 claim 14)
            // spec: S-0309 r116 claim 27 (in attack the fall-back to rest that follows a drop with no target found lets the beam go)
            assert!(
                !temporal::holds(&w, f, m),
                "attack: the fall-back lets the beam go"
            );
            assert_eq!(w.obj(f).unwrap().beh.current, Some(Behaviour::Guard));
        }
    }
}

#[test]
fn r115_claim39a_a_deployed_jumpjet_firer_and_a_second_fall_back_of_the_tick_keep_the_beam_in_attack()
 {
    // spec: S-0513 r115 claim 39a (The firer, Q-2468: a unit of a jumpjet type does not fall back there: it stays in attack with no target, its beam still attached; the fall-back lets the beam go unless a fall-back to rest has already taken those effects in that tick)
    // spec: S-0309 r116 claim 27 (in attack the fall-back lets the beam go, unless its type is a jumpjet type, which does not fall back there, or a fall-back to rest has already taken its common effects in that tick)
    // spec: S-0517 claim 16 (a unit left targetless falls back to rest unless its current behaviour is guard or its type is a jumpjet type) and S-0215 claim 13 (the common effects at most once per actor per tick)
    for (jumpjet, earlier) in [(true, false), (false, true), (false, false)] {
        let mut r = rules_r91();
        r.sim.warheads.get_mut("SA").unwrap().temporal = Some(true);
        let walker = r.sim.infantry.get_mut("WALKER").unwrap();
        walker.deploy_fire = Some(true);
        walker.jumpjet = Some(jumpjet);
        let s = given_at_departure(&r, "WALKER", -6, -6, |w, f| {
            w.obj_mut(f).unwrap().infantry.as_mut().unwrap().seq =
                Some(glasnost_sim::infantry::Seq::Deployed);
        });
        let Waiting { mut w, f, m, .. } = s;
        let t = w.tick;
        let o = w.obj_mut(f).unwrap();
        o.beh.force(Behaviour::Attack, t, false);
        o.target = Some(Target::Object(m));
        if earlier {
            // A fall-back to rest has already taken the common effects in
            // this tick.
            o.beh.last_common_effects = Some(t);
        }
        step(&mut w);
        let o = w.obj(f).unwrap();
        let case = format!("jumpjet {jumpjet}, earlier fall-back {earlier}");
        assert_eq!(o.target, None, "{case}: the member dropped");
        assert_eq!(
            temporal::holds(&w, f, m),
            jumpjet || earlier,
            "{case}: the beam"
        );
        if jumpjet {
            assert_eq!(
                o.beh.current,
                Some(Behaviour::Attack),
                "{case}: stays in attack with no target"
            );
        }
    }
}

/// One tick of SW12 part l9's record of *T*: target, current and queued
/// behaviour, sequence, cell, body facing and whether its beam is attached.
#[derive(Clone, Debug)]
struct L9 {
    tick: i32,
    target: Option<Target>,
    current: Option<Behaviour>,
    queued: Option<Behaviour>,
    seq: Option<glasnost_sim::infantry::Seq>,
    cell: glasnost_map::CellXY,
    facing: glasnost_num::geom::Facing,
    beam: bool,
}

fn l9_row(w: &World, f: ObjectId, m: ObjectId) -> L9 {
    let o = w.obj(f).unwrap();
    L9 {
        tick: w.tick,
        target: o.target,
        current: o.beh.current,
        queued: o.beh.queued,
        seq: o.infantry.as_ref().and_then(|s| s.seq),
        cell: o.pos.cell(),
        facing: o.body.value(w.tick),
        beam: temporal::holds(w, f, m),
    }
}

fn deploy_complete(seq: Option<glasnost_sim::infantry::Seq>) -> bool {
    use glasnost_sim::infantry::Seq;
    matches!(
        seq,
        Some(Seq::Deployed | Seq::DeployedFire | Seq::DeployedIdle)
    )
}

#[test]
fn r115_claim39a_sw12_l9a_and_l9_a_deploy_order_then_an_attack_order_on_the_member_beyond_the_deploy_weapons_range()
 {
    // spec: S-0513 r115 claim 39a (The firer, Q-2468: a deploy order clears the unit's target and queues unload, S-0213 claim 10, which becomes current at the first update of the unit after the order at which it is ready to change, S-0215 claim 11; until then it stays in attack with no target; otherwise the deploy goes on, and unload forces guard, S-0215 claim 30, so a unit whose deploy completes holds no target in guard, its beam still attached; in attack, S-0215 claim 23, a drop with no target found makes a unit whose type is not a jumpjet type fall back to rest, S-0517 claim 16, and that fall-back lets the beam go as one of its common effects, S-0215 claim 13, S-0514 claim 14) and SW12 parts l9a and l9
    // spec: S-0309 r116 claim 27 (a deployed infantry firer of a human-controlled owner neither moves nor turns; its beam stays attached in guard, and in attack the fall-back to rest that follows a drop with no target found lets it go)
    let mut r = rules();
    r.sim.warheads.get_mut("SA").unwrap().temporal = Some(true);
    // SW12 l9's deploy weapon (S-0308 claim 8, the secondary slot by the
    // import default), without the area-fire flag, with a range shorter
    // than T's distance to the tank; the undeployed weapon is the Rifle,
    // whose warhead SA is temporal here (part l's temporal weapon).
    r.sim.weapons.insert(
        "Stub".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(1),
            rof: Some(25),
            range: Some(glasnost_rules::sim::Decimal("1".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.deployer = Some(true);
    walker.deploy_fire = Some(true);
    walker.secondary = Some("Stub".into());
    // spec: S-0513 r115 SW12 part l9 (Setup: T's type has a negative undeploy delay)
    walker.undeploy_delay = Some(-1);
    // Invented deploy sequences (as the S-0517 claim tests' deployed
    // infantry type has them), so its deploy can start and complete.
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 6),
        ("ready", 1),
        ("stand", 1),
        ("deployed_fire", 2),
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
    let mut w = ready(&r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w, PERIOD, 0);
    let f = spawn(&mut w, "WALKER", ALPHA, -3, 0);
    step(&mut w);
    // The lock in tick d (claim 39a, part l): T's beam attached, T in
    // attack on the tank.
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::holds(&w, f, m), "set-up: locked");
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(m));
    o.combat.opportunity = false;
    o.targeting_timer = t + 400;
    step(&mut w);
    assert!(
        w.houses[usize::from(ALPHA.0)].human,
        "set-up: a human owner"
    );
    let before = l9_row(&w, f, m);
    assert_eq!(before.current, Some(Behaviour::Attack), "set-up: in attack");
    assert!(before.beam, "set-up: its beam attached before the order");
    let dep = tl + 62;
    // The deploy order (tick d + 1).
    assert!(
        orders::issue(&mut w, ALPHA, orders::Order::Deploy { actor: f }),
        "the deploy order is taken"
    );
    let mut rec = Vec::new();
    while w.tick < dep && !deploy_complete(rec.last().and_then(|x: &L9| x.seq)) {
        step(&mut w);
        rec.push(l9_row(&w, f, m));
    }
    // (l9a) unload current no later than T's second update after the order
    // (S-0215 claim 30: its processing forces guard at once), and no
    // attack processing before it: no target held and no beam let go.
    let unload_at = rec
        .iter()
        .position(|x| x.current == Some(Behaviour::Guard) && x.seq.is_some_and(|q| q.is_deploy()))
        .unwrap_or_else(|| panic!("l9a: unload made current and processed: {rec:#?}"));
    assert!(unload_at <= 1, "l9a: unload by the second update: {rec:#?}");
    for x in &rec[..=unload_at] {
        assert_eq!(x.target, None, "l9a: no target from the order on: {x:?}");
        assert!(x.beam, "l9a: the beam stays attached: {x:?}");
    }
    // Once unload is current the deploy starts and completes before the
    // departure update.
    let last = rec.last().unwrap();
    assert!(
        deploy_complete(last.seq),
        "l9a: the deploy completes by tick {}: {rec:#?}",
        last.tick
    );
    assert!(
        last.tick < dep,
        "set-up: the completion comes before the departure"
    );
    // (l9) From its unload processing on T is in guard; from the completion
    // to the attack order it holds no target, its beam attached.
    for x in &rec[unload_at..] {
        assert_eq!(x.current, Some(Behaviour::Guard), "l9: guard: {x:?}");
        assert_eq!(x.target, None, "l9: no target: {x:?}");
        assert!(x.beam, "l9: the beam stays attached: {x:?}");
        assert_eq!(
            (x.cell, x.facing),
            (before.cell, before.facing),
            "no move or turn"
        );
    }
    let done = last.clone();
    for _ in 0..3 {
        step(&mut w);
        let x = l9_row(&w, f, m);
        assert_eq!(x.current, Some(Behaviour::Guard), "l9: guard: {x:?}");
        assert!(x.beam && x.target.is_none(), "l9: {x:?}");
    }
    // The attack order, with force-fire held as in (l1).
    assert!(
        orders::issue(
            &mut w,
            ALPHA,
            orders::Order::Behaviour {
                actor: f,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(m)),
                dest: None,
            },
        ),
        "the attack order is taken"
    );
    let mut held = false;
    let mut after = None;
    while w.tick < dep {
        step(&mut w);
        let x = l9_row(&w, f, m);
        assert_eq!(
            (x.cell, x.facing),
            (done.cell, done.facing),
            "l9: no move or turn: {x:?}"
        );
        if x.target == Some(Target::Object(m)) {
            held = true;
        } else if held {
            after = Some(x);
            break;
        }
    }
    assert!(held, "set-up: the attack order gave T the tank");
    let x =
        after.expect("l9: an attack processing ran S-0517 claim 16's rule before the departure");
    assert_eq!(x.target, None, "l9: no target just after it");
    assert!(!x.beam, "l9: the fall-back to rest let the beam go");
    assert!(
        x.current == Some(Behaviour::Guard) || x.queued == Some(Behaviour::Guard),
        "l9: guard current or queued: {x:?}"
    );
}

#[test]
fn r91_claim39a_a_zero_speed_firer_beyond_range_keeps_the_member_and_its_beam_while_it_does_not_move()
 {
    // spec: S-0513 r91 claim 39a (The firer, Q-2376: a firer whose type speed is 0 takes the approach step, but the step alone supplies no movement, S-0215 claim 37 g, so it keeps the member and its beam while it does not move)
    let Waiting {
        mut w,
        f,
        m,
        tl,
        dep,
        app,
    } = given_at_departure(&rules_r91(), "STILL", -8, -8, |_, _| {});
    let cell = w.obj(f).unwrap().pos.cell();
    for t in dep..=app {
        step(&mut w);
        let o = w.obj(f).unwrap();
        assert_eq!(o.pos.cell(), cell, "it does not move");
        assert_eq!(o.target, Some(Target::Object(m)), "t_L + {}", t - tl);
        assert!(temporal::holds(&w, f, m), "attached in t_L + {}", t - tl);
    }
}

// --- Revision 91: a member removed by its erasure update in a sparkle tick (Q-2377) -------

/// A member of `name` being erased in its leaving stage, removed by the
/// erasure update of its turn in a tick divisible by 24 (its amount set to
/// 1 just before that tick). Returns the sparkle (SPARK) and warp-away
/// (GONE) animations that tick made, in order, with their positions, and
/// the member's last position.
fn removed_in_a_sparkle_tick(name: &str) -> (Vec<(String, Pos)>, Pos) {
    let mut r = rules();
    r.sim.general.warp_away = Some("GONE".into());
    r.sim
        .animations
        .insert("GONE".into(), glasnost_rules::sim::AnimationType::default());
    r.registries
        .entry("animations".into())
        .or_default()
        .push("GONE".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(100_000);
    r.sim.infantry.get_mut("JUMPER").unwrap().strength = Some(100_000);
    let mut w = ready(&r);
    let f = spawn(&mut w, "CART", ALPHA, -3, -2);
    let m = spawn(&mut w, name, BETA, 0, 0);
    // t_L + 30 is a sparkle tick, inside the leaving stage.
    let tl = launch(&mut w, SPARKLE, (2 * SPARKLE - 30) % SPARKLE);
    step(&mut w);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m), "{name}: set-up: being erased");
    run_through(&mut w, tl + 29);
    assert_eq!(w.tick, tl + 30);
    assert_eq!(w.tick.rem_euclid(SPARKLE), 0);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))), "{name}");
    let last = w.obj(m).unwrap().pos;
    w.temporal.locks.get_mut(&m).unwrap().amount = 1;
    w.effects.clear();
    step(&mut w);
    assert!(w.obj(m).is_none(), "{name}: removed in that tick");
    let made = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } if name == "SPARK" || name == "GONE" => {
                Some((name.clone(), *pos))
            }
            _ => None,
        })
        .collect();
    (made, last)
}

#[test]
fn r91_claim39a_a_member_removed_by_its_erasure_update_in_a_sparkle_tick_sparkles_in_its_kinds_order()
 {
    // spec: S-0513 r91 claim 39a (Sparkles and drops in the warp's turns, Q-2377: a turn whose erasure update removes the member sparkles too, in a sparkle tick, when the member is being erased or phasing in at the turn's start; an infantry member makes the sparkle before the turn's erasure update, a vehicle or aircraft member after it, both before the warp step) and S-0514 r16 claim 18 (placed 120 leptons farther along each horizontal axis from the last position) and claim 12 (the warp-away animation at completion)
    let (got, last) = removed_in_a_sparkle_tick("JUMPER");
    assert_eq!(
        got,
        vec![
            ("SPARK".to_string(), placed(last)),
            ("GONE".to_string(), last)
        ],
        "infantry: the sparkle before the update"
    );
    let (got, last) = removed_in_a_sparkle_tick("BUGGY");
    assert_eq!(
        got,
        vec![
            ("GONE".to_string(), last),
            ("SPARK".to_string(), placed(last))
        ],
        "vehicle: the sparkle after the update"
    );
}

// --- Revision 108: a held temporal target within rank 4 (Q-2416) -------------------------

/// A CART of ALPHA whose temporal beam is locked on a BUGGY of BETA on the
/// map, holding it as its target, with no warp.
fn beamed_on_the_map() -> (World, ObjectId, ObjectId) {
    let mut w = ready(&rules());
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let f = spawn(&mut w, "CART", ALPHA, -3, -2);
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::holds(&w, f, m), "set-up: locked");
    w.obj_mut(f).unwrap().target = Some(Target::Object(m));
    (w, f, m)
}

/// Makes `m` protected (S-0513 claim 27) from now on.
fn protect_now(w: &mut World, m: ObjectId) {
    let t = w.tick;
    w.obj_mut(m).unwrap().sw.protection = Some(superweapons::Protection {
        start: t,
        duration: 10_000,
        kind: superweapons::ProtectKind::Curtain,
        tint: Default::default(),
    });
}

/// Makes `m` fully cloaked, with no sensor of the firer's house, so that
/// it is hidden from that house (S-0516 claim 24).
fn hide(w: &mut World, m: ObjectId) {
    w.obj_mut(m).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
}

/// Makes ALPHA a computer-controlled house, whose firing check reads
/// protection (S-0513 claim 28).
fn computer_alpha(w: &mut World) {
    w.houses[usize::from(ALPHA.0)].human = false;
}

#[test]
fn r108_claim39a_the_held_temporal_rearming_outranks_a_protected_targets_illegal_on_the_map() {
    // spec: S-0513 r108 claim 39a and its Changes for S-0301 claim 9 (A held temporal target within rank 4, Q-2416: the held-temporal-target rearming outranks the illegal for a protected target, S-0513 claim 28, so a firer whose temporal beam is attached to a protected target keeps it at S-0309 claim 29's check, on the map); S-0513 claim 28 (a firer that is not a player-human gets illegal against a protected object)
    // spec: S-0301 r70 claim 9 (A held temporal target within rank 4: the rearming outranks the illegal for a protected target)
    let (mut w, f, m) = beamed_on_the_map();
    computer_alpha(&mut w);
    protect_now(&mut w, m);
    assert!(superweapons::protected(&w, m));
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, false),
        Answer::Rearming,
        "the beam's rearming"
    );
    // The control: another firer of the same house with no beam attached.
    let g = spawn(&mut w, "CART", ALPHA, -3, 2);
    assert_eq!(
        check(&w, g, Some(Target::Object(m)), 0, false),
        Answer::Illegal,
        "with no beam: the protected target's illegal"
    );
    // S-0309 claim 29's check keeps it.
    let next = (w.tick..).find(|&t| held_check(t)).unwrap();
    run_through(&mut w, next);
    assert_eq!(target(&w, f), Some(Target::Object(m)), "kept at the check");
}

#[test]
fn r108_claim39a_the_held_temporal_rearming_outranks_a_hidden_targets_impossible_on_the_map() {
    // spec: S-0513 r108 claim 39a and its Changes for S-0301 claim 9 (A held temporal target within rank 4, Q-2416: the held-temporal-target rearming outranks the hidden-target impossible, S-0516 claim 24, so a firer whose temporal beam is attached to a hidden target keeps it at S-0309 claim 29's check, on the map)
    // spec: S-0301 r70 claim 9 (A held temporal target within rank 4: the rearming outranks the hidden-target impossible)
    let (mut w, f, m) = beamed_on_the_map();
    hide(&mut w, m);
    let owner = w.obj(f).unwrap().owner;
    assert!(glasnost_sim::cloak::hidden_from(&w, m, owner));
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, false),
        Answer::Rearming,
        "the beam's rearming"
    );
    let g = spawn(&mut w, "CART", ALPHA, -3, 2);
    assert_eq!(
        check(&w, g, Some(Target::Object(m)), 0, false),
        Answer::Impossible,
        "with no beam: the hidden target's impossible"
    );
}

#[test]
fn r108_claim39a_a_firer_keeps_a_protected_or_hidden_member_off_the_map() {
    // spec: S-0513 r108 claim 39a (The firer, Q-2416: neither the member's protection, claims 26 and 28, nor its being hidden from the firer's house, S-0516 claim 24, gives such an answer: rearming outranks both, S-0301 claim 9 as amended, on the map or off it, so the firer keeps a protected or hidden member at S-0309 claim 29's checks)
    // spec: S-0301 r70 claim 9 (A held temporal target within rank 4: the keep holds on the map or, for a superweapon warp member, off it)
    for hidden in [false, true] {
        let Waiting {
            mut w, f, m, dep, ..
        } = waiting(&rules(), "CART", -3, -2);
        run_through(&mut w, dep);
        assert!(superweapons::warp::departed(&w, m));
        if hidden {
            hide(&mut w, m);
            let owner = w.obj(f).unwrap().owner;
            assert!(glasnost_sim::cloak::hidden_from(&w, m, owner));
        } else {
            computer_alpha(&mut w);
            protect_now(&mut w, m);
        }
        assert_eq!(
            check(&w, f, Some(Target::Object(m)), 0, false),
            Answer::Rearming,
            "hidden {hidden}: rearming off the map"
        );
        // The tick after the departure is a check tick (`waiting`).
        run_through(&mut w, dep + 1);
        assert!(superweapons::warp::departed(&w, m));
        assert_eq!(
            target(&w, f),
            Some(Target::Object(m)),
            "hidden {hidden}: kept at the check off the map"
        );
    }
}

// --- Revision 108: a building firer before the lock (SW12 parts l7a and l7b) -------------

/// The rules of part l7: the TOWER's weapon a temporal weapon (warhead HE)
/// with a rearm delay of 20 ticks and a straight projectile (no steering,
/// not arcing), whose shot speed is the arc speed for the distance under
/// the test data's gravity `g` (S-0306 claim 1, F2), capped at half the
/// distance to the impact point (claim 2), so that the harness can place
/// the TOWER for a detonation in the second tick after the shot.
fn rules_l7(g: i32, keeps: bool) -> Ruleset {
    let mut r = keeps_beam(keeps);
    r.sim.general.gravity = Some(g);
    r.sim.projectiles.insert(
        "Straight".into(),
        glasnost_rules::sim::ProjectileType::default(),
    );
    r.sim.weapons.insert(
        "Timer".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(1),
            // spec: S-0513 SW12 (l7) (the test data give the weapon a rearm delay of 20 ticks)
            rof: Some(20),
            range: Some(glasnost_rules::sim::Decimal("7.25".into())),
            projectile: Some("Straight".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    r.sim.buildings.get_mut("TOWER").unwrap().primary = Some("Timer".into());
    r
}

/// Part l7's set-up: the tank (a BUGGY of BETA) in the block at (0, 0) and
/// a TOWER *T* of ALPHA ordered by the human to attack it without
/// force-fire, with ALPHA's teleport source picked on the block. Gives the
/// world before the tick of *T*'s first shot, *T*, the tank and that tick
/// *s*, found by a rehearsal on a copy; `None` when the detonation is not
/// in *s* + 2 or a check tick of S-0309 claim 29 falls from *s* to *s* + 3
/// (such a run decides nothing).
fn l7_setup(r: &Ruleset, fx: i32, fy: i32, wait: i32) -> Option<(World, ObjectId, ObjectId, i32)> {
    let mut w = ready(r);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    // As in `waiting_at`: the landing is redirected, so the tank waits to
    // land for more than one tick.
    spawn(&mut w, "TOWER", BETA, 5, 5);
    let f = spawn(&mut w, "TOWER", ALPHA, fx, fy);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    while orders::blocked(&w) {
        step(&mut w);
    }
    for _ in 0..wait {
        step(&mut w);
    }
    let ok = orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: f,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(m)),
            dest: None,
        },
    );
    assert!(ok, "the attack order is taken");
    let mut p = w.clone();
    let mut s = None;
    for _ in 0..200 {
        let t = p.tick;
        p.effects.clear();
        step(&mut p);
        if shots_by(&p, f) > 0 {
            s = Some(t);
            break;
        }
    }
    let s = s?;
    let mut d = None;
    for _ in 0..20 {
        let t = p.tick;
        step(&mut p);
        if temporal::holds(&p, f, m) {
            d = Some(t);
            break;
        }
    }
    if d != Some(s + 2) || (s..=s + 3).any(held_check) {
        return None;
    }
    // Back to the world before tick s.
    while w.tick < s {
        step(&mut w);
    }
    Some((w, f, m, s))
}

/// Finds a placement, projectile speed and start for part l7.
fn l7(keeps: bool) -> (World, ObjectId, ObjectId, i32) {
    for g in [120, 200, 400, 800] {
        let r = rules_l7(g, keeps);
        for (fx, fy) in [(-2, 0), (-3, 0), (-2, -2), (-3, -3), (-4, 0), (-4, -4)] {
            for wait in 0..16 {
                if let Some(found) = l7_setup(&r, fx, fy, wait) {
                    return found;
                }
            }
        }
    }
    panic!("no l7 placement detonates in the second tick after the shot");
}

/// Runs tick `s` with the warp order applied after its turns (part l's
/// launch), returning t_L.
fn launch_in(w: &mut World, s: i32) -> i32 {
    while w.tick < s {
        step(w);
    }
    fire(w, ALPHA, WARP, at(5, 5));
    let tl = w.tick;
    step(w);
    tl
}

#[test]
fn r108_claim39a_sw12_l7a_a_building_drops_the_member_at_its_processing_before_the_lock() {
    // spec: S-0513 r108 claim 39a (The firer: until the lock a building's firing check answers claim 39's illegal, unless an answer that S-0301 claim 9 ranks above it holds; so it drops the member, after the launch and before the lock, at the first of an attack processing at which its check answers that illegal, where S-0215 claim 32's row drops it and starts guard, and its checking turn in a check tick of S-0309 claim 29; a building whose shot is made at an attack processing sets its timer to 1 tick there, so its next processing is in the next tick) and SW12 part (l7a) (the launch after the turns of tick s; no beam attached just before T's processing in s + 1, and just after it T holds no target and is in guard); S-0309 r113 claim 27 (one drops the member, after the launch and before the lock, at the first of two places)
    // spec: S-0513 SW12 (l7a is as in the original with quirks.warp_departure_keeps_beam off)
    for keeps in [true, false] {
        let (mut w, f, m, s) = l7(keeps);
        let tl = launch_in(&mut w, s);
        assert_eq!(tl, s, "the launch after the turns of tick s");
        assert!(
            superweapons::warp_pending_or_leaving(&w, m),
            "set-up: the warp is launched"
        );
        assert_eq!(w.tick, s + 1);
        assert!(!temporal::holds(&w, f, m), "no beam just before s + 1");
        assert_eq!(target(&w, f), Some(Target::Object(m)));
        step(&mut w);
        let o = w.obj(f).unwrap();
        assert_eq!(
            o.beh.last.map(|(k, _)| k),
            Some(s + 1),
            "keeps {keeps}: T processes in s + 1"
        );
        assert_eq!(o.target, None, "keeps {keeps}: no target after s + 1");
        assert_eq!(
            o.beh.current,
            Some(Behaviour::Guard),
            "keeps {keeps}: in guard"
        );
    }
}

#[test]
fn r108_claim39a_a_building_that_dropped_the_member_holds_it_again_after_the_lock_as_at_the_lock() {
    // spec: S-0513 r108 claim 39a (The firer: after the drop the building takes targets as any building does, the member among them; its beam stays attached either way; its firing check about the member answers claim 39's illegal before the lock and S-0514 claim 19's rearming from the lock on; a building that holds the member again after the lock holds it as one that held it at the lock does, unless a revert gave it back while the member is off the map); S-0309 r113 claim 27 (one that holds the member again after the lock keeps it as one that held it at the lock does)
    let (mut w, f, m, s) = l7(true);
    launch_in(&mut w, s);
    step(&mut w);
    assert_eq!(target(&w, f), None, "set-up: dropped in s + 1");
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, true),
        Answer::Illegal,
        "before the lock: claim 39's illegal"
    );
    step(&mut w);
    assert!(temporal::holds(&w, f, m), "the lock in s + 2");
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, true),
        Answer::Rearming,
        "from the lock on: S-0514 claim 19's rearming"
    );
    // It takes the member again, as an order or a search gives it.
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(m));
    let mut p = w.clone();
    let app = until(&mut p, 120, |p| stage(p, m) == Some(Stage::Placed)) - 1;
    while w.tick <= app {
        let t = w.tick;
        w.effects.clear();
        step(&mut w);
        assert_eq!(shots_by(&w, f), 0, "no shot in {t}");
        assert_eq!(target(&w, f), Some(Target::Object(m)), "kept in {t}");
        assert_eq!(w.obj(f).unwrap().beh.current, Some(Behaviour::Attack));
    }
    assert!(temporal::holds(&w, f, m), "attached after the appearance");
}

#[test]
fn r108_claim39a_sw12_l7b_a_building_that_holds_the_member_at_the_lock_keeps_it() {
    // spec: S-0513 r108 claim 39a (The firer: a building firer without the SAM flag whose beam is attached to the member keeps it as its target while it waits to land if it still holds it at the lock; its attack processing takes S-0215 claim 32's rearming row: no shot, that row's delay, no guard for the departure) and SW12 part (l7b) (the launch one tick later, after the turns of s + 1; T's processing in s + 1 answers rearming, so it makes no second shot and processes next in s + 3; the lock in s + 2 comes before it; at each of T's processings from s + 3 to the departure update T holds the tank and stays in attack; from the departure update to the appearance T holds it, stays in attack, makes no shot, keeps its facing, processes every other tick, and its beam is attached after the appearance); S-0309 r113 claim 27 (a building firer without the SAM flag that holds the member at the lock keeps it too)
    let (mut w, f, m, s) = l7(true);
    let tl = launch_in(&mut w, s + 1);
    assert_eq!(tl, s + 1);
    let last = w.obj(f).unwrap().beh.last;
    assert_eq!(last.map(|(k, _)| k), Some(s + 1), "T processed in s + 1");
    assert_eq!(target(&w, f), Some(Target::Object(m)), "kept in s + 1");
    step(&mut w);
    assert!(temporal::holds(&w, f, m), "the lock in s + 2");
    let mut p = w.clone();
    let dep = until(&mut p, 80, |p| superweapons::warp::departed(p, m)) - 1;
    let app = until(&mut p, 30, |p| stage(p, m) == Some(Stage::Placed)) - 1;
    let mut processings = Vec::new();
    while w.tick < dep {
        let t = w.tick;
        w.effects.clear();
        step(&mut w);
        assert_eq!(shots_by(&w, f), 0, "no shot in {t}");
        let o = w.obj(f).unwrap();
        if o.beh.last.is_some_and(|(k, _)| k == t) {
            processings.push(t);
            assert_eq!(o.target, Some(Target::Object(m)), "held after {t}");
            assert_eq!(o.beh.current, Some(Behaviour::Attack), "attack after {t}");
        }
    }
    assert_eq!(processings.first(), Some(&(s + 3)), "next processing s + 3");
    let s = Waiting {
        w,
        f,
        m,
        tl,
        dep,
        app,
    };
    let during = assert_kept_while_waiting(s);
    for p in during.windows(2) {
        assert_eq!(p[1].0 - p[0].0, 2, "processings every other tick");
    }
}

#[test]
fn r108_claim39a_a_building_with_a_delayed_shot_pending_answers_its_rearming_for_the_member() {
    // spec: S-0513 r108 claim 39a (The firer: while a delayed shot of the building is pending, its firing check about the member answers as S-0301 claims 9 and 14 give: the pending shot's rearming, whose S-0215 claim 32 row keeps the member, unless one of the building's own answers of S-0405 table T1 holds) and S-0301 claims 9 and 14 (a building's pending delayed shot: rearming at rank 4, yielding only to the building's own answers)
    let mut w = ready(&rules());
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let f = spawn(&mut w, "TOWER", ALPHA, -3, -3);
    let tl = launch(&mut w, PERIOD, 1);
    assert!(superweapons::warp_pending_or_leaving(&w, m));
    assert!(!temporal::holds(&w, f, m));
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, true),
        Answer::Illegal,
        "no shot pending: claim 39's illegal"
    );
    w.obj_mut(f).unwrap().combat.delayed_shot = Some(tl + 30);
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, true),
        Answer::Rearming,
        "the pending shot's rearming"
    );
    // One of the building's own answers of S-0405 table T1 outranks it.
    w.obj_mut(f).unwrap().econ.building.switched_off = true;
    assert_eq!(
        check(&w, f, Some(Target::Object(m)), 0, true),
        Answer::Impossible,
        "not in operation: the building's own impossible"
    );
}

// --- Revision 108: a revert that gives back a member off the map (S-0309 claim 27) -------

#[test]
fn r108_claim39a_a_member_a_revert_gives_back_off_the_map_is_dropped_whatever_the_beam() {
    // spec: S-0513 r108 claim 39a (The firer: a holder that holds the member again after the lock holds it as one that held it at the lock does, unless a revert gave it back while the member is off the map, which is the case of S-0309 claim 27's rule) and S-0309 r113 claim 27 (a member that a revert gives back to a holder while it is off the map is such a target: no target in the firing step, dropped at the start of the holder's next turn, no draw, no revert) and S-0215 claim 10 (a revert brings back the target put aside)
    let Waiting {
        mut w, f, m, dep, ..
    } = waiting(&rules(), "CART", -3, -2);
    run_through(&mut w, dep);
    assert!(superweapons::warp::departed(&w, m));
    assert!(temporal::holds(&w, f, m), "set-up: the beam is attached");
    // An override put the member aside: the firer's target is another
    // object X, its attack on the member suspended.
    let x = spawn(&mut w, "BUGGY", BETA, -6, 6);
    {
        let o = w.obj_mut(f).unwrap();
        o.beh.suspended = Some(Behaviour::Attack);
        o.beh.aside = Some((Some(Target::Object(m)), None));
        o.target = Some(Target::Object(x));
    }
    // X leaves the game: the drop ends the override, and the revert gives
    // the member back while it is off the map.
    glasnost_sim::objects::remove(&mut w, x);
    assert_eq!(target(&w, f), Some(Target::Object(m)), "given back");
    assert!(temporal::holds(&w, f, m), "its beam is still attached");
    assert_eq!(
        glasnost_sim::combat::held::firing_target(&w, w.obj(f).unwrap()),
        None,
        "no target in the firing step"
    );
    step(&mut w);
    assert!(superweapons::warp::departed(&w, m), "still off the map");
    assert_eq!(target(&w, f), None, "dropped at its next turn's start");
    assert_eq!(
        w.obj(f).unwrap().combat.revert_gave_departed,
        None,
        "the mark ends there"
    );
}

// --- Revision 108: a SAM building short of power on the map (SW12 part l10) --------------

/// Part l10's set-up, after S-0215 scenario B16 part (j): SAMT of ALPHA, a
/// type with the SAM flag that needs and drains power, with no charger, and
/// GEN, a test power plant whose output covers the drain, so that ALPHA has
/// full power; a JET of BETA in flight within SAMT's range, which SAMT
/// attacks. Gives the world, SAMT, GEN and the JET.
fn l10() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = rules_sam_drain();
    let mut plant = r.sim.buildings["TOWER"].clone();
    plant.primary = None;
    // spec: S-0513 SW12 (l10) and S-0215 B16 (j) (the test power-plant type's output covers the whole drain)
    plant.econ.power = Some(100);
    r.sim.buildings.insert("GEN".into(), plant);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("GEN".into());
    // The SAM's weapon can fire at aircraft in flight.
    r.sim.projectiles.get_mut("Rocket").unwrap().aa = Some(true);
    // An unarmed test aircraft type that can land (B16 part (j)), strong
    // enough to survive the run.
    r.sim.aircraft.insert(
        "JET".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(10_000),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        },
    );
    r.sim.aircraft.get_mut("JET").unwrap().econ.landable = Some(true);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("JET".into());
    let mut w = game(&r);
    let g = spawn(&mut w, "GEN", ALPHA, -8, 8);
    let f = spawn(&mut w, "SAMT", ALPHA, 0, 0);
    let a = glasnost_sim::testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Aircraft,
        "JET",
        BETA.0,
        at(3, 3),
        0,
    );
    // In flight at its cruise height, held there by a move order far away.
    let ok = orders::issue(
        &mut w,
        BETA,
        orders::Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(3, -12))),
        },
    );
    assert!(ok);
    until(&mut w, 60, |w| w.height_above_surface(a) > 0);
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(a));
    step(&mut w);
    (w, f, g, a)
}

#[test]
fn r108_claim39a_sw12_l10_a_sam_building_short_of_power_drops_its_target_by_the_first_check_tick() {
    // spec: S-0513 r108 claim 39a (The firer, Q-2418: a building that is not charged, S-0405 claim 8, is not in operation while S-0215 claim 32's power row applies, so its check answers S-0405 table T1's impossible, and it drops the target, as a target on the map, no later than its checking turn in the first check tick of S-0309 claim 29 in that state) and SW12 part (l10) (the building holds its target while its house has power and drops it after the power loss, no later than its checking turn in the first tick after the loss whose number is a multiple of 16; a drop before that turn is the aimed stage's and is not judged; a loss in a tick whose number is a multiple of 16 decides nothing)
    let (mut w, f, g, a) = l10();
    assert_eq!(
        glasnost_sim::econ::power::chargers(&w, f),
        0,
        "set-up: no charger"
    );
    // While the house has power the building holds its target.
    for _ in 0..20 {
        step(&mut w);
        assert!(!glasnost_sim::econ::power::low_power(&w, ALPHA));
        assert_eq!(target(&w, f), Some(Target::Object(a)), "held with power");
    }
    // The power loss, in a tick whose number is not a multiple of 16.
    while held_check(w.tick) || held_check(w.tick + 1) {
        step(&mut w);
    }
    // The control: the same ticks with the plant kept.
    let mut control = w.clone();
    glasnost_sim::objects::remove(&mut w, g);
    until(&mut w, 40, |w| {
        glasnost_sim::econ::power::low_power(w, ALPHA)
    });
    let loss = w.tick - 1;
    assert!(!held_check(loss), "set-up: the loss is no check tick");
    let first = (loss + 1..).find(|&t| held_check(t)).unwrap();
    run_through(&mut control, first);
    assert!(!glasnost_sim::econ::power::low_power(&control, ALPHA));
    assert_eq!(
        target(&control, f),
        Some(Target::Object(a)),
        "control: held through tick {first} with power"
    );
    run_through(&mut w, first);
    assert!(glasnost_sim::econ::power::low_power(&w, ALPHA));
    assert!(w.height_above_surface(a) > 0, "set-up: still in flight");
    assert_eq!(
        target(&w, f),
        None,
        "dropped no later than the check turn in tick {first}"
    );
}

// --- Revision 108: a firer in sticky beyond range ----------------------------------------

#[test]
fn r108_claim39a_a_sticky_firer_beyond_range_drops_the_member_with_its_destination_and_keeps_its_beam()
 {
    // spec: S-0513 r108 claim 39a (The firer: an infantry or vehicle firer beyond its weapon's range of the old position takes S-0215 claim 37's approach step as for a target on the map there; in sticky, and in that table's other rows that drop an out-of-range target, it drops the member with the destination; dropping it does not release the beam) and S-0309 r113 claim 27 (it keeps the member, approaches it, or drops it with the destination as claim 37's parts (a), (d) and (g) give); S-0215 claim 37 (d) (sticky drops an out-of-range target and the destination at once)
    let Waiting {
        mut w, f, m, dep, ..
    } = given_at_departure(&rules_r91(), "CART", -8, -8, |_, _| {});
    assert!(
        !glasnost_sim::approach::in_range(&w, f, Target::Object(m)),
        "set-up: beyond range"
    );
    // The departure update runs; then the firer is in sticky with a
    // destination, and the harness runs S-0215 claim 37's approach step for
    // it (sticky is processed as guard, S-0215 claim 5, which takes no
    // approach step of its own, so no processing of ours reaches the row).
    step(&mut w);
    assert_eq!(w.tick - 1, dep);
    assert!(superweapons::warp::departed(&w, m), "set-up: off the map");
    let t = w.tick;
    let o = w.obj_mut(f).unwrap();
    o.beh.force(Behaviour::Sticky, t, false);
    o.target = Some(Target::Object(m));
    o.dest = Some(Target::Cell(at(-7, -7)));
    glasnost_sim::approach::step(&mut w, f);
    let o = w.obj(f).unwrap();
    assert_eq!(o.target, None, "sticky drops the member beyond range");
    assert_eq!(o.dest, None, "the destination goes with it");
    assert!(
        temporal::holds(&w, f, m),
        "dropping it does not release the beam"
    );
}
