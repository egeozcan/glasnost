//! Claim tests of S-0513 revision 43, claim 39 (Q-2156): the holders of a
//! superweapon warp member at its departure, in its leaving stage and at
//! its landing, on synthetic rules (`tests/sw_common`). Every name and
//! number is invented.
//!
//! A change made in the member's own update is told apart from a holder's
//! own processing as SW12 part i does: the member's warp update is called
//! on its own (`superweapons::warp::turn`) between two reads. Where a whole
//! tick is run, the holders are created before the member, so their turns
//! come first in the tick (S-0203 claim 3) and an end-of-tick read follows
//! the member's update.

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons::warp::{self, Stage};
use glasnost_sim::superweapons::{ProtectKind, protect};
use glasnost_sim::teleport::{self, TeleportState};
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

/// Launches ALPHA's warp of the block at `at(0, 0)` to `at(5, 5)`; returns
/// t_L, the tick that applied the warp order, which has run.
fn launch(w: &mut World) -> i32 {
    fire_now(w, ALPHA, SOURCE, at(0, 0));
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

fn hold(w: &mut World, holder: ObjectId, member: ObjectId, left: i32, opportunity: bool) {
    let tick = w.tick;
    let o = w.obj_mut(holder).unwrap();
    o.target = Some(Target::Object(member));
    o.targeting_timer = tick + left;
    o.combat.opportunity = opportunity;
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().target
}

/// Two holders (a vehicle and an infantry unit, both outside the warp
/// block and within range of it), then the member, an unarmed vehicle of
/// the other house, in the block; the warp launched and run to the update
/// before departure (the wait expired, S-0513 T2).
fn at_expiry(r: &Ruleset) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = ready(r);
    let h1 = spawn(&mut w, "CART", ALPHA, 0, 3);
    let h2 = spawn(&mut w, "WALKER", ALPHA, 3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    (w, h1, h2, m)
}

// --- Departure ------------------------------------------------------------------

#[test]
fn claim39_departure_clears_no_holder_and_takes_no_timer_draw() {
    // spec: S-0513 claim 39 (departure itself clears no holder's target and resets no holder's targeting timer: no ranged draw over 4 to 8 ticks for a holder, whatever its timer shows; SW12 part i, H1 with more than 10 ticks left, H2 with at most 10)
    let (mut w, h1, h2, m) = at_expiry(&rules());
    // The control: the same world with nobody holding the member.
    let mut control = w.clone();
    hold(&mut w, h1, m, 50, true);
    hold(&mut w, h2, m, 10, true);
    let timers = (
        w.obj(h1).unwrap().targeting_timer,
        w.obj(h2).unwrap().targeting_timer,
    );
    let before = w.rng.clone();
    assert!(warp::turn(&mut w, m), "the member's departure update");
    assert!(warp::turn(&mut control, m));
    assert_eq!(
        stage(&w, m),
        Some(Stage::Away),
        "departed, landing check passed"
    );
    assert!(!w.obj(m).unwrap().on_map, "off the map until it appears");
    assert_eq!(target(&w, h1), Some(Target::Object(m)));
    assert_eq!(target(&w, h2), Some(Target::Object(m)));
    assert_eq!(
        (
            w.obj(h1).unwrap().targeting_timer,
            w.obj(h2).unwrap().targeting_timer
        ),
        timers,
        "no timer reset"
    );
    assert_eq!(w.rng, control.rng, "the departure adds no holder draw");
    // The warp-out animation here has a constant delay: no draw at all.
    assert_eq!(w.rng, before);
}

#[test]
fn claim39_departure_reverts_no_overridden_holder() {
    // spec: S-0513 claim 39 and its Changes for S-0215 claim 10 (the departure is not that claim's off-map revert cause: a holder with an override in effect keeps the member as its target and keeps its suspended behaviour)
    let (mut w, h1, _, m) = at_expiry(&rules());
    let aside = spawn(&mut w, "BUGGY", BETA, -3, 3);
    {
        let o = w.obj_mut(h1).unwrap();
        o.beh.current = Some(Behaviour::Attack);
        o.beh.suspended = Some(Behaviour::Guard);
        o.beh.aside = Some((Some(Target::Object(aside)), None));
        o.target = Some(Target::Object(m));
    }
    warp::turn(&mut w, m);
    let o = w.obj(h1).unwrap();
    assert_eq!(o.target, Some(Target::Object(m)));
    assert_eq!(o.beh.suspended, Some(Behaviour::Guard));
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

#[test]
fn claim39_a_projectile_in_flight_keeps_the_member() {
    // spec: S-0513 claim 39 and its Changes for S-0306 claim 17 (a projectile already in flight towards the member does not lose it at departure: its target does not become the cell the member stood in)
    let (mut w, h1, _, m) = at_expiry(&rules());
    // The arcing shot aims at the firer's own target (S-0306 claim 3).
    hold(&mut w, h1, m, 50, true);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        h1,
        0,
        Target::Object(m)
    ));
    let aimed = |w: &World| {
        w.projectiles
            .values()
            .filter(|p| p.target == Some(Target::Object(m)))
            .count()
    };
    assert_eq!(aimed(&w), 1);
    warp::turn(&mut w, m);
    assert_eq!(aimed(&w), 1, "still aimed at the member object");
}

#[test]
fn claim39_whole_tick_holder_first_then_the_interim_drop() {
    // spec: S-0513 claim 39 (the departure clears no holder's target; a holder's own processing in the same tick is its own) and SW12 part i (H1 holds the tank through the departure update, its timer counting down from its value before departure)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "WALKER", ALPHA, 3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 1);
    assert_eq!(stage(&w, m), Some(Stage::Leaving(tl + 1)));
    hold(&mut w, h, m, 100, true);
    let timer = w.obj(h).unwrap().targeting_timer;
    // spec: S-0513 claim 39 (an infantry holder whose weapon does not heal keeps the member when its firing step gets the leaving stage's answer; SW12 part i: no holder drops it in its firing step in the leaving stage)
    while w.tick <= tl + 61 {
        step(&mut w);
        assert_eq!(
            target(&w, h),
            Some(Target::Object(m)),
            "tick {}",
            w.tick - 1
        );
    }
    // The departure tick: the holder's turn comes first, then the member departs.
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Away));
    assert_eq!(
        target(&w, h),
        Some(Target::Object(m)),
        "kept through the departure"
    );
    assert_eq!(w.obj(h).unwrap().targeting_timer, timer);
    // spec: S-0309 claim 27 (DECIDED turn-start drop) applied to a departed warp member, our interim for S-0513 claim 39's open question (Q-2211): the holder's own next turn drops it, with no timer draw and no revert
    step(&mut w);
    assert_eq!(target(&w, h), None);
    assert_eq!(w.obj(h).unwrap().targeting_timer, timer, "no draw");
}

// --- The leaving stage's firing check -------------------------------------------------

#[test]
fn claim39_leaving_stage_answers_illegal_ahead_of_rearming_and_range() {
    // spec: S-0513 claim 39 and its Changes for S-0301 (a target in the leaving stage answers illegal, a rank-4 reason that outranks a running rearm countdown, rank 5, and the range test, rank 10)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let far = spawn(&mut w, "CART", ALPHA, 0, -9);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let other = spawn(&mut w, "BUGGY", BETA, 3, 3);
    let tl = launch(&mut w);
    let rearming = |w: &mut World| {
        let t = w.tick;
        w.obj_mut(h).unwrap().combat.rearm_until = t + 100;
    };
    // Pending (before its first update): not yet the leaving stage.
    assert_eq!(stage(&w, m), Some(Stage::Pending));
    rearming(&mut w);
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Rearming
    );
    run_through(&mut w, tl + 30);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    rearming(&mut w);
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Illegal
    );
    // The control target, not in a warp, gets the rearm countdown's answer.
    assert_eq!(
        check(&w, h, Some(Target::Object(other)), 0, true),
        Answer::Rearming
    );
    // Out of range: illegal, not out of range.
    assert_eq!(
        check(&w, far, Some(Target::Object(m)), 0, true),
        Answer::Illegal
    );
    assert_eq!(
        check(&w, far, Some(Target::Object(other)), 0, true),
        Answer::OutOfRange
    );
    // The expiry update is still the leaving stage (Terms).
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    rearming(&mut w);
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Illegal
    );
}

#[test]
fn claim39_the_materialising_firer_keeps_its_earlier_rearming() {
    // spec: S-0513 claim 39 and its Changes for S-0301 (the leaving stage's answer comes after the early rearming of a materialising firer) and S-0301 claim 9 rank 4
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 30);
    let t = w.tick;
    w.obj_mut(h).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, 100)),
        ..Default::default()
    });
    assert!(teleport::phasing_in(&w, h));
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Rearming
    );
}

#[test]
fn claim39_a_temporal_weapon_is_not_refused_by_the_leaving_stage() {
    // spec: S-0513 claim 39 and its Changes for S-0301 (illegal unless the asking weapon is a temporal weapon, S-0301 Terms: its warhead has the temporal flag)
    let mut r = rules();
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    let mut w = ready(&r);
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 30);
    let t = w.tick;
    w.obj_mut(h).unwrap().combat.rearm_until = t + 100;
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Rearming
    );
}

#[test]
fn claim39_an_opportunity_target_is_dropped_at_the_search_without_a_drop_draw() {
    // spec: S-0513 claim 39 (a holder whose weapon is not temporal and whose member is an opportunity target drops it at its next search in the leaving stage, S-0309 claims 4 and 19; the drop is not claim 27's and takes no draw over 4 to 8 ticks; SW12 part i, H3)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "WALKER", ALPHA, 3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 10);
    // The control: the same holder holding nothing, with the same timer.
    let mut control = w.clone();
    hold(&mut w, h, m, 5, true);
    let t = w.tick;
    control.obj_mut(h).unwrap().targeting_timer = t + 5;
    run_through(&mut w, tl + 20);
    run_through(&mut control, tl + 20);
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    assert_eq!(target(&w, h), None, "dropped at its search");
    assert!(
        w.obj(h).unwrap().targeting_timer > t + 5,
        "the search ran and restarted the timer"
    );
    assert_eq!(
        w.obj(h).unwrap().targeting_timer,
        control.obj(h).unwrap().targeting_timer
    );
    assert_eq!(w.rng, control.rng, "no draw beyond the search's own");
}

// --- The landing check's kill ---------------------------------------------------------

#[test]
fn claim39_holders_of_a_member_killed_by_its_landing_check_draw_after_the_warp_out() {
    // spec: S-0513 claim 39 (a member that the landing check kills, claim 37, is a destroyed object: its holders drop it as S-0309 claim 27 gives, with that claim's timer reset, at the hit that kills it; the warp-out animation is made before the landing check, so all its creation draws come before those holders' draws) and S-0309 claim 27 (holders draw in creation order)
    let mut r = rules();
    // A random rate pair whose delays differ gives the animation's creation one ranged draw (S-0109 claim 3).
    r.sim.animations.get_mut("WARPA").unwrap().random_rate = Some([450, 225]);
    let mut w = ready(&r);
    let h1 = spawn(&mut w, "CART", ALPHA, 0, 3);
    let h2 = spawn(&mut w, "WALKER", ALPHA, 3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let guard = spawn(&mut w, "BUGGY", BETA, 5, 5);
    let tl = launch(&mut w);
    run_through(&mut w, tl + 61);
    protect::start(&mut w, guard, ProtectKind::Curtain, 500);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    hold(&mut w, h1, m, 50, true);
    hold(&mut w, h2, m, 60, true);
    // The control: the member lands on a free cell with nobody holding it,
    // so its update takes the warp-out animation's draws only.
    let mut control = w.clone();
    for h in [h1, h2] {
        control.obj_mut(h).unwrap().target = None;
    }
    let free = at(-5, 5);
    if let Some(mm) = control.obj_mut(m).unwrap().sw.warp.as_mut() {
        mm.landing = Pos::new(free.x * 256 + 128, free.y * 256 + 128, mm.landing.z);
    }
    let before = w.rng.taken();
    warp::turn(&mut control, m);
    assert_eq!(stage(&control, m), Some(Stage::Away));
    assert!(control.rng.taken() > before, "the warp-out animation drew");
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expected = control.rng.clone();
    let d1 = expected.ranged(4, 8, half);
    let d2 = expected.ranged(4, 8, half);
    let t = w.tick;
    warp::turn(&mut w, m);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "killed by the landing check"
    );
    assert_eq!(target(&w, h1), None);
    assert_eq!(target(&w, h2), None);
    assert_eq!(
        w.obj(h1).unwrap().targeting_timer,
        t + d1,
        "first created draws first"
    );
    assert_eq!(w.obj(h2).unwrap().targeting_timer, t + d2);
}

// --- The own jump -------------------------------------------------------------------

#[test]
fn claim39_the_own_jump_drops_with_no_timer_draw() {
    // spec: S-0513 claim 39 and its Changes for S-0207 claim 20 (e) (the teleporting locomotor's own jump never gives a holder the ranged draw over 4 to 8 ticks: a holder that drops the jumper keeps its timer, whatever it shows)
    let mut r = rules();
    let mut tele = r.sim.vehicles["CART"].clone();
    tele.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport);
    tele.harvester = Some(false);
    r.sim.vehicles.insert("TELE".into(), tele);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("TELE".into());
    let mut w = game(&r);
    let h = spawn(&mut w, "CART", BETA, 0, 9);
    let j = spawn(&mut w, "TELE", ALPHA, 0, 0);
    let order = glasnost_sim::orders::Order::Behaviour {
        actor: j,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(at(-4, -4))),
    };
    assert!(glasnost_sim::orders::issue(&mut w, ALPHA, order));
    let start = w.obj(j).unwrap().pos;
    for _ in 0..40 {
        let t = w.tick;
        {
            let o = w.obj_mut(h).unwrap();
            o.target = Some(Target::Object(j));
            o.targeting_timer = t + 50;
        }
        step(&mut w);
        if w.obj(j).unwrap().pos != start {
            assert_ne!(
                target(&w, h),
                Some(Target::Object(j)),
                "dropped at the jump"
            );
            assert_eq!(w.obj(h).unwrap().targeting_timer, t + 50, "no draw");
            return;
        }
    }
    panic!("the jumper did not jump");
}
