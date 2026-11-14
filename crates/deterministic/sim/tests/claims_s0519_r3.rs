//! Claim tests for S-0519 revision 3 (answers Q-0986): the arrival delay of a
//! charge planter's rearm (claim 13), the strike cancellation moment
//! (claim 19), a bomb with no bomb warhead (claim 22) and a suicide with no
//! demolition warhead (claim 23). Synthetic data only.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::bomb;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use terror_common::*;

fn sabotage(w: &mut World, name: &str, house: u8, target: ObjectId) -> ObjectId {
    let cell = w.obj(target).unwrap().pos.cell();
    let s = quiet(w, Kind::Infantry, name, house, cell);
    let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    let o = w.obj_mut(s).unwrap();
    o.beh.force(Behaviour::Sabotage, tick, stale);
    o.dest = Some(Target::Object(target));
    s
}

fn target_building(w: &mut World) -> ObjectId {
    quiet(w, Kind::Building, "BLDG", 1, at(0, 0))
}

// spec: S-0519 claim 13 (revision 3): arrival is not a shot; a fresh walker with the counter 0 and a positive second-weapon burst count (even 1) takes one inclusive ranged draw from 3 through 5, and the counter is neither advanced nor wrapped
#[test]
fn a_fresh_walker_takes_the_short_arrival_delay() {
    for refusal in [false, true] {
        let mut w = world(|_| {});
        let b = target_building(&mut w);
        if refusal {
            // The running-countdown row refuses the plant: every row rearms alike.
            w.obj_mut(b).unwrap().combat.delay_kill_until = Some(w.tick + 100);
        }
        let s = sabotage(&mut w, "SABO", 0, b);
        let mut r = w.rng.clone();
        let want = r.ranged(3, 5, w.rules().quirks.ranged_draw_half_span);
        assert_eq!(w.obj(s).unwrap().combat.burst, 0);
        assert!(bomb::charge_arrival(&mut w, s));
        let tick = w.tick;
        let o = w.obj(s).unwrap();
        assert_eq!(o.combat.rearm_delay, want, "refusal {refusal}");
        assert_eq!(o.combat.rearm_until, tick + want);
        assert_eq!(o.combat.burst, 0, "arrival does not advance the counter");
    }
}

// spec: S-0519 claim 13 (revision 3): with the counter at or above the second weapon's burst count the full delay of S-0301 claim 2 applies with one draw over 0 to 2, without berserk halving and without advancing the counter
#[test]
fn a_spent_burst_takes_the_full_arrival_delay_without_berserk_halving() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    {
        let o = w.obj_mut(s).unwrap();
        o.combat.burst = 1; // fired its primary before the sabotage: k is not below the burst count of 1
        o.combat.berserk = Some(1000);
    }
    let mut r = w.rng.clone();
    let u = r.ranged(0, 2, w.rules().quirks.ranged_draw_half_span);
    assert!(bomb::charge_arrival(&mut w, s));
    let o = w.obj(s).unwrap();
    assert_eq!(
        o.combat.rearm_delay,
        30 + u,
        "rate 30 plus the draw, not halved"
    );
    assert_eq!(o.combat.burst, 1, "not advanced or wrapped");
}

// spec: S-0519 claim 13 (revision 3): no second weapon gives 1 tick and no draw
#[test]
fn no_second_weapon_gives_one_tick() {
    let mut w = world(|r| terror_common::infantry(r, "BARE", |t| t.c4 = Some(true)));
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "BARE", 0, b);
    assert!(bomb::charge_arrival(&mut w, s));
    assert_eq!(w.obj(s).unwrap().combat.rearm_delay, 1);
}

// spec: S-0519 claim 13 (revision 3): a sonic second weapon gives its rate-of-fire field unchanged and takes no delay draw
#[test]
fn a_sonic_second_weapon_keeps_its_bare_rate() {
    let mut w = world(|r| {
        let mut g = r.sim.weapons["SaboGun"].clone();
        g.is_sonic = Some(true);
        g.rof = Some(44);
        r.sim.weapons.insert("SonicGun".into(), g);
        terror_common::infantry(r, "SONIC", |t| {
            t.c4 = Some(true);
            t.secondary = Some("SonicGun".into());
        });
    });
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SONIC", 0, b);
    assert!(bomb::charge_arrival(&mut w, s));
    assert_eq!(w.obj(s).unwrap().combat.rearm_delay, 44);
}

// spec: S-0301 claim 19 (revision 20, scenario F18) and claim 1: charge arrival replaces the rearm countdown without a shot even when a countdown already runs, and leaves the burst counter and ammunition alone
#[test]
fn arrival_replaces_a_running_countdown() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    let tick = w.tick;
    {
        let o = w.obj_mut(s).unwrap();
        o.combat.rearm_until = tick + 900;
        o.combat.rearm_delay = 900;
    }
    let mut r = w.rng.clone();
    let want = r.ranged(3, 5, w.rules().quirks.ranged_draw_half_span);
    let ammo = w.obj(s).unwrap().combat.ammo;
    assert!(bomb::charge_arrival(&mut w, s));
    let o = w.obj(s).unwrap();
    assert_eq!(o.combat.rearm_delay, want, "replaced, not kept or extended");
    assert_eq!(o.combat.rearm_until, tick + want);
    assert_eq!((o.combat.burst, o.combat.ammo), (0, ammo));
}

fn strike_setup() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = world(|_| {});
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(0, 0));
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(6, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(8, 0));
    run_to(&mut w, 100);
    designate(&mut w, d, a);
    assert!(!linked(&w, d).is_empty());
    assert!(w.strike.strikes[&d].active);
    (w, d, a, b)
}

// spec: S-0519 claim 19 (revision 3): a change of the designator's target cancels its strike when the change applies (an order's application at the end of a tick), not at the strike's next look or the designator's next turn
#[test]
fn an_order_changing_the_target_cancels_in_its_own_tick() {
    let (mut w, d, _a, b) = strike_setup();
    let before = linked(&w, d);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: d,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(b)),
            dest: None,
        }
    ));
    orders::apply_pending(&mut w);
    let s = &w.strike.strikes[&d];
    assert!(
        !s.active && s.target.is_none(),
        "cancelled at the order boundary"
    );
    for x in before {
        let o = w.obj(x).unwrap();
        assert!(o.target.is_none(), "the aircraft forgot their target");
    }
}

// spec: S-0519 claim 19 (revision 3): a change from A to B and back to A within one tick still cancels; restoring A does not undo it
#[test]
fn a_change_and_back_still_cancels() {
    let (mut w, d, a, b) = strike_setup();
    let had = w.obj(d).unwrap().target;
    assert_eq!(had, Some(Target::Object(a)));
    {
        let o = w.obj_mut(d).unwrap();
        glasnost_sim::spawn::set_own_target(o, Some(Target::Object(b)));
        glasnost_sim::spawn::set_own_target(o, Some(Target::Object(a)));
    }
    assert_eq!(w.obj(d).unwrap().target, had);
    glasnost_sim::strike::sweep(&mut w);
    let s = &w.strike.strikes[&d];
    assert!(!s.active && s.target.is_none());
}

// spec: S-0519 claim 19 (revision 3): the same target does not cancel
#[test]
fn the_same_target_does_not_cancel() {
    let (mut w, d, a, _b) = strike_setup();
    glasnost_sim::spawn::set_own_target(w.obj_mut(d).unwrap(), Some(Target::Object(a)));
    glasnost_sim::strike::sweep(&mut w);
    assert!(w.strike.strikes[&d].active);
}

// spec: S-0519 claim 19 (revision 3): the sweep after a turn finds the one designator whose target changed among several whose targets did not (#409: the look before the cancels)
#[test]
fn the_sweep_finds_a_change_behind_unchanged_designators() {
    let mut w = world(|_| {});
    let d1 = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(0, 0));
    let d2 = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(0, 2));
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(6, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(8, 0));
    run_to(&mut w, 100);
    designate(&mut w, d1, a);
    designate(&mut w, d2, a);
    assert!(w.strike.strikes[&d1].active && w.strike.strikes[&d2].active);
    glasnost_sim::strike::sweep(&mut w);
    assert!(w.strike.strikes[&d1].active && w.strike.strikes[&d2].active);
    let (lo, hi) = if d1 < d2 { (d1, d2) } else { (d2, d1) };
    glasnost_sim::spawn::set_own_target(w.obj_mut(hi).unwrap(), Some(Target::Object(b)));
    glasnost_sim::strike::sweep(&mut w);
    assert!(
        w.strike.strikes[&lo].active,
        "the unchanged one keeps its strike"
    );
    let s = &w.strike.strikes[&hi];
    assert!(
        !s.active && s.target.is_none(),
        "the changed one is cancelled"
    );
}

// spec: S-0519 claim 22 (revision 3): a bomb with no bomb warhead ends normally, stops ticking, detaches from its carrier and makes no area hits
#[test]
fn a_bomb_with_no_warhead_ends_without_a_blast() {
    let mut w = world(|r| r.sim.general.ivan_warhead = None);
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-8, -8));
    assert!(shoot(&mut w, p, v));
    let t = w.obj(v).unwrap().combat.bomb.expect("planted").planted;
    run_to(&mut w, t + 60);
    assert!(
        w.obj(v).unwrap().combat.bomb.is_none(),
        "ended and detached"
    );
    assert_eq!(health(&w, v), 100_000);
    assert_eq!(health(&w, near), 100_000);
}

// spec: S-0519 claim 23 (revision 3): a suicide with no demolition warhead still takes its self-hit, bypassing defences; our engine completes the death, and the death weapon still detonates with its own valid warhead
#[test]
fn a_suicide_without_a_demolition_warhead_still_dies_and_its_death_weapon_fires() {
    let mut w = world(|r| r.sim.general.c4_warhead = None);
    let k = quiet(&mut w, Kind::Infantry, "KAMI", 0, at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        k,
        0,
        Target::Object(near)
    ));
    testkit::run(&mut w, 2);
    assert!(w.obj(k).is_none(), "the firer is dead");
    assert_eq!(health(&w, near), 100_000 - 80, "the death weapon fired");
}
