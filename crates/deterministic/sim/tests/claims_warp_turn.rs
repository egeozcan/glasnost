//! Claim tests of the warp turn's whole-member-turn exclusion (S-0513 claims 51 and 52, Q-0870
//! answered, with S-0203 claim 4, S-0516 claim 42, S-0301 claims 11 and 13 and S-0614 claim 18):
//! from the first pending warp update through arrival effects the warp update is the whole member
//! turn; ordinary eligibility returns in the arrival hold after the hold-end test, with no
//! catch-up, while elapsed-tick clocks keep running. Engine-design contract tests (SW23 part f),
//! on the synthetic rules of `tests/sw_common`; they say nothing about the original (claim 52).

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::cloak::CloakState;
use glasnost_sim::combat::fire;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
use glasnost_sim::world::{ObjectId, Target, World};
use sw_common::*;

const DELAY: i32 = 7;
/// Cloaking stages and the cart's cloaking speed.
const STAGES: i32 = 20;
const SPEED: i32 = 2;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(DELAY);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    r.sim.general.cloaking_stages = Some(STAGES);
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    // The hold lasts DELAY ticks.
    r.globals.quirks.warp_hold_persists = true;
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.cloakable = Some(true);
    cart.combat.cloaking_speed = Some(SPEED);
    cart.combat.sensors = Some(true);
    cart.combat.sensors_sight = Some(2);
    cart.combat.ammo = Some(4);
    cart.combat.initial_ammo = Some(4);
    cart.combat.reload = Some(40);
    cart.combat.reload_increment = Some(0);
    cart.combat.pip_wrap = Some(0);
    cart.combat.empty_reload = Some(40);
    cart.mobile_fire = Some(true);
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

/// Launches the warp of the block around (0, 0) to (5, 5); the launch tick has been run.
fn launch(w: &mut World) {
    fire_now(w, ALPHA, SOURCE, at(0, 0));
    fire(w, ALPHA, WARP, at(5, 5));
    step(w);
}

/// A member under warp, with the arrival hold lasting `DELAY` ticks.
fn member(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    let m = spawn(&mut w, "CART", ALPHA, 0, 0);
    w.obj_mut(m).unwrap().sw.held_delay = DELAY;
    // The chrono building, charged: the tests set their states after this wait and launch at once.
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    (w, m)
}

/// Steps until the member's arrival hold begins (the stage becomes `Hold`), calling `each` before
/// every step whose update is suppressed (`Some(stage)` that is not a hold).
fn through_exclusion(w: &mut World, m: ObjectId, mut each: impl FnMut(&World, Stage)) {
    for _ in 0..300 {
        match stage(w, m) {
            Some(Stage::Hold(_)) => return,
            Some(s) => each(w, s),
            None => panic!("the hold was skipped"),
        }
        step(w);
    }
    panic!("never reached the hold");
}

// ---------------------------------------------------------------------------
// S-0516 claim 42 (warp suppression)

#[test]
fn s0516_claim_42_no_fade_step_through_arrival_effects_and_one_stage_in_the_hold() {
    // spec: S-0516 claim 42 (for warp members, ordinary cloak updates are suppressed from the first pending update through arrival effects inclusive; elapsed-tick stage countdowns continue to elapse but elapsed time alone advances no fade; no missed steps are replayed: the first ordinary update after the hold-end test advances at most one stage)
    let (mut w, m) = member(&rules());
    let now = w.tick;
    {
        let o = w.obj_mut(m).unwrap();
        o.cloak.state = CloakState::Cloaking;
        o.cloak.stage = 3;
        o.cloak.stepped = now;
    }
    launch(&mut w);
    let (s0, st0) = {
        let o = w.obj(m).unwrap();
        (o.cloak.stage, o.cloak.state)
    };
    assert_eq!(st0, CloakState::Cloaking);
    let mut updates = 0;
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!(
            (o.cloak.stage, o.cloak.state),
            (s0, CloakState::Cloaking),
            "no fade step before the hold"
        );
        updates += 1;
    });
    assert!(
        updates > 60,
        "the exclusion lasted through the leaving wait"
    );
    // The first hold turn: the countdown has long elapsed, yet only one stage is taken.
    step(&mut w);
    assert_eq!(
        w.obj(m).unwrap().cloak.stage,
        s0 + 1,
        "at most one stage, no burst"
    );
    // The countdown restarted: the next step is S ticks later, not the next tick.
    step(&mut w);
    assert_eq!(w.obj(m).unwrap().cloak.stage, s0 + 1);
    step(&mut w);
    assert_eq!(w.obj(m).unwrap().cloak.stage, s0 + 2);
}

#[test]
fn s0516_claim_42_a_hit_and_the_arrival_sensor_reveal_are_not_postponed() {
    // spec: S-0516 claim 42 (warp suppression does not postpone applicable immediate damage reactions, or the arrival sensor transfer and adjacency reveal of claims 13 and 22; starting an uncloak does not itself advance its fading stage; explicit uncloak causes change the cloak state when they occur)
    // A surviving hit after departure (a member in the leaving stage is being erased and takes no
    // hit at all, S-0302 claim 8).
    let (mut w, m) = member(&rules());
    {
        let o = w.obj_mut(m).unwrap();
        o.cloak.own = true;
    }
    glasnost_sim::cloak::machine::grant_own_flag(&mut w, m);
    {
        let o = w.obj_mut(m).unwrap();
        o.cloak.state = CloakState::Cloaked;
        o.cloak.stage = 0;
    }
    launch(&mut w);
    for _ in 0..200 {
        if stage(&w, m) == Some(Stage::Away) {
            break;
        }
        step(&mut w);
    }
    assert_eq!(stage(&w, m), Some(Stage::Away));
    assert_eq!(
        glasnost_sim::cloak::state(&w, m),
        CloakState::Cloaked,
        "no ordinary update ran"
    );
    let r = glasnost_sim::combat::damage::apply(
        &mut w,
        m,
        glasnost_sim::combat::damage::Hit {
            amount: 1,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    let _ = r;
    assert_eq!(
        glasnost_sim::cloak::state(&w, m),
        CloakState::Uncloaking,
        "the damage reaction takes effect at once"
    );
    let n = w.obj(m).unwrap().cloak.stage;
    assert_eq!(n, STAGES - 1, "starting the uncloak advances no stage");
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!((o.cloak.state, o.cloak.stage), (CloakState::Uncloaking, n));
    });
    step(&mut w);
    assert_eq!(
        w.obj(m).unwrap().cloak.stage,
        n - 1,
        "one stage on the first hold turn"
    );

    // The arrival sensor reveal beside an enemy sensor happens in the arrival-effects update,
    // before any ordinary fade work.
    let (mut w, m) = member(&rules());
    spawn(&mut w, "CART", BETA, 6, 5);
    {
        let o = w.obj_mut(m).unwrap();
        o.cloak.state = CloakState::Cloaked;
        o.cloak.stage = 0;
    }
    launch(&mut w);
    through_exclusion(&mut w, m, |w, s| {
        if matches!(s, Stage::Placed) {
            // The update about to run is the arrival effects: still fully cloaked until it does.
            assert_eq!(glasnost_sim::cloak::state(w, m), CloakState::Cloaked);
        }
    });
    let o = w.obj(m).unwrap();
    assert_eq!(
        o.cloak.state,
        CloakState::Uncloaking,
        "revealed at the arrival effects"
    );
    assert_eq!(o.cloak.stage, STAGES - 1, "with no ordinary fade step yet");
}

// ---------------------------------------------------------------------------
// S-0301 claims 11 and 13 (warp exclusion)

#[test]
fn s0301_claim_11_reload_countdown_elapses_but_no_round_is_granted_before_the_hold() {
    // spec: S-0301 claim 11 (warp exclusion: from the first pending warp update through arrival effects no ordinary reload work, so no round is granted and no countdown is restarted; elapsed time on an existing countdown still passes; at most one round in the first eligible hold turn, followed by the ordinary restart; no catch-up grants)
    let (mut w, m) = member(&rules());
    launch(&mut w);
    {
        let o = w.obj_mut(m).unwrap();
        o.combat.ammo = 2;
        o.combat.reload_left = Some(30);
    }
    let mut elapsed = 0;
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!(o.combat.ammo, 2, "no round granted");
        assert_eq!(
            o.combat.reload_left,
            Some(30 - elapsed),
            "the countdown keeps elapsing"
        );
        elapsed += 1;
    });
    assert!(30 - elapsed < 0, "it expired during the warp");
    step(&mut w);
    let o = w.obj(m).unwrap();
    assert_eq!(o.combat.ammo, 3, "one round on the first eligible turn");
    assert_eq!(o.combat.reload_left, Some(40), "and the ordinary restart");
    for _ in 0..4 {
        step(&mut w);
    }
    assert_eq!(w.obj(m).unwrap().combat.ammo, 3, "no catch-up grants");
}

#[test]
fn s0301_claim_11_a_never_started_countdown_waits_for_the_hold() {
    // spec: S-0301 claim 11 (warp exclusion; the never-started countdown counts as over, and its first grant likewise waits for an eligible hold turn)
    let (mut w, m) = member(&rules());
    launch(&mut w);
    {
        let o = w.obj_mut(m).unwrap();
        o.combat.ammo = 1;
        o.combat.reload_left = None;
    }
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!((o.combat.ammo, o.combat.reload_left), (1, None));
    });
    step(&mut w);
    assert_eq!(w.obj(m).unwrap().combat.ammo, 2);
}

#[test]
fn s0301_claim_13_firing_frames_stand_still_and_the_rearm_deadline_is_not_extended() {
    // spec: S-0301 claim 13 (warp exclusion: no firing-frame advancement from the first pending update through arrival effects, the counter retaining its value; the separate rearm countdown still elapses and is not extended; in the arrival hold only the normal single advancement per turn after the hold-end test, without catch-up decrements)
    let (mut w, m) = member(&rules());
    launch(&mut w);
    let deadline = w.tick + 30;
    {
        let o = w.obj_mut(m).unwrap();
        o.combat.firing_frame = Some(5);
        o.combat.rearm_until = deadline;
    }
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!(
            o.combat.firing_frame,
            Some(5),
            "the counter retains its value"
        );
        assert_eq!(
            o.combat.rearm_until, deadline,
            "the deadline is not extended"
        );
    });
    assert!(w.tick >= deadline, "the rearm countdown elapsed meanwhile");
    step(&mut w);
    assert_eq!(w.obj(m).unwrap().combat.firing_frame, Some(4));
    step(&mut w);
    assert_eq!(
        w.obj(m).unwrap().combat.firing_frame,
        Some(3),
        "one per turn"
    );
}

// ---------------------------------------------------------------------------
// S-0513 claims 51 and 52: the other ordinary work of the turn

#[test]
fn claim51_the_expected_health_bookkeeping_and_the_protection_phases_wait_for_the_hold() {
    // spec: S-0513 claim 51 and S-0203 claim 4 (the whole-member-turn exclusion also covers expected-health turn bookkeeping and protection-brightness phase processing, which resume after the hold-end test; elapsed protection duration is not suppressed; skipping the bookkeeping cancels no actual damage)
    let (mut w, m) = member(&rules());
    launch(&mut w);
    let t = w.tick;
    {
        let o = w.obj_mut(m).unwrap();
        o.combat.expected_health = -10;
        o.sw.protection = Some(Protection {
            start: t,
            duration: 10_000,
            kind: ProtectKind::Curtain,
            tint: Tint::default(),
        });
    }
    through_exclusion(&mut w, m, |w, _| {
        let o = w.obj(m).unwrap();
        assert_eq!(o.combat.expected_health, -10, "no bookkeeping");
        assert_eq!(
            o.sw.protection.unwrap().tint,
            Tint::BeforeFirstTurn,
            "no phase processing"
        );
        assert!(
            glasnost_sim::superweapons::protected(w, m),
            "the elapsed duration still counts"
        );
    });
    let mut advanced = false;
    let mut phased = false;
    for _ in 0..DELAY {
        step(&mut w);
        let o = w.obj(m).unwrap();
        advanced |= o.combat.expected_health > -10;
        phased |= o.sw.protection.unwrap().tint != Tint::BeforeFirstTurn;
    }
    assert!(advanced && phased, "both resume in the hold");
}

#[test]
fn claim51_a_queued_depot_entry_is_not_retried_before_the_hold() {
    // spec: S-0513 claim 51 and S-0314 claim 23 (the queued-entry retry is part of the ordinary vehicle turn and is suppressed through arrival effects; it resumes in the hold)
    let (mut w, m) = member(&rules());
    launch(&mut w);
    // A queued entry for a depot that is gone is cleared by the next retry.
    w.obj_mut(m).unwrap().depot.queued = Some(ObjectId(99_999));
    through_exclusion(&mut w, m, |w, _| {
        assert_eq!(w.obj(m).unwrap().depot.queued, Some(ObjectId(99_999)));
    });
    // The retry is skipped while chrono phase-in is active (S-0314 claim 23's own rule), so it
    // happens in the turn in which the hold expires.
    for _ in 0..DELAY + 2 {
        step(&mut w);
    }
    assert!(w.obj(m).unwrap().sw.warp.is_none());
    assert_eq!(
        w.obj(m).unwrap().depot.queued,
        None,
        "retried once the ordinary turn is back"
    );
}

// ---------------------------------------------------------------------------
// S-0614 claim 18 (warp)

fn drone_rules() -> Ruleset {
    let mut r = rules();
    let d = |s: &str| Decimal(s.into());
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("DRONE".into());
    let s = &mut r.sim;
    s.projectiles
        .insert("Hop".into(), ProjectileType::default());
    s.weapons.insert(
        "Latch".into(),
        WeaponType {
            damage: Some(DMG),
            rof: Some(R),
            range: Some(d("6")),
            speed: Some(60),
            projectile: Some("Hop".into()),
            warhead: Some("Grip".into()),
            limbo_launch: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Grip".into(),
        WarheadType {
            verses: Some((0..11).map(|_| d("1")).collect()),
            parasite: Some(true),
            paralyzes: Some(5),
            ..Default::default()
        },
    );
    let mut drone = UnitType {
        strength: Some(100),
        speed: Some(9),
        rot: Some(8),
        cost: Some(200),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Latch".into()),
        ..Default::default()
    };
    drone.combat.armor = Some(glasnost_rules::sim::ArmourClass::Light);
    s.vehicles.insert("DRONE".into(), drone);
    // A tough host.
    r.sim.vehicles.get_mut("CART").unwrap().strength = Some(100_000);
    r
}

const DMG: i32 = 10;
const R: i32 = 15;

#[test]
fn s0614_claim_18_an_attached_parasite_makes_no_ordinary_step_before_the_hold() {
    // spec: S-0614 claim 18 (for a vehicle under a superweapon warp, the ordinary attached-parasite step is suppressed from the first pending update through arrival effects, so no delivery or grappling advancement; during the arrival hold ordinary eligibility returns after the hold-end test, with no catch-up burst; a non-naval drone's release at departure is unresolved, so the attachment survives here) and S-0513 claim 51
    let mut w = game(&drone_rules());
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let host = spawn(&mut w, "CART", ALPHA, 0, 0);
    let p = spawn(&mut w, "DRONE", BETA, -2, 0);
    w.obj_mut(host).unwrap().sw.held_delay = DELAY;
    // Latch on by the drone's own shot, then run until it has attached.
    assert!(
        fire::shot(&mut w, p, 0, Target::Object(host)),
        "the latching shot was made"
    );
    for _ in 0..50 {
        if glasnost_sim::parasite::host_of(&w, p) == Some(host) {
            break;
        }
        step(&mut w);
    }
    assert_eq!(glasnost_sim::parasite::host_of(&w, p), Some(host));
    // Let a delivery happen so the cadence is established.
    run(&mut w, R + 2);
    assert!(
        w.obj(host).unwrap().health < 100_000,
        "it delivers in ordinary turns"
    );
    launch(&mut w);
    assert!(
        w.obj(host).unwrap().sw.warp.is_some(),
        "the host joined the warp"
    );
    let mut h = w.obj(host).unwrap().health;
    through_exclusion(&mut w, host, |w, _| {
        assert_eq!(
            w.obj(host).unwrap().health,
            h,
            "no delivery in a suppressed turn"
        );
        assert_eq!(
            glasnost_sim::parasite::host_of(w, p),
            Some(host),
            "still attached"
        );
        h = w.obj(host).unwrap().health;
    });
    // The first hold turn delivers once (its deadline passed long ago), then the cadence resumes.
    let h0 = w.obj(host).unwrap().health;
    step(&mut w);
    let h1 = w.obj(host).unwrap().health;
    assert!(h1 < h0, "an ordinary delivery on the first hold turn");
    assert!(h0 - h1 <= DMG, "one delivery, no burst of the skipped ones");
}
