//! Claim tests for magnetic lift revision 37 (S-0607, M8 step A3 intake 61;
//! answers Q-2160): a hold starts only with a shot (claim 3: no range test
//! at the detonation, the minimum range tested at the shot, the cleared
//! target after an arrival and a second seize after restoration) and a
//! spawner while seized and after restoration (claim 20: the slots cleared
//! as S-0506 claim 13 gives, the refill at the next pass, the kept
//! controller or pending target, the launch after restoration with no new
//! shot, the loss of a kept target), as scenario L19 lays them out.
//! Synthetic data only (the lift fixture of `lift_common` with a test
//! missile launcher and an invented rocket set); no stock value.
//! Ledger: `docs/m8-claims/intake61-S-0607-S-0306.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::lift;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::spawn::{ControllerState, SlotState, SpawnController};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use lift_common::*;

const HUMAN: HouseId = HouseId(0);

/// The lift fixture with L19's test types (all values invented):
/// - the lifter's beam gets a minimum range of 3 cells;
/// - a slow beam weapon SlowBeam (a straight, visible projectile of speed
///   8) on a second lifter type SLOWLIFT;
/// - rocket type RKT (set A, the missile-spawn flag) with long pause and
///   tilt durations (rail time 60 ticks);
/// - launcher LN1 (one RKT slot) and LN2 (two RKT slots), vehicles of the
///   seizable fixture size with the spawner weapon Launch (30 cells);
/// - the tower of the combat ruleset, unarmed, of strength 50000 and with a
///   short build-up sprite, so that it can be sold.
fn rules(tweak: impl FnOnce(&mut Ruleset)) -> Ruleset {
    let mut r = ruleset();
    r.sim.weapons.get_mut("MagBeam").unwrap().minimum_range = Some(d("3"));
    r.sim.weapons.insert(
        "SlowBeam".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(400),
            range: Some(d("12")),
            minimum_range: Some(d("3")),
            speed: Some(8),
            projectile: Some("Rocket".into()),
            warhead: Some("MAG".into()),
            is_mag_beam: Some(true),
            ..Default::default()
        },
    );
    vehicle(&mut r, "SLOWLIFT", |t| {
        t.strength = Some(1000);
        t.primary = Some("SlowBeam".into());
    });
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("RKT".into());
    r.sim.aircraft.insert(
        "RKT".into(),
        UnitType {
            strength: Some(40),
            speed: Some(50),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            ..Default::default()
        },
    );
    for (name, n) in [("LN1", 1), ("LN2", 2)] {
        vehicle(&mut r, name, |t| {
            t.strength = Some(2000);
            t.primary = Some("Launch".into());
            t.missile_spawn = Some(true);
            t.combat.spawns = Some("RKT".into());
            t.combat.spawns_number = Some(n);
            t.combat.spawn_regen_rate = Some(50);
            t.combat.spawn_reload_rate = Some(0);
        });
    }
    r.sim.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: Some(d("30")),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    let tower = r.sim.buildings.get_mut("TOWER").unwrap();
    tower.primary = None;
    tower.strength = Some(50000);
    // A build-up sprite of 10 frames, so that a sell order unbuilds it soon (S-0408 claim 3).
    tower.econ.buildup = Some(true);
    tower.econ.buildup_frames = Some(10);
    let g = &mut r.sim.general;
    g.v3_rocket_type = Some("RKT".into());
    g.v3_rocket_pause_frames = Some(30);
    g.v3_rocket_tilt_frames = Some(30);
    g.v3_rocket_altitude = Some(400);
    g.v3_rocket_pitch_final = Some(d("0.6"));
    g.v3_rocket_acceleration = Some(d("3"));
    g.v3_rocket_damage = Some(300);
    g.v3_rocket_elite_damage = Some(300);
    g.v3_warhead = Some("HE".into());
    g.v3_elite_warhead = Some("HE".into());
    tweak(&mut r);
    r
}

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

/// The human house's attack order (force-fire: every object here is the
/// human house's).
fn order_attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    open(w);
    let order = Order::Behaviour {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(target)),
        dest: None,
    };
    assert!(orders::issue(w, HUMAN, order), "the attack order is taken");
}

fn ctl(w: &World, id: ObjectId) -> SpawnController {
    w.obj(id).unwrap().spawn.clone().unwrap()
}

fn rockets(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.rocket.is_some() && o.on_map)
        .map(|o| o.id)
        .collect()
}

/// One tick; the shots `firer` made in it.
fn tick(w: &mut World, firer: ObjectId) -> usize {
    glasnost_sim::tick::step(w);
    std::mem::take(&mut w.effects)
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer: f, .. } if *f == firer))
        .count()
}

/// Steps until `f` holds (at most `limit` ticks), panicking otherwise;
/// counts the shots of `firer` on the way.
fn until(
    w: &mut World,
    firer: ObjectId,
    limit: i32,
    what: &str,
    mut f: impl FnMut(&World) -> bool,
) -> usize {
    let mut shots = 0;
    for _ in 0..limit {
        if f(w) {
            return shots;
        }
        shots += tick(w, firer);
    }
    assert!(f(w), "{what}: not within {limit} ticks");
    shots
}

/// The tower (house 0) at (8, 0), launcher `ln` (house 0) at (0, 0) and the
/// lifter (house 0) at (-6, 0), facing it.
struct Scene {
    w: World,
    lifter: ObjectId,
    ln: ObjectId,
    tower: ObjectId,
}

fn scene(ln: &str, tweak: impl FnOnce(&mut Ruleset)) -> Scene {
    let r = rules(tweak);
    let mut w = game(&r);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, at(8, 0));
    let ln = spawn(&mut w, Kind::Vehicle, ln, 0, at(0, 0));
    let lifter = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(-6, 0));
    // The launcher's slots fill at the first passes.
    until(&mut w, ln, 200, "the launcher's slots fill", |w| {
        ctl(w, ln)
            .slots
            .iter()
            .all(|s| s.state == SlotState::Docked)
    });
    Scene {
        w,
        lifter,
        ln,
        tower,
    }
}

/// Orders the launcher to attack the tower and steps until its fire event
/// (the spawner weapon's shot sets the pending target, S-0506 claim 1).
fn launcher_fires(s: &mut Scene) {
    order_attack(&mut s.w, s.ln, s.tower);
    let (w, ln) = (&mut s.w, s.ln);
    for _ in 0..200 {
        if tick(w, ln) > 0 {
            return;
        }
    }
    panic!("the launcher did not fire");
}

/// The seize of the launcher by the lifter's beam (lift_common's `beam`:
/// the beam is invisible and detonates at once).
fn seize(s: &mut Scene) {
    beam(&mut s.w, s.lifter, s.ln);
    assert!(lift::seized(&s.w, s.ln), "the launcher is seized");
}

/// Steps until the launcher is restored (S-0607 claim 20), with no launch
/// and no fire event on the way; the controller target at each tick is
/// passed to `watch`.
fn hold_until_restored(s: &mut Scene, mut watch: impl FnMut(&World)) {
    let (w, ln) = (&mut s.w, s.ln);
    let launches = ctl(w, ln).last_launch;
    for _ in 0..800 {
        if !lift::seized(w, ln) {
            return;
        }
        watch(w);
        // spec: S-0607 claims 10 and 20 (a seized spawner neither fires nor launches until restoration)
        assert_eq!(tick(w, ln), 0, "no fire event while seized");
        if lift::seized(w, ln) {
            assert_eq!(ctl(w, ln).last_launch, launches, "no launch while seized");
        }
    }
    panic!("the launcher was not restored");
}

// ---------------------------------------------------------------------------
// Claim 3: a hold starts only with a shot (L19 parts a, e and f)

// spec: S-0607 claim 3 (a hold starts only with a shot: the shot needs the firing check, whose range test includes the minimum range, S-0307 claim 9; the ordinary approach step before the grip; an arrival clears the lifter's target; a restored unit is an ordinary target that a later shot seizes again)
#[test]
fn l19a_minimum_range_refusal_approach_arrival_and_second_seize() {
    let r = rules(|_| {});
    let mut w = game(&r);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "LN1", 0, at(2, 0));
    let t = Target::Object(v);
    assert!(
        !glasnost_sim::approach::in_range(&w, l, t),
        "the launcher starts inside the minimum range"
    );
    order_attack(&mut w, l, v);
    let mut dest = false;
    let mut first = None;
    for _ in 0..400 {
        let shots = tick(&mut w, l);
        // The lifter fires after its movement in its turn and the launcher stands still, so the
        // distance at the end of the tick is the distance at the fire event.
        let inside = !glasnost_sim::approach::in_range(&w, l, t);
        dest |= w.obj(l).unwrap().dest.is_some();
        if shots > 0 {
            // spec: S-0607 claim 3 and S-0307 claim 9 (no fire event while the distance is below the minimum range)
            assert!(!inside, "a shot from inside the minimum range");
            first = Some(w.tick);
            break;
        }
        assert!(!lift::seized(&w, v), "a seize with no shot");
    }
    assert!(first.is_some(), "the lifter never fired");
    // spec: S-0215 claim 37 parts d and f (the ordinary approach step before the grip: an approach destination)
    assert!(dest, "the lifter took an approach destination");
    assert!(
        run_until(&mut w, 40, |w| lift::grips(w, l, v)),
        "the first shot seizes"
    );
    // The grip ends by arrival (claim 8): the lifter's target is then empty (claim 12 row 1).
    assert!(
        run_until(&mut w, 800, |w| !lift::grips(w, l, v)),
        "the hold ended"
    );
    assert!(lift::dropped(&w, v), "the hold ended by arrival");
    assert_eq!(
        w.obj(l).unwrap().target,
        None,
        "the arrival clears the lifter's target"
    );
    assert!(restored(&mut w, v), "the launcher is restored");
    // The restored launcher is an ordinary target: a new order gives a shot once the distance is
    // at least the minimum range, and a second seize.
    order_attack(&mut w, l, v);
    let mut second = false;
    for _ in 0..400 {
        let shots = tick(&mut w, l);
        let inside = !glasnost_sim::approach::in_range(&w, l, t);
        if shots > 0 {
            assert!(!inside, "a second shot from inside the minimum range");
            second = true;
            break;
        }
    }
    assert!(second, "the lifter never fired again");
    assert!(
        run_until(&mut w, 40, |w| lift::grips(w, l, v)),
        "the second shot seizes the restored launcher"
    );
}

/// The slow beam fired at `v` by SLOWLIFT `l` (the shot is made directly,
/// at a moment its range test passes) and run until it detonates; whether
/// the target was outside the range test (minimum range included) on the
/// ticks around the detonation.
fn slow_shot(w: &mut World, l: ObjectId, v: ObjectId) -> bool {
    let t = Target::Object(v);
    assert!(
        glasnost_sim::approach::in_range(w, l, t),
        "the shot passes its range test"
    );
    assert!(glasnost_sim::combat::fire::shot(w, l, 0, t));
    assert!(!w.projectiles.is_empty(), "the slow beam flies");
    let mut out = false;
    for _ in 0..800 {
        let before = !glasnost_sim::approach::in_range(w, l, t);
        run(w, 1);
        if w.projectiles.is_empty() {
            out = before && !glasnost_sim::approach::in_range(w, l, t);
            attack(w, l, v);
            return out;
        }
    }
    assert!(out, "the slow beam did not detonate");
    out
}

// spec: S-0607 claim 3 (no minimum-range test at the detonation: a unit outside the minimum range at the shot and inside it at the detonation is seized)
#[test]
fn l19e_inside_the_minimum_range_at_the_detonation_is_seized() {
    let r = rules(|_| {});
    let mut w = game(&r);
    let l = spawn(&mut w, Kind::Vehicle, "SLOWLIFT", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "LN1", 0, at(4, 0));
    open(&mut w);
    assert!(order_move(&mut w, v, at(-1, 0)));
    run(&mut w, 1);
    let out = slow_shot(&mut w, l, v);
    assert!(
        out,
        "the launcher was inside the minimum range at the detonation"
    );
    assert!(lift::grips(&w, l, v), "seized at the detonation");
}

// spec: S-0607 claim 3 (no test of the weapon's range at the detonation: a unit within range at the shot and beyond it at the detonation is seized)
#[test]
fn l19f_beyond_the_range_at_the_detonation_is_seized() {
    let r = rules(|_| {});
    let mut w = game(&r);
    let l = spawn(&mut w, Kind::Vehicle, "SLOWLIFT", 0, at(-10, 0));
    let v = spawn(&mut w, Kind::Vehicle, "LN1", 0, at(1, 0));
    open(&mut w);
    assert!(order_move(&mut w, v, at(18, 0)));
    let t = Target::Object(v);
    // The shot falls while the distance is at most the weapon's range.
    assert!(run_until(&mut w, 400, |w| {
        w.obj(v).is_some_and(|o| o.pos.cell().x > at(0, 0).x)
            && glasnost_sim::approach::in_range(w, l, t)
    }));
    let out = slow_shot(&mut w, l, v);
    assert!(
        out,
        "the launcher was beyond the weapon's range at the detonation"
    );
    assert!(lift::grips(&w, l, v), "seized at the detonation");
}

// ---------------------------------------------------------------------------
// Claim 20: a spawner while seized and after restoration (L19 parts b, c, d and g)

// spec: S-0607 claim 20 (a seize after the adopting pass and before the release: the controller target is kept, no fire and no launch while seized, the launch at it after restoration once not moving, with no new shot) and claim 5 (the docked rocket removed)
#[test]
fn l19b_kept_controller_target_launches_after_restoration() {
    let mut s = scene("LN1", |_| {});
    launcher_fires(&mut s);
    let (ln, tower) = (s.ln, s.tower);
    until(&mut s.w, ln, 20, "the adopting pass", |w| {
        ctl(w, ln).target == Some(Target::Object(tower))
    });
    let c = ctl(&s.w, ln);
    assert_eq!(c.state, ControllerState::Launching);
    assert_eq!(c.slots[0].state, SlotState::Docked, "before the launch");
    let docked = c.slots[0].object.unwrap();
    seize(&mut s);
    // spec: S-0607 claim 20 and S-0506 claim 13 (a docked rocket is removed)
    assert!(s.w.obj(docked).is_none(), "the docked rocket is removed");
    hold_until_restored(&mut s, |w| {
        // spec: S-0607 claim 20 (clearing the slots does not change the controller target)
        assert_eq!(
            ctl(w, ln).target,
            Some(Target::Object(tower)),
            "the controller target is kept"
        );
    });
    assert!(rockets(&s.w).is_empty());
    // After restoration, with no order given, it launches at the tower once not moving.
    let shots = until(&mut s.w, ln, 200, "a launch after restoration", |w| {
        !rockets(w).is_empty()
    });
    // spec: S-0607 claim 20 (with no new shot)
    assert_eq!(shots, 0, "no new fire event before the launch");
    assert!(
        !glasnost_sim::occupancy::in_progress(&s.w, ln),
        "not moving at the launch"
    );
    let x = rockets(&s.w)[0];
    assert_eq!(s.w.obj(x).unwrap().dest, Some(Target::Object(tower)));
}

// spec: S-0607 claim 20 (a kept target that leaves play while the spawner is seized is lost as S-0506 claim 25 gives: the controller target cleared with a reset; no launch follows restoration)
#[test]
fn l19b_sold_target_while_seized_resets_and_no_launch() {
    let mut s = scene("LN1", |_| {});
    launcher_fires(&mut s);
    let (ln, tower) = (s.ln, s.tower);
    until(&mut s.w, ln, 20, "the adopting pass", |w| {
        ctl(w, ln).target == Some(Target::Object(tower))
    });
    seize(&mut s);
    open(&mut s.w);
    assert!(orders::issue(&mut s.w, HUMAN, Order::Sell { actor: tower }));
    let mut gone = false;
    hold_until_restored(&mut s, |w| {
        if w.obj(tower).is_none() {
            gone = true;
            let c = ctl(w, ln);
            // spec: S-0506 claims 11 and 25 (a lost controller target is cleared, with a reset when no pending target waits)
            assert_eq!(c.target, None, "the lost target is cleared");
            assert_eq!(c.pending, None);
            assert_eq!(c.state, ControllerState::Idle, "a reset");
        }
    });
    assert!(gone, "the tower left play while the launcher was seized");
    glasnost_sim::testkit::run(&mut s.w, 200);
    assert!(rockets(&s.w).is_empty(), "no launch after restoration");
}

/// L19 part c's rail run: the seize while the rocket is on the rail.
fn rail_run(vanishes: bool) -> (Scene, i32, i32, Option<ObjectId>) {
    let mut s = scene("LN1", |r| {
        r.globals.quirks.rail_missile_vanishes_with_spawner = vanishes;
    });
    launcher_fires(&mut s);
    let ln = s.ln;
    until(&mut s.w, ln, 100, "the rocket on the rail", |w| {
        ctl(w, ln).slots[0].state == SlotState::OnRail
    });
    let x = ctl(&s.w, ln).slots[0].object;
    let health = s.w.obj(s.tower).unwrap().health;
    seize(&mut s);
    let next = ctl(&s.w, ln).next_pass;
    (s, health, next, x)
}

// spec: S-0607 claim 20 and S-0506 claim 13 (a rocket on the rail vanishes with no detonation, quirks.rail_missile_vanishes_with_spawner on; the emptied slot gets a countdown of 0 and is refilled at the next pass while still seized)
#[test]
fn l19c_rail_rocket_vanishes_and_the_slot_refills_at_the_next_pass() {
    let (mut s, health, next, x) = rail_run(true);
    let ln = s.ln;
    let x = x.unwrap();
    assert!(s.w.obj(x).is_none(), "the rail rocket vanished");
    assert_eq!(ctl(&s.w, ln).slots[0].state, SlotState::Empty);
    until(&mut s.w, ln, 20, "the next pass", |w| w.tick > next);
    assert!(lift::seized(&s.w, ln), "still seized at the next pass");
    // spec: S-0506 claim 9 (a countdown of 0: refilled at the next pass, seized or not)
    assert_eq!(
        ctl(&s.w, ln).slots[0].state,
        SlotState::Docked,
        "refilled at the next pass"
    );
    glasnost_sim::testkit::run(&mut s.w, 150);
    assert_eq!(s.w.obj(s.tower).unwrap().health, health, "no detonation");
}

// spec: S-0607 claim 20 and S-0506 claim 13 (quirks.rail_missile_vanishes_with_spawner off: the rail rocket is released as a preparing rocket is, and flies on)
#[test]
fn l19c_rail_rocket_released_with_the_quirk_off() {
    let (mut s, health, _, x) = rail_run(false);
    let x = x.unwrap();
    assert!(s.w.obj(x).is_some(), "the rail rocket is released");
    let tower = s.tower;
    assert!(
        run_until(&mut s.w, 300, |w| w.obj(tower).unwrap().health < health),
        "the released rocket detonates at its aim point"
    );
}

/// L19 part c's left-slot runs: the slot's refill tick with and without a
/// seize just after the rocket has left its slot, and whether the tower
/// was hit.
fn left_slot_run(seized: bool) -> (i32, bool) {
    let mut s = scene("LN1", |_| {});
    launcher_fires(&mut s);
    let ln = s.ln;
    until(&mut s.w, ln, 100, "the rocket on the rail", |w| {
        ctl(w, ln).slots[0].state == SlotState::OnRail
    });
    let x = ctl(&s.w, ln).slots[0].object.unwrap();
    until(&mut s.w, ln, 100, "the rocket left its slot", |w| {
        ctl(w, ln).slots[0].state == SlotState::Empty
    });
    assert!(s.w.obj(x).is_some(), "the rocket flies");
    let health = s.w.obj(s.tower).unwrap().health;
    if seized {
        seize(&mut s);
        assert!(
            s.w.obj(x).is_some(),
            "the seize does not touch a rocket that has left its slot"
        );
    }
    let mut refill = None;
    let mut hit = false;
    for _ in 0..400 {
        glasnost_sim::testkit::run(&mut s.w, 1);
        hit |= s.w.obj(s.tower).unwrap().health < health;
        if refill.is_none() && ctl(&s.w, ln).slots[0].state == SlotState::Docked {
            refill = Some(s.w.tick);
        }
        if hit && refill.is_some() {
            break;
        }
    }
    (refill.expect("the slot was refilled"), hit)
}

// spec: S-0607 claim 20 and S-0506 claim 13 (a rocket that has already left its slot is not affected: it flies on and detonates; a slot already empty at the seize keeps its own countdown, so it refills at the same pass as with no seize)
#[test]
fn l19c_left_slot_rocket_flies_on_and_the_refill_is_unchanged() {
    let (plain, plain_hit) = left_slot_run(false);
    let (with, with_hit) = left_slot_run(true);
    assert!(
        plain_hit && with_hit,
        "the rocket detonates at its aim point"
    );
    assert_eq!(
        with, plain,
        "the refill comes at the same pass as with no seize"
    );
}

/// L19 part d: the seize after the launcher's fire event and before the
/// controller's adopting pass.
fn pending_run(survives: bool) -> Scene {
    let mut s = scene("LN1", |r| {
        r.globals.quirks.spawner_pending_target_survives_idle_clear = survives;
    });
    // The fire event comes at least 3 ticks before the next pass, so that the seize precedes it.
    let (ln, tower) = (s.ln, s.tower);
    order_attack(&mut s.w, ln, tower);
    loop {
        let shot = tick(&mut s.w, ln) > 0;
        if shot {
            assert!(
                ctl(&s.w, ln).next_pass > s.w.tick + 2,
                "a fire event too close to the pass"
            );
            break;
        }
        assert!(s.w.tick < 300, "the launcher did not fire");
    }
    let c = ctl(&s.w, ln);
    assert_eq!(c.target, None, "before the adopting pass");
    assert_eq!(c.pending, Some(Target::Object(tower)));
    seize(&mut s);
    assert!(
        ctl(&s.w, ln).next_pass > s.w.tick,
        "the seize precedes the adopting pass"
    );
    s
}

// spec: S-0607 claim 20 and S-0506 claim 25 (quirks.spawner_pending_target_survives_idle_clear on: the seize's own-target clear leaves the pending target while the controller target is empty; the controller adopts it at a pass while seized, and the launcher launches at it once restored and not moving, with no new shot)
#[test]
fn l19d_pending_target_survives_and_is_adopted_while_seized() {
    let mut s = pending_run(true);
    let (ln, tower) = (s.ln, s.tower);
    assert_eq!(
        ctl(&s.w, ln).pending,
        Some(Target::Object(tower)),
        "the pending target survives"
    );
    let mut adopted = false;
    hold_until_restored(&mut s, |w| {
        adopted |= ctl(w, ln).target == Some(Target::Object(tower));
    });
    assert!(
        adopted,
        "the controller adopted the pending target while seized"
    );
    let shots = until(&mut s.w, ln, 200, "a launch after restoration", |w| {
        !rockets(w).is_empty()
    });
    assert_eq!(shots, 0, "no new fire event before the launch");
    assert!(
        !glasnost_sim::occupancy::in_progress(&s.w, ln),
        "not moving at the launch"
    );
}

// spec: S-0607 claim 20 and S-0506 claim 25 (quirks.spawner_pending_target_survives_idle_clear off: every explicit own-target clear empties the pending target, so nothing is adopted and nothing is launched after restoration)
#[test]
fn l19d_pending_target_cleared_with_the_quirk_off() {
    let mut s = pending_run(false);
    let ln = s.ln;
    assert_eq!(ctl(&s.w, ln).pending, None, "the pending target is cleared");
    hold_until_restored(&mut s, |w| {
        assert_eq!(ctl(w, ln).target, None, "nothing adopted");
    });
    glasnost_sim::testkit::run(&mut s.w, 200);
    assert!(rockets(&s.w).is_empty(), "no launch after restoration");
}

/// One run of L19 part d's reset case: a test vehicle of the human house
/// drives away from the launcher, which is ordered to fire at it `delay`
/// ticks later and is seized after its fire event and before the adopting
/// pass. The run counts only when the vehicle is out of range from where
/// the launcher then is at that pass, which comes while it is seized; the
/// scene after the pass is returned for a counted run. (The lift itself
/// takes the launcher only some 30 leptons up before that pass, too little
/// to leave the range on its own in any fixture here.)
fn out_of_range_at_the_pass(delay: i32) -> Option<Scene> {
    let mut s = scene("LN1", |r| {
        r.sim.weapons.get_mut("Launch").unwrap().range = Some(d("8"));
        r.sim.vehicles.get_mut("VICT").unwrap().speed = Some(40);
    });
    let ln = s.ln;
    let v = spawn(&mut s.w, Kind::Vehicle, "VICT", 0, at(0, 6));
    let tv = Target::Object(v);
    open(&mut s.w);
    assert!(order_move(&mut s.w, v, at(0, 18)));
    glasnost_sim::testkit::run(&mut s.w, delay);
    order_attack(&mut s.w, ln, v);
    let mut fired = false;
    for _ in 0..40 {
        if tick(&mut s.w, ln) > 0 {
            fired = true;
            break;
        }
    }
    let c = ctl(&s.w, ln);
    if !fired || c.pending != Some(tv) || c.next_pass <= s.w.tick + 2 {
        return None;
    }
    seize(&mut s);
    let pass = ctl(&s.w, ln).next_pass;
    // spec: S-0607 claim 20 and S-0506 claim 25 (the pending target survives the seize while the controller target is empty)
    assert_eq!(
        ctl(&s.w, ln).pending,
        Some(tv),
        "the pending target survives the seize"
    );
    let weapon = glasnost_sim::spawn::spawner_weapon(&s.w, ln).map(|(_, wp)| wp);
    while s.w.tick < pass {
        glasnost_sim::tick::step(&mut s.w);
        s.w.effects.clear();
    }
    let out = !glasnost_sim::combat::range::in_range(&s.w, ln, weapon, Some(tv), None);
    glasnost_sim::tick::step(&mut s.w);
    s.w.effects.clear();
    (out && lift::seized(&s.w, ln)).then_some(s)
}

// spec: S-0607 claim 20 and S-0506 claims 5 and 11 (the idle controller's range test at a pass while the spawner is seized, from where the spawner then is: a pending target out of range at the adopting pass gives a reset, and no launch follows restoration)
#[test]
fn l19d_out_of_range_at_the_adopting_pass_resets() {
    let mut counted = 0;
    for delay in 0..40 {
        let Some(mut s) = out_of_range_at_the_pass(delay) else {
            continue;
        };
        counted += 1;
        let ln = s.ln;
        let c = ctl(&s.w, ln);
        // spec: S-0506 claim 11 (the reset: no controller target, no pending target, idle)
        assert_eq!(
            (c.target, c.pending, c.state),
            (None, None, ControllerState::Idle),
            "a reset (delay {delay})"
        );
        hold_until_restored(&mut s, |_| {});
        glasnost_sim::testkit::run(&mut s.w, 200);
        assert!(
            rockets(&s.w).is_empty(),
            "no launch after the reset (delay {delay})"
        );
    }
    assert!(counted > 0, "no counted run");
}

// spec: S-0607 claim 20 and S-0506 claim 13 (two slots: the preparing rocket is released and flies on to its aim point, the docked one is removed with no launch; both slots empty, refilled at the next pass while seized)
#[test]
fn l19g_preparing_rocket_flies_on_docked_one_removed() {
    let mut s = scene("LN2", |_| {});
    launcher_fires(&mut s);
    let ln = s.ln;
    until(&mut s.w, ln, 40, "the first slot's launch", |w| {
        let c = ctl(w, ln);
        c.slots[0].state == SlotState::Preparing && c.slots[1].state == SlotState::Docked
    });
    let c = ctl(&s.w, ln);
    let (first, second) = (c.slots[0].object.unwrap(), c.slots[1].object.unwrap());
    let health = s.w.obj(s.tower).unwrap().health;
    seize(&mut s);
    assert!(s.w.obj(first).is_some(), "the preparing rocket is released");
    assert!(s.w.obj(second).is_none(), "the docked rocket is removed");
    let c = ctl(&s.w, ln);
    assert!(
        c.slots.iter().all(|x| x.state == SlotState::Empty),
        "both slots empty"
    );
    let next = c.next_pass;
    until(&mut s.w, ln, 20, "the next pass", |w| w.tick > next);
    assert!(lift::seized(&s.w, ln), "still seized at the next pass");
    assert!(
        ctl(&s.w, ln)
            .slots
            .iter()
            .all(|x| x.state == SlotState::Docked),
        "both slots refilled at the next pass"
    );
    let tower = s.tower;
    assert!(
        run_until(&mut s.w, 300, |w| w.obj(tower).unwrap().health < health),
        "the released rocket detonates at its aim point"
    );
    assert_eq!(
        ctl(&s.w, ln).last_launch,
        c.last_launch,
        "no launch while seized"
    );
}
