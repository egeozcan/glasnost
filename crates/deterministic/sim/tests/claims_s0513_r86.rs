//! Claim tests of S-0513 revision 86, claim 39a (Q-2276 answered), with
//! S-0514 revision 12 (claims 3, 9, 14 and 17) and S-0610 revision 12
//! (claim 5's excluded updates) (M8 step A3 intake 75): a temporal beam
//! locked on a superweapon warp member by a projectile in flight at the
//! launch, the erasure update in every member turn before its warp step,
//! the no-step rule, the departure's ending of the being-erased state while
//! the beam stays attached, the turns cut short while phasing in, the
//! release by a firer that approaches, a later lock, and the two quirk
//! switches. Synthetic rules (`tests/sw_common`): every name and number is
//! invented. The ledger is `docs/m8-claims/intake75.md`.
//!
//! Tick reading: after `tick::step` has run tick *k*, the tick number reads
//! *k* + 1. The warp order is applied after the turn pass of tick *t*_L
//! (S-0203 claim 7).

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::combat::held;
use glasnost_sim::econ::stats;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::temporal;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use sw_common::*;

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
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    // Every weapon of warhead HE is a temporal weapon (S-0301 Terms): the
    // CART's Gun (damage 30, an arcing Shell) and the TOWER's Cannon.
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    r
}

/// The rules with the Gun's damage, the raw damage of S-0514 claim 7, set
/// to `n`.
fn gun(n: i32) -> Ruleset {
    let mut r = rules();
    r.sim.weapons.get_mut("Gun").unwrap().damage = Some(n);
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn amount(w: &World, id: ObjectId) -> Option<i32> {
    w.temporal.locks.get(&id).map(|l| l.amount)
}

/// A game with ALPHA's teleport source ready.
fn ready(r: &Ruleset) -> World {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    w
}

/// Runs until tick `t` has run (the next tick to run is `t + 1`).
fn run_through(w: &mut World, t: i32) {
    while w.tick <= t {
        step(w);
    }
}

/// Gives `id` target `x` and current behaviour `b` directly, as an order
/// would.
fn hold(w: &mut World, id: ObjectId, x: ObjectId, b: Behaviour) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, false);
    o.target = Some(Target::Object(x));
    o.combat.opportunity = false;
    o.targeting_timer = t + 200;
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

fn departure_seen(w: &World) -> bool {
    anims(w).iter().any(|a| a == "WARPA")
        || w.effects.iter().any(|e| {
            matches!(
                e,
                Effect::Sound {
                    sound: Sound::ChronoOut,
                    ..
                }
            )
        })
}

fn mv(actor: ObjectId, c: glasnost_map::CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

/// An ordinary hit of 10 of warhead SA with no attacker (S-0302 claim 8's
/// protection of an object being erased applies to it).
fn hit(w: &mut World, id: ObjectId) {
    let sa = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "SA")
        .unwrap() as u32;
    damage::apply(
        w,
        id,
        Hit {
            amount: 10,
            warhead: Some(sa),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

/// A run of SW12 part l's set-up on our engine: firer T, a CART of ALPHA
/// at `at(tx, ty)`, in *attack* on member M, a BUGGY of BETA at `at(0, 0)`
/// (strength 120, so R₀ = 1200, S-0514 F1). T makes its first shot before
/// the launch; the launch comes in the tick before that shot's projectile
/// detonates at M, so the projectile is in flight at the launch (the first
/// lock route) and no check tick of S-0309 claim 29 falls between them.
struct Run {
    w: World,
    t: ObjectId,
    m: ObjectId,
    /// t_L.
    tl: i32,
    /// The detonation tick d.
    d: i32,
}

fn in_flight(r: &Ruleset, tx: i32, ty: i32) -> Run {
    let mut w = ready(r);
    let t = spawn(&mut w, "CART", ALPHA, tx, ty);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    hold(&mut w, t, m, Behaviour::Attack);
    until(&mut w, 300, |w| shots_by(w, t) == 1);
    // The detonation tick, read from a copy that is not launched.
    let mut p = w.clone();
    let d = until(&mut p, 200, |p| temporal::attached(p, m)) - 1;
    assert!(d > w.tick, "the shot is in flight");
    assert!(
        !held::check_tick(d),
        "set-up: no check tick at the detonation"
    );
    let tl = d - 1;
    run_through(&mut w, tl - 1);
    assert!(!orders::blocked(&w));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    assert_eq!(w.tick, tl);
    step(&mut w);
    assert_eq!(
        stage(&w, m),
        Some(Stage::Pending),
        "launched with the shot in flight"
    );
    assert!(!temporal::attached(&w, m));
    Run { w, t, m, tl, d }
}

// --- The lock, the erasure in every turn, the departure (SW12 part l1) -------------

#[test]
fn claim39a_l1_a_shot_in_flight_locks_the_member_and_the_erasure_removes_it_after_its_hold() {
    // spec: S-0513 claim 39a (The lock routes: a temporal shot made in t_L or earlier still in flight at the launch; The lock: by S-0514 claims 1 to 4 alone, after the member's turn of tick d; The warp goes on: departure in t_L + 62; The erasure goes on: an erasure update in each member turn after the lock, at every stage, removal in r = d + k; Departure: no longer being erased while the beam stays attached, ordinary turns after the hold; The firer: rearming, no further shot) and S-0514 claims 3, 14 and 17
    // Gun damage 10: k = ⌈1200 / 10⌉ = 120 erasure updates (S-0514 F3).
    let Run {
        mut w, t, m, tl, d, ..
    } = in_flight(&gun(10), 3, 2);
    run_through(&mut w, d);
    assert!(temporal::holds(&w, t, m), "the lock attaches at d");
    assert!(temporal::erased(&w, m));
    assert_eq!(amount(&w, m), Some(1200), "the first update is in d + 1");
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    assert!(superweapons::being_erased(&w, m));
    assert_eq!(
        amount(&w, m),
        Some(1200 - 10 * (tl + 61 - d)),
        "an erasure update in every warp turn from d + 1"
    );
    assert_eq!(shots_by(&w, t), 1, "one shot up to the departure");
    w.effects.clear();
    run_through(&mut w, tl + 62);
    assert!(departure_seen(&w), "it departs in t_L + 62");
    assert!(superweapons::warp::departed(&w, m));
    assert!(temporal::attached(&w, m), "the beam stays attached");
    assert!(temporal::holds(&w, t, m));
    assert!(!temporal::erased(&w, m), "no longer being erased");
    assert!(!superweapons::being_erased(&w, m));
    assert_eq!(amount(&w, m), Some(1200 - 10 * (tl + 62 - d)));
    // The hold ends; ordinary turns follow with the beam still attached.
    let end = until(&mut w, 40, |w| stage(w, m).is_none());
    let r = d + 120;
    assert!(end < r - 10);
    assert_eq!(amount(&w, m), Some(1200 - 10 * (end - 1 - d)));
    let health = w.obj(m).unwrap().health;
    hit(&mut w, m);
    assert!(
        w.obj(m).unwrap().health < health,
        "ordinary damage: no being-erased protection"
    );
    assert!(orders::issue(&mut w, BETA, mv(m, at(4, 4))));
    let p0 = pos(&w, m);
    run_through(&mut w, r - 1);
    assert!(w.obj(m).is_some());
    assert_ne!(pos(&w, m), p0, "an ordinary turn: it moves");
    assert!(temporal::holds(&w, t, m));
    assert_eq!(shots_by(&w, t), 0, "the firer makes no further shot");
    assert_eq!(w.temporal.locks[&m].leader, t);
    run_through(&mut w, r);
    assert!(w.obj(m).is_none(), "removed in tick d + k");
    assert!(!temporal::holds(&w, t, m));
    assert_eq!(stats::of(&w, ALPHA).kills, 1, "the leader is credited");
}

#[test]
fn claim39a_l2_a_member_removed_before_its_departure_does_not_depart() {
    // spec: S-0513 claim 39a (The erasure goes on: removed at its k-th erasure update after the lock whatever its stage; the no-step rule) and S-0514 claims 7, 8 and 12
    // Gun damage 30: k = 40, so r = d + 40 falls in the leaving stage.
    let Run {
        mut w, t, m, tl, d, ..
    } = in_flight(&gun(30), 3, 2);
    let r = d + 40;
    assert!(r < tl + 61);
    run_through(&mut w, r - 1);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    assert_eq!(amount(&w, m), Some(30));
    run_through(&mut w, r);
    assert!(w.obj(m).is_none(), "removed in tick d + k");
    assert!(!temporal::holds(&w, t, m));
    w.effects.clear();
    run_through(&mut w, tl + 64);
    assert!(!departure_seen(&w), "no departure");
    assert_eq!(stats::of(&w, ALPHA).kills, 1);
}

// --- The no-step rule (variant l4) ------------------------------------------------------

#[test]
fn claim39a_l4_a_member_removed_in_its_departure_update_takes_no_warp_step() {
    // spec: S-0513 claim 39a (The erasure goes on: the erasure update comes before the turn's warp step; a member removed this way takes no warp step in that update or later, the no-step rule, ours: Departure from the original) and Edge cases (no switch)
    let Run {
        mut w, m, tl, d, ..
    } = in_flight(&gun(10), 3, 2);
    run_through(&mut w, d);
    // The amount left for exactly tl + 62 − d updates of 10: the last in t_L + 62.
    w.temporal.locks.get_mut(&m).unwrap().amount = 10 * (tl + 62 - d);
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    w.effects.clear();
    run_through(&mut w, tl + 62);
    assert!(w.obj(m).is_none(), "removed in its departure update");
    assert!(
        !departure_seen(&w),
        "no departure animation or sound: the erasure update came first"
    );
}

#[test]
fn claim39a_l4_a_member_removed_in_its_expiry_update_is_removed_before_the_expiry() {
    // spec: S-0513 claim 39a (the no-step rule: a member removed in its expiry update makes no warp step there) and claim 39 (the wait expires in t_L + 61)
    let Run {
        mut w, m, tl, d, ..
    } = in_flight(&gun(10), 3, 2);
    run_through(&mut w, d);
    w.temporal.locks.get_mut(&m).unwrap().amount = 10 * (tl + 61 - d);
    run_through(&mut w, tl + 60);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    run_through(&mut w, tl + 61);
    assert!(w.obj(m).is_none(), "removed in its expiry update");
    w.effects.clear();
    run_through(&mut w, tl + 63);
    assert!(!departure_seen(&w));
}

// --- Release by an approaching firer (variant l3) ---------------------------------------

#[test]
fn claim39a_l3_a_firer_in_attack_approaches_a_member_landed_beyond_range_and_lets_go() {
    // spec: S-0513 claim 39a (Release: the warp is no release cause; a firer in attack whose member lands beyond its weapon's range moves towards it and lets go at its first arrival in a new cell; as the last attacker the member loses all progress) and S-0514 claims 10 and 14
    // T at (−3, −2): within the Gun's 5.5 cells of (0, 0), beyond them from (5, 5).
    let Run {
        mut w, t, m, tl, d, ..
    } = in_flight(&gun(10), -3, -2);
    run_through(&mut w, tl + 62);
    assert!(temporal::holds(&w, t, m), "the departure releases nothing");
    assert_eq!(w.obj(t).unwrap().target, Some(Target::Object(m)));
    let c0 = w.obj(t).unwrap().pos.cell();
    let gone = until(&mut w, 200, |w| !temporal::holds(w, t, m));
    assert!(gone < d + 120, "before the removal tick");
    assert_ne!(
        w.obj(t).unwrap().pos.cell(),
        c0,
        "it lets go on arriving in a new cell"
    );
    assert!(!temporal::attached(&w, m), "the last release");
    assert_eq!(amount(&w, m), None, "all progress lost");
    run_through(&mut w, d + 125);
    assert!(w.obj(m).is_some(), "not removed");
}

// --- A later lock (variant l5) and the handover -----------------------------------------

#[test]
fn claim39a_l5_a_joining_lock_after_the_hold_makes_the_member_erased_again_and_removes_it_earlier()
{
    // spec: S-0513 claim 39a (A lock after the departure: a firer not on the beam joins without resetting R; the member is being erased again with every effect of that state, S-0514 claim 17's skipped turns and S-0302 claim 8's protection; the removal comes earlier than r when ⌈R′ / S′⌉ < ⌈R′ / S⌉; the leader stays the leader and is credited) and S-0514 claims 4, 7, 13 and 17
    let Run { mut w, t, m, d, .. } = in_flight(&gun(10), 3, 2);
    let t2 = spawn(&mut w, "CART", ALPHA, -8, 8);
    until(&mut w, 120, |w| stage(w, m).is_none());
    assert!(!temporal::erased(&w, m));
    let r = d + 120;
    // The detonation of t2's shot at the member (S-0514 claims 1 to 4).
    temporal::deliver(&mut w, Some(t2), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m), "being erased again");
    assert!(superweapons::being_erased(&w, m));
    let l = &w.temporal.locks[&m];
    assert_eq!((l.leader, l.joiners.clone()), (t, vec![t2]));
    let rest = l.amount;
    // S′ = 20: the next update is in this tick, so the removal comes ⌈R′ / 20⌉ − 1 ticks on.
    let expected = w.tick + (rest + 19) / 20 - 1;
    assert!(expected < r);
    let health = w.obj(m).unwrap().health;
    hit(&mut w, m);
    assert_eq!(w.obj(m).unwrap().health, health, "protected again");
    assert!(orders::issue(&mut w, BETA, mv(m, at(4, 4))));
    let p0 = pos(&w, m);
    run_through(&mut w, expected - 1);
    assert_eq!(pos(&w, m), p0, "skipped turns: no movement");
    assert_eq!(w.obj(m).unwrap().dest, None);
    run_through(&mut w, expected);
    assert!(w.obj(m).is_none(), "removed earlier than r");
    assert_eq!(stats::of(&w, ALPHA).kills, 1);
}

/// A member being erased in its leaving stage by `f`'s beam (a delivery
/// standing for the detonation of a temporal projectile in flight at the
/// launch), with an unprotected TOWER of its owner on its landing cell, so
/// that the departure's landing check is redirected and the hold is the
/// teleport delay of 7 (S-0513 claims 37 and 40). The member's type has
/// strength 10 000, so no erasure here completes. Runs into the hold.
fn long_hold(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let (w, f, _, m) = long_hold_joined(r, name, false);
    (w, f, m)
}

/// [`long_hold`], with a second firer `g` joining the beam before the
/// departure when `join` holds.
fn long_hold_joined(
    r: &Ruleset,
    name: &str,
    join: bool,
) -> (World, ObjectId, Option<ObjectId>, ObjectId) {
    let mut r = r.clone();
    for t in [r.sim.vehicles.get_mut(name), r.sim.infantry.get_mut(name)]
        .into_iter()
        .flatten()
    {
        t.strength = Some(10_000);
    }
    let mut w = ready(&r);
    let f = spawn(&mut w, "CART", ALPHA, -8, 8);
    let m = spawn(&mut w, name, BETA, 0, 0);
    spawn(&mut w, "TOWER", BETA, 5, 5);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    while orders::blocked(&w) {
        step(&mut w);
    }
    fire(&mut w, ALPHA, WARP, at(5, 5));
    step(&mut w);
    step(&mut w);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    temporal::deliver(&mut w, Some(f), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    let g = join.then(|| {
        let g = spawn(&mut w, "CART", ALPHA, -8, 6);
        temporal::deliver(&mut w, Some(g), Some(Target::Object(m)));
        g
    });
    until(&mut w, 120, |w| matches!(stage(w, m), Some(Stage::Hold(_))));
    assert!(w.obj(m).unwrap().sw.warp.as_ref().unwrap().redirected);
    // The first hold turn has come; at least 5 more follow.
    step(&mut w);
    assert!(matches!(stage(&w, m), Some(Stage::Hold(_))));
    assert!(temporal::attached(&w, m) && !temporal::erased(&w, m));
    (w, f, g, m)
}

/// One tick with an order of the member's owner applied before it: the
/// order gives it a destination and *move*; the next turn either keeps or
/// clears them.
fn ordered_turn(w: &mut World, m: ObjectId) {
    assert!(orders::issue(w, BETA, mv(m, at(5, 9))));
    step(w);
    assert!(
        w.obj(m).unwrap().dest.is_some(),
        "the order gave a destination"
    );
    step(w);
}

#[test]
fn claim39a_a_vehicle_member_with_a_beam_has_its_turns_cut_short_while_it_phases_in() {
    // spec: S-0513 claim 39a (Departure: while it phases in, a vehicle or aircraft with a temporal beam attached has its turns cut short whether or not it is being erased: it clears its target and destination and takes no ordinary behaviour or movement) and claim 51 (the hold row); S-0514 claim 17
    let (mut w, _, m) = long_hold(&rules(), "BUGGY");
    let p0 = pos(&w, m);
    ordered_turn(&mut w, m);
    assert!(matches!(stage(&w, m), Some(Stage::Hold(_))));
    assert_eq!(w.obj(m).unwrap().dest, None, "destination cleared");
    assert_eq!(pos(&w, m), p0);
    // The hold ends: ordinary turns with the beam attached.
    until(&mut w, 20, |w| stage(w, m).is_none());
    assert!(temporal::attached(&w, m));
    assert!(orders::issue(&mut w, BETA, mv(m, at(5, 9))));
    run(&mut w, 40);
    assert_ne!(pos(&w, m), p0, "it moves after the hold");
}

#[test]
fn claim39a_an_infantry_member_with_a_beam_phases_in_as_one_with_no_beam_in_both_positions() {
    // spec: S-0513 claim 39a and Edge cases (quirks.phase_in_beam_spares_infantry: on, an infantry unit's turns are cut short while phasing in only while it is being erased; off, also while a beam is attached) and S-0514 claim 17
    for spares in [true, false] {
        let mut r = rules();
        r.globals.quirks.phase_in_beam_spares_infantry = spares;
        let (mut w, _, m) = long_hold(&r, "JUMPER");
        ordered_turn(&mut w, m);
        assert!(matches!(stage(&w, m), Some(Stage::Hold(_))));
        assert_eq!(
            w.obj(m).unwrap().dest.is_some(),
            spares,
            "quirk {spares}: the destination is kept only when not cut short"
        );
    }
}

#[test]
fn claim39a_the_berserk_countdown_skips_the_cut_short_phasing_in_updates() {
    // spec: S-0610 claim 5 (Excluded updates: those in which a superweapon-warp member phases in that S-0215 claim 2 excludes: a vehicle member with a temporal beam attached, being erased or not; an infantry member while being erased or, with quirks.phase_in_beam_spares_infantry off, while a beam is attached; such an update takes no countdown step) and Terms (excluded update)
    for (name, spares, steps) in [
        ("BUGGY", true, false),
        ("JUMPER", true, true),
        ("JUMPER", false, false),
    ] {
        let mut r = rules();
        r.globals.quirks.phase_in_beam_spares_infantry = spares;
        let (mut w, _, m) = long_hold(&r, name);
        w.obj_mut(m).unwrap().combat.berserk = Some(50);
        step(&mut w);
        assert!(matches!(stage(&w, m), Some(Stage::Hold(_))));
        let left = w.obj(m).unwrap().combat.berserk;
        assert_eq!(
            left,
            Some(if steps { 49 } else { 50 }),
            "{name}, quirk {spares}"
        );
    }
}

#[test]
fn claim39a_a_handover_after_the_departure_keeps_the_member_not_erased_up_to_a_later_lock() {
    // spec: S-0514 claim 9 (exception: a warp member that stopped being erased at its departure while its beam stays attached stays not being erased through a handover, up to a later lock after the departure) and S-0513 claim 39a (Departure; A lock after the departure: every lock makes it being erased again)
    let (mut w, f, g, m) = long_hold_joined(&rules(), "BUGGY", true);
    let g = g.unwrap();
    let rest = amount(&w, m);
    // The leader lets go (S-0514 claim 14: a fall-back release).
    temporal::let_go(&mut w, f);
    assert_eq!(w.temporal.locks[&m].leader, g, "the joiner leads");
    assert_eq!(amount(&w, m), rest, "with the same amount");
    assert!(
        !temporal::erased(&w, m),
        "still not being erased after the handover"
    );
    let k = spawn(&mut w, "CART", ALPHA, -6, 6);
    temporal::deliver(&mut w, Some(k), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m), "a later lock: being erased again");
    assert_eq!(amount(&w, m), rest, "joined without resetting R");
}

// --- The switch -------------------------------------------------------------------------

#[test]
fn claim39a_switch_off_a_pending_warp_is_one_more_lock_refusal() {
    // spec: S-0513 claim 39a (Switch: quirks.warp_departure_keeps_beam off, a target with a superweapon warp pending, from the launch until its departure update, is one more lock refusal of S-0514 claim 3; no beam of these routes is attached and the warp goes on) and S-0514 claim 3
    let mut r = gun(10);
    r.globals.quirks.warp_departure_keeps_beam = false;
    let Run {
        mut w, t, m, tl, d, ..
    } = in_flight(&r, 3, 2);
    run_through(&mut w, d);
    assert!(!temporal::attached(&w, m), "refused");
    assert!(!temporal::holds(&w, t, m));
    w.effects.clear();
    run_through(&mut w, tl + 62);
    assert!(departure_seen(&w), "the warp goes on");
    until(&mut w, 40, |w| stage(w, m).is_none());
    run_through(&mut w, d + 130);
    assert!(w.obj(m).is_some(), "no removal");
    // After the departure a lock is a leader's lock, as usual.
    temporal::deliver(&mut w, Some(t), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    assert_eq!(amount(&w, m), Some(1200));
}

#[test]
fn claim39a_switch_on_a_pending_warp_is_no_lock_refusal() {
    // spec: S-0514 claim 3 (a target's pending superweapon warp, its leaving stage included, is not a refusal) and S-0513 claim 39a (The lock)
    let mut r = gun(10);
    r.globals.quirks.warp_departure_keeps_beam = true;
    let Run { mut w, t, m, d, .. } = in_flight(&r, 3, 2);
    run_through(&mut w, d);
    assert!(temporal::holds(&w, t, m));
}
