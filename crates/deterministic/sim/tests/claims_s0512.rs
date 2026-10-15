//! Claim tests of S-0512 (superweapon charge and readiness) on synthetic
//! rules (`tests/sw_common`): grants, suspension, loss, completion, firing
//! and recharge, the two stages, the trigger actions on records, drain and
//! manual modes, the sidebar figures and the import.

mod sw_common;

use glasnost_sim::superweapons::charge::{self, Label};
use glasnost_sim::superweapons::{DrainState, SwEvent};
use sw_common::*;

/// The tick in which the record became granted, running ticks until it is.
fn grant_tick(w: &mut glasnost_sim::world::World, h: glasnost_sim::world::HouseId, i: u32) -> i32 {
    until(w, 20, |w| rec(w, h, i).granted)
}

/// A game with Alpha's IRONB placed and its CURTAIN record granted.
fn curtain_game() -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    i32,
) {
    let r = ruleset();
    let mut w = game(&r);
    let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
    let g = grant_tick(&mut w, ALPHA, CURTAIN);
    (w, b, g)
}

// --- Claim 1 -----------------------------------------------------------------

#[test]
fn claim1_records_start_ungranted_and_are_per_house() {
    // spec: S-0512 claim 1 (independent per house; not granted, ready or suspended before a grant)
    let r = ruleset();
    let mut w = game(&r);
    let a = rec(&w, ALPHA, CURTAIN);
    assert!(!a.granted && !a.ready && !a.suspended);
    spawn(&mut w, "IRONB", ALPHA, 0, 0);
    grant_tick(&mut w, ALPHA, CURTAIN);
    assert!(rec(&w, ALPHA, CURTAIN).granted);
    assert!(!rec(&w, BETA, CURTAIN).granted);
    assert!(!rec(&w, ALPHA, STORM).granted);
}

// --- Claims 2, 3 and 5 -------------------------------------------------------

#[test]
fn claim2_grant_takes_effect_at_a_recheck_only() {
    // spec: S-0512 claim 2 (a) (a building that comes and goes between rechecks changes nothing)
    let r = ruleset();
    let mut w = game(&r);
    let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
    // Removing the building before any house update: no grant ever.
    remove(&mut w, b);
    run(&mut w, 5);
    assert!(!rec(&w, ALPHA, CURTAIN).granted);
}

#[test]
fn claim3_and_5_grant_sets_t_and_readiness_comes_t_ticks_later() {
    // spec: S-0512 claim 3 (the grant sets remaining T and starts the clock, no announcement); claim 5 (ready in tick g + T, with the ready announcement for its own house)
    let (mut w, _, _) = curtain_game();
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(x.running && !x.ready);
    assert_eq!(x.remain, T);
    let g = x.start;
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    let ready = rec(&w, ALPHA, CURTAIN).ready_tick;
    assert_eq!(ready - g, T, "ready in tick g + T");
    let ev = events(&w);
    assert!(ev.contains(&(SwEvent::Ready, Some(ALPHA), Some(CURTAIN))));
}

#[test]
fn claim3_grant_takes_no_draw_and_no_announcement() {
    // spec: S-0512 claim 3 (no announcement; the grant takes no random draw)
    let r = ruleset();
    let mut a = game(&r);
    let mut b = game(&r);
    spawn(&mut a, "IRONB", ALPHA, 0, 0);
    spawn(&mut b, "AUXB", ALPHA, 0, 0);
    for _ in 0..5 {
        step(&mut a);
        step(&mut b);
        assert!(events(&a).is_empty());
    }
    assert!(rec(&a, ALPHA, CURTAIN).granted);
    assert_eq!(a.rng, b.rng, "the grant took no draw");
}

#[test]
fn claim3_disableable_type_not_granted_with_setting_off() {
    // spec: S-0512 claim 3 (a disableable superweapon is not granted while the superweapon setting is off)
    let r = ruleset();
    let mut w = game(&r);
    w.econ.settings.superweapons = Some(false);
    spawn(&mut w, "TIMEB", ALPHA, 0, 0);
    spawn(&mut w, "AUXB", ALPHA, 2, 0);
    run(&mut w, 5);
    assert!(!rec(&w, ALPHA, TIMED).granted);
    w.econ.settings.superweapons = Some(true);
    spawn(&mut w, "AUXB", ALPHA, 4, 0);
    run(&mut w, 3);
    assert!(rec(&w, ALPHA, TIMED).granted);
}

#[test]
fn claim3_auxiliary_building_required_for_own_fields() {
    // spec: S-0512 Terms (giving building: with an auxiliary building type, the house must own one)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "TIMEB", ALPHA, 0, 0);
    run(&mut w, 5);
    assert!(!rec(&w, ALPHA, TIMED).granted);
    spawn(&mut w, "AUXB", ALPHA, 2, 0);
    run(&mut w, 3);
    assert!(rec(&w, ALPHA, TIMED).granted);
}

// --- Claim 2 (b), 4 and 6 ----------------------------------------------------

#[test]
fn claim4_grant_under_low_power_starts_suspended_even_if_not_power_dependent() {
    // spec: S-0512 claim 4 (a grant while the house has low power starts suspended with remaining T, power-dependent or not)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "HOG", ALPHA, -4, 0);
    run(&mut w, 3);
    spawn(&mut w, "TIMEB", ALPHA, 0, 0);
    spawn(&mut w, "AUXB", ALPHA, 2, 0);
    grant_tick(&mut w, ALPHA, TIMED);
    let x = rec(&w, ALPHA, TIMED);
    assert!(x.suspended && !x.running);
    assert_eq!(x.remain, T);
    run(&mut w, T + 10);
    assert!(
        !rec(&w, ALPHA, TIMED).ready,
        "stays suspended until powered"
    );
}

#[test]
fn claim4_quirk_off_not_power_dependent_type_charges() {
    // spec: S-0512 claim 4 (quirks.superweapon_grant_suspends_by_house_power off: a type that is not power-dependent always starts charging)
    let mut r = ruleset();
    r.globals.quirks.superweapon_grant_suspends_by_house_power = false;
    let mut w = game(&r);
    spawn(&mut w, "HOG", ALPHA, -4, 0);
    run(&mut w, 3);
    spawn(&mut w, "TIMEB", ALPHA, 0, 0);
    spawn(&mut w, "AUXB", ALPHA, 2, 0);
    grant_tick(&mut w, ALPHA, TIMED);
    let x = rec(&w, ALPHA, TIMED);
    assert!(!x.suspended && x.running);
}

#[test]
fn claim2b_and_6_low_power_suspends_and_full_power_resumes() {
    // spec: S-0512 claim 2 (b) and claim 6 (a change to low power suspends a power-dependent record at once; a suspension of d ticks delays readiness by d)
    let (mut w, _, g) = curtain_game();
    run(&mut w, 10);
    let hog = spawn(&mut w, "HOG", ALPHA, -4, 0);
    until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
    let s = w.tick;
    let kept = rec(&w, ALPHA, CURTAIN).remain;
    run(&mut w, 25);
    assert_eq!(rec(&w, ALPHA, CURTAIN).remain, kept, "the clock is stopped");
    remove(&mut w, hog);
    until(&mut w, 10, |w| !rec(w, ALPHA, CURTAIN).suspended);
    let back = w.tick;
    let ready = ready_tick(&mut w, ALPHA, CURTAIN, 300);
    let d = back - s;
    assert_eq!(
        ready - g,
        T + d,
        "readiness delayed by exactly the suspension"
    );
}

#[test]
fn claim6_switching_off_every_giving_building_suspends() {
    // spec: S-0512 claim 6 (powered: at least one giving building switched on and no low power; a switch-off is judged at the next check)
    let (mut w, b, _) = curtain_game();
    let b2 = spawn(&mut w, "IRONB", ALPHA, 3, 0);
    run(&mut w, 2);
    switch(&mut w, b, false);
    run(&mut w, 3);
    assert!(
        !rec(&w, ALPHA, CURTAIN).suspended,
        "another giving building is on"
    );
    switch(&mut w, b2, false);
    until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
    switch(&mut w, b, true);
    until(&mut w, 10, |w| !rec(w, ALPHA, CURTAIN).suspended);
    assert!(rec(&w, ALPHA, CURTAIN).running);
}

#[test]
fn claim6_ready_record_suspended_stays_ready_but_cannot_fire() {
    // spec: S-0512 claim 6 (a ready record can be suspended: it stays ready but normally cannot fire)
    let (mut w, b, _) = curtain_game();
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    switch(&mut w, b, false);
    until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
    assert!(rec(&w, ALPHA, CURTAIN).ready);
    assert_eq!(charge::label(&w, ALPHA, CURTAIN), Label::OnHold);
    assert!(!charge::offers_targeting(&w, ALPHA, CURTAIN));
    fire_now(&mut w, ALPHA, CURTAIN, at(5, 5));
    assert!(rec(&w, ALPHA, CURTAIN).ready, "the order did nothing");
}

// --- Claim 2 (c) and (d), claim 7 ---------------------------------------------

#[test]
fn claim2c_upgrade_gained_grants_at_once() {
    // spec: S-0512 claim 2 (c) and Terms (an upgrade naming the type gives it; the grant check at once)
    let r = ruleset();
    let mut w = game(&r);
    let base = spawn(&mut w, "BASE", ALPHA, 0, 0);
    run(&mut w, 3);
    let upg = w
        .rules()
        .index_of(glasnost_sim::rules::Kind::Building, "UPG")
        .unwrap();
    glasnost_sim::econ::placement::add_upgrade(&mut w, base, upg);
    assert!(
        rec(&w, ALPHA, CURTAIN).granted,
        "granted in the same moment"
    );
}

#[test]
fn claim2d_removal_of_giving_building_loses_at_once() {
    // spec: S-0512 claim 2 (d) and claim 7 (a giving building removed: the check at once; no giving building: lost)
    let (mut w, b, _) = curtain_game();
    remove(&mut w, b);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(!x.granted && !x.ready);
}

#[test]
fn claim7_loss_forgets_the_charge() {
    // spec: S-0512 claim 7 (the charge is forgotten: a new grant starts from the full T)
    let (mut w, b, _) = curtain_game();
    run(&mut w, 40);
    remove(&mut w, b);
    spawn(&mut w, "IRONB", ALPHA, 2, 2);
    grant_tick(&mut w, ALPHA, CURTAIN);
    let x = rec(&w, ALPHA, CURTAIN);
    assert_eq!(x.remain, T);
}

#[test]
fn claim7_lost_suspended_record_stays_marked_suspended() {
    // spec: S-0512 claim 7 (a suspended record that is lost stays marked suspended)
    let (mut w, b, _) = curtain_game();
    switch(&mut w, b, false);
    until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
    remove(&mut w, b);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(!x.granted && x.suspended);
}

#[test]
fn claim2d_upgrade_second_field_removal_waits() {
    // spec: S-0512 claim 2 (d) (quirks.superweapon_upgrade_second_removal_waits: an upgrade naming only a second superweapon makes no immediate check)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_upgrade_second_removal_waits = quirk;
        let mut w = game(&r);
        let base = spawn(&mut w, "BASE", ALPHA, 0, 0);
        run(&mut w, 3);
        let upg2 = w
            .rules()
            .index_of(glasnost_sim::rules::Kind::Building, "UPG2")
            .unwrap();
        glasnost_sim::econ::placement::add_upgrade(&mut w, base, upg2);
        assert!(rec(&w, ALPHA, CURTAIN).granted);
        w.obj_mut(base).unwrap().econ.building.upgrades = [None; 3];
        charge::upgrade_lost(&mut w, base, upg2);
        assert_eq!(rec(&w, ALPHA, CURTAIN).granted, quirk, "quirk {quirk}");
    }
}

// --- Claim 8 ---------------------------------------------------------------

#[test]
fn claim8_fire_consumes_and_recharges_from_t() {
    // spec: S-0512 claim 8 (carried out: not ready, the clock restarts with T; ready again in tick L + T)
    let (mut w, _, _) = curtain_game();
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    fire(&mut w, ALPHA, CURTAIN, at(5, 5));
    let l = w.tick;
    step(&mut w);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(!x.ready && x.running && x.remain == T);
    assert_eq!(x.start, l);
    let again = ready_tick(&mut w, ALPHA, CURTAIN, 200);
    assert_eq!(
        again,
        l + T + 1,
        "ready in the house update of L + T, seen after that tick ran"
    );
}

#[test]
fn claim8_unready_order_does_nothing() {
    // spec: S-0512 claim 8 (otherwise it does nothing)
    let (mut w, _, _) = curtain_game();
    run(&mut w, 5);
    let before = rec(&w, ALPHA, CURTAIN);
    fire_now(&mut w, ALPHA, CURTAIN, at(5, 5));
    let after = rec(&w, ALPHA, CURTAIN);
    assert_eq!(before.remain, after.remain);
    assert_eq!(before.start, after.start);
}

#[test]
fn claim8_storm_refusal_keeps_charge_and_ready() {
    // spec: S-0512 claim 8 (a weather order refused while a storm is pending or raging keeps the charge and ready state; a refusal message)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "WEATHERB", ALPHA, 0, 0);
    spawn(&mut w, "WEATHERB", BETA, 4, 4);
    ready_tick(&mut w, ALPHA, STORM, 200);
    until(&mut w, 10, |w| rec(w, BETA, STORM).ready);
    fire_now(&mut w, ALPHA, STORM, at(5, 5));
    assert!(!rec(&w, ALPHA, STORM).ready);
    // The storm itself is S-0513; set it pending here so only the record rule is tested.
    w.sw.storm.state = glasnost_sim::superweapons::StormState::Counting;
    w.effects.clear();
    fire_now(&mut w, BETA, STORM, at(-5, -5));
    assert!(rec(&w, BETA, STORM).ready, "refused: still ready");
    assert!(events(&w).contains(&(SwEvent::Refused, Some(BETA), Some(STORM))));
}

#[test]
fn claim8_missile_without_launcher() {
    // spec: S-0512 claim 8 (quirks.missile_without_launcher_consumes_charge: a missile with no launching building consumes its charge; off: refused, stays ready)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.missile_without_launcher_consumes_charge = quirk;
        // A non-silo giving building, so no building can launch.
        r.sim
            .buildings
            .get_mut("SHAREB")
            .unwrap()
            .econ
            .super_weapon2 = Some("STRIKE".into());
        let mut w = game(&r);
        spawn(&mut w, "SHAREB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, STRIKE, 200);
        fire_now(&mut w, ALPHA, STRIKE, at(5, 5));
        assert_eq!(rec(&w, ALPHA, STRIKE).ready, !quirk, "quirk {quirk}");
    }
}

#[test]
fn claim8_suspended_running_bypass() {
    // spec: S-0512 claim 8 (quirks.superweapon_order_tests_clock: a ready suspended record whose clock runs accepts an order, consuming readiness without restarting the clock; off: refused)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_order_tests_clock = quirk;
        let mut w = game(&r);
        let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, CURTAIN, 200);
        switch(&mut w, b, false);
        until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
        // Claim 14 restarts the clock through the suspension.
        charge::set_charge(&mut w, ALPHA, CURTAIN, 50);
        let before = rec(&w, ALPHA, CURTAIN);
        assert!(before.ready && before.running && before.suspended);
        fire_now(&mut w, ALPHA, CURTAIN, at(5, 5));
        let after = rec(&w, ALPHA, CURTAIN);
        assert_eq!(after.ready, !quirk, "quirk {quirk}");
        if quirk {
            assert_eq!((after.start, after.remain), (before.start, before.remain));
        }
    }
}

// --- Claim 9 ---------------------------------------------------------------

#[test]
fn claim9_firing_touches_no_other_record() {
    // spec: S-0512 claim 9 (firing one record changes no other record)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "IRONB", ALPHA, 0, 0);
    spawn(&mut w, "WEATHERB", ALPHA, 3, 0);
    spawn(&mut w, "IRONB", BETA, -5, -5);
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    run(&mut w, 2);
    let other = rec(&w, ALPHA, STORM);
    let beta = rec(&w, BETA, CURTAIN);
    fire_now(&mut w, ALPHA, CURTAIN, at(8, 8));
    assert_eq!(rec(&w, ALPHA, STORM), other);
    assert_eq!(rec(&w, BETA, CURTAIN), beta);
}

// --- Claims 10 and 11 ----------------------------------------------------------

#[test]
fn claim10_source_pick_keeps_charge_and_is_silent() {
    // spec: S-0512 claim 10 (a source pick does not consume the charge; ready again at the next house update; our engine repeats no announcement)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, SOURCE, 200);
    let ready = rec(&w, ALPHA, SOURCE).ready_tick;
    fire(&mut w, ALPHA, SOURCE, at(3, 3));
    step(&mut w);
    let x = rec(&w, ALPHA, SOURCE);
    assert!(!x.ready);
    assert_eq!(x.source, Some(at(3, 3)));
    w.effects.clear();
    step(&mut w);
    let y = rec(&w, ALPHA, SOURCE);
    assert!(y.ready, "ready again in the next house update");
    assert_eq!(y.ready_tick, ready, "the flash window is not restarted");
    assert!(!events(&w).iter().any(|e| e.0 == SwEvent::Ready));
}

#[test]
fn claim10_second_stage_restarts_first_stage() {
    // spec: S-0512 claim 10 (the second stage uses the last source and restarts the first stage at T; the second-stage record is untouched)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, SOURCE, 200);
    fire_now(&mut w, ALPHA, SOURCE, at(3, 3));
    let warp_before = rec(&w, ALPHA, WARP);
    fire(&mut w, ALPHA, WARP, at(-3, -3));
    let l = w.tick;
    step(&mut w);
    let x = rec(&w, ALPHA, SOURCE);
    assert!(!x.ready && x.running && x.remain == T && x.start == l);
    assert_eq!(rec(&w, ALPHA, WARP), warp_before);
}

#[test]
fn claim10_second_stage_unchecked_quirk() {
    // spec: S-0512 claim 10 (quirks.second_stage_order_unchecked off: refused unless the first stage is granted, not suspended and has an unconsumed source)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.second_stage_order_unchecked = quirk;
        let mut w = game(&r);
        spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, SOURCE, 200);
        fire_now(&mut w, ALPHA, SOURCE, at(3, 3));
        fire_now(&mut w, ALPHA, WARP, at(-3, -3));
        run(&mut w, 3);
        // A second warp with the source already used.
        let before = rec(&w, ALPHA, SOURCE);
        fire_now(&mut w, ALPHA, WARP, at(-4, -4));
        let after = rec(&w, ALPHA, SOURCE);
        assert_eq!(after.start != before.start, quirk, "quirk {quirk}");
    }
}

#[test]
fn claim10_suspended_first_stage_restart_runs_through_suspension() {
    // spec: S-0512 claim 10 (quirks.superweapon_restart_ignores_suspension: the restart runs even when the first stage is suspended; off: stopped at T)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_restart_ignores_suspension = quirk;
        let mut w = game(&r);
        let b = spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, SOURCE, 200);
        fire_now(&mut w, ALPHA, SOURCE, at(3, 3));
        switch(&mut w, b, false);
        until(&mut w, 10, |w| rec(w, ALPHA, SOURCE).suspended);
        fire_now(&mut w, ALPHA, WARP, at(-3, -3));
        let x = rec(&w, ALPHA, SOURCE);
        assert_eq!(x.running, quirk);
        assert_eq!(x.remain, T);
    }
}

#[test]
fn claim11_link_as_list_position_or_kind() {
    // spec: S-0512 claim 11 (quirks.predependent_kind_as_list_position: the kind number as a list position; off: the first type of that kind)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.predependent_kind_as_list_position = quirk;
        // The link names the storm kind (kind number 2); list position 2 is
        // STORM, and the first type of the weather kind is STORM too, so
        // instead link the missile kind (0): position 0 is STRIKE, and the
        // first missile-kind type is STRIKE as well. Make them differ:
        // relabel STORM (position 2) as a missile-kind type.
        r.sim.superweapons.get_mut("WARP").unwrap().pre_dependent = Some("LightningStorm".into());
        r.sim.superweapons.get_mut("STORM").unwrap().kind = Some("IronCurtain".into());
        let mut w = game(&r);
        spawn(&mut w, "WEATHERB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, STORM, 200);
        // Give STORM a source to use: a one-time first stage is not needed,
        // the record's source is set by hand.
        glasnost_sim::superweapons::record_mut(&mut w, ALPHA, STORM)
            .unwrap()
            .source = Some(at(2, 2));
        fire_now(&mut w, ALPHA, WARP, at(-3, -3));
        let x = rec(&w, ALPHA, STORM);
        // With the quirk, STORM (position 2) is the first stage and restarts.
        assert_eq!(!x.ready, quirk, "quirk {quirk}");
    }
}

// --- Claim 12 ----------------------------------------------------------------

#[test]
fn claim12_one_time_grant_ready_at_once_and_revoked_on_fire() {
    // spec: S-0512 claim 12 (granted as one-time and ready at once, its ready tick that tick, no announcement; revoked when it fires)
    let r = ruleset();
    let mut w = game(&r);
    charge::one_time_grant(&mut w, ALPHA, CURTAIN);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(x.granted && x.one_time && x.ready);
    assert_eq!(x.ready_tick, w.tick);
    assert!(events(&w).is_empty());
    fire_now(&mut w, ALPHA, CURTAIN, at(5, 5));
    let y = rec(&w, ALPHA, CURTAIN);
    assert!(!y.granted && !y.ready);
}

#[test]
fn claim12_one_time_grant_on_granted_record_does_nothing() {
    // spec: S-0512 claim 12 (a one-time grant on a granted record does nothing: a charging record stays charging)
    let (mut w, _, _) = curtain_game();
    let before = rec(&w, ALPHA, CURTAIN);
    charge::one_time_grant(&mut w, ALPHA, CURTAIN);
    assert_eq!(rec(&w, ALPHA, CURTAIN), before);
}

#[test]
fn claim12_one_time_not_lost_by_checks() {
    // spec: S-0512 claim 12 (a one-time record is not suspended or lost by building and power checks)
    let r = ruleset();
    let mut w = game(&r);
    charge::one_time_grant(&mut w, ALPHA, CURTAIN);
    spawn(&mut w, "HOG", ALPHA, 0, 0);
    run(&mut w, 5);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(x.granted && x.ready && !x.suspended);
}

#[test]
fn claim12_stale_suspension_quirk() {
    // spec: S-0512 claim 12 (quirks.one_time_grant_keeps_stale_suspension: a record lost while suspended stays suspended after a one-time grant; off: ended)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.one_time_grant_keeps_stale_suspension = quirk;
        let mut w = game(&r);
        let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
        grant_tick(&mut w, ALPHA, CURTAIN);
        switch(&mut w, b, false);
        until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
        remove(&mut w, b);
        charge::one_time_grant(&mut w, ALPHA, CURTAIN);
        let x = rec(&w, ALPHA, CURTAIN);
        assert!(x.ready);
        assert_eq!(x.suspended, quirk);
    }
}

// --- Claim 13 ----------------------------------------------------------------

#[test]
fn claim13_permanent_grant_survives_building_loss() {
    // spec: S-0512 claim 13 (granted as a building would, never suspended at the grant, never lost by the checks)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "HOG", ALPHA, 4, 4);
    run(&mut w, 3);
    charge::permanent_grant(&mut w, ALPHA, CURTAIN);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(x.granted && x.permanent && !x.suspended && x.running && x.remain == T);
    run(&mut w, 5);
    assert!(rec(&w, ALPHA, CURTAIN).granted && !rec(&w, ALPHA, CURTAIN).suspended);
}

#[test]
fn claim13_permanent_grant_freezes_suspension() {
    // spec: S-0512 claim 13 (quirks.permanent_grant_freezes_suspension: a suspended record stays suspended for good; off: resumed)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.permanent_grant_freezes_suspension = quirk;
        let mut w = game(&r);
        let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
        grant_tick(&mut w, ALPHA, CURTAIN);
        switch(&mut w, b, false);
        until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
        charge::permanent_grant(&mut w, ALPHA, CURTAIN);
        switch(&mut w, b, true);
        run(&mut w, 5);
        assert_eq!(rec(&w, ALPHA, CURTAIN).suspended, quirk, "quirk {quirk}");
    }
}

// --- Claim 14 ----------------------------------------------------------------

#[test]
fn claim14_set_charge_formula() {
    // spec: S-0512 claim 14 and F4 (r = T − trunc(Z(Z(p × c) × T)); p = 100 gives 1; outside 0 to 100 nothing)
    let (mut w, _, _) = curtain_game();
    let t = w.tick;
    charge::set_charge(&mut w, ALPHA, CURTAIN, 100);
    let x = rec(&w, ALPHA, CURTAIN);
    assert_eq!((x.remain, x.start, x.running), (1, t, true));
    charge::set_charge(&mut w, ALPHA, CURTAIN, 50);
    // 50 × single(0.01) = 0.4999999888241291; × 90 = 44.99999899... → 44.
    assert_eq!(rec(&w, ALPHA, CURTAIN).remain, T - 44);
    charge::set_charge(&mut w, ALPHA, CURTAIN, 0);
    assert_eq!(rec(&w, ALPHA, CURTAIN).remain, T);
    let before = rec(&w, ALPHA, CURTAIN);
    charge::set_charge(&mut w, ALPHA, CURTAIN, 101);
    charge::set_charge(&mut w, ALPHA, CURTAIN, -1);
    assert_eq!(rec(&w, ALPHA, CURTAIN), before);
}

#[test]
fn claim14_set_charge_ready_next_tick_with_announcement() {
    // spec: S-0512 claim 14 (a charging record becomes ready through claim 5, with its announcement, at the earliest in the next tick's house update)
    let (mut w, _, _) = curtain_game();
    charge::set_charge(&mut w, ALPHA, CURTAIN, 100);
    let t = w.tick;
    step(&mut w);
    step(&mut w);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(x.ready);
    assert!(x.ready_tick > t);
}

#[test]
fn claim14_set_charge_on_ungranted_does_nothing() {
    // spec: S-0512 claim 14 (acts only on a granted record)
    let r = ruleset();
    let mut w = game(&r);
    charge::set_charge(&mut w, ALPHA, CURTAIN, 50);
    assert!(!rec(&w, ALPHA, CURTAIN).granted);
}

#[test]
fn claim14_set_charge_keeps_ready_quirk() {
    // spec: S-0512 claim 14 (quirks.set_charge_keeps_ready: a ready record stays ready; off: readiness cleared)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.set_charge_keeps_ready = quirk;
        let mut w = game(&r);
        spawn(&mut w, "IRONB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, CURTAIN, 200);
        charge::set_charge(&mut w, ALPHA, CURTAIN, 10);
        assert_eq!(rec(&w, ALPHA, CURTAIN).ready, quirk);
    }
}

#[test]
fn claim14_set_charge_runs_through_suspension() {
    // spec: S-0512 claim 14 (quirks.superweapon_restart_ignores_suspension: the clock runs through the suspension; off: kept stopped)
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_restart_ignores_suspension = quirk;
        let mut w = game(&r);
        let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
        grant_tick(&mut w, ALPHA, CURTAIN);
        switch(&mut w, b, false);
        until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
        charge::set_charge(&mut w, ALPHA, CURTAIN, 50);
        assert_eq!(rec(&w, ALPHA, CURTAIN).running, quirk);
    }
}

// --- Claim 15 ----------------------------------------------------------------

#[test]
fn claim15_reset() {
    // spec: S-0512 claim 15 (not ready, remaining T; a running clock restarts from that tick; a stopped clock stays stopped; no grant)
    let (mut w, b, _) = curtain_game();
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    let t = w.tick;
    charge::reset(&mut w, ALPHA, CURTAIN);
    let x = rec(&w, ALPHA, CURTAIN);
    assert!(!x.ready && x.running && x.remain == T && x.start == t);
    switch(&mut w, b, false);
    until(&mut w, 10, |w| rec(w, ALPHA, CURTAIN).suspended);
    charge::reset(&mut w, ALPHA, CURTAIN);
    let y = rec(&w, ALPHA, CURTAIN);
    assert!(!y.running && y.remain == T);
    charge::reset(&mut w, BETA, CURTAIN);
    assert!(!rec(&w, BETA, CURTAIN).granted);
}

#[test]
fn claim15_spy_reset_first_superweapon_only() {
    // spec: S-0512 claim 15 (spy infiltration resets only the building type's first superweapon, never its second)
    let r = ruleset();
    let mut w = game(&r);
    let b = spawn(&mut w, "AIRB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, DROP, 200);
    assert!(rec(&w, ALPHA, SPY).ready);
    charge::spy_reset(&mut w, b);
    assert!(!rec(&w, ALPHA, DROP).ready);
    assert!(rec(&w, ALPHA, SPY).ready);
}

// --- Claims 16, 17 and 18 ------------------------------------------------------

#[test]
fn claim16_clock_stage_and_labels() {
    // spec: S-0512 claim 16 and F3 (stage min(53, ⌊54 (T − r) / T⌋) while charging, 54 ready; labels)
    let r = ruleset();
    let mut w = game(&r);
    assert_eq!(charge::stage(&w, ALPHA, CURTAIN), 0);
    spawn(&mut w, "IRONB", ALPHA, 0, 0);
    let g = grant_tick(&mut w, ALPHA, CURTAIN);
    let x = rec(&w, ALPHA, CURTAIN);
    let rem = x.remaining(w.tick);
    assert_eq!(
        charge::stage(&w, ALPHA, CURTAIN),
        (54 * (T - rem) / T).min(53)
    );
    assert_eq!(charge::label(&w, ALPHA, CURTAIN), Label::None);
    let _ = g;
    run(&mut w, 45);
    let rem = rec(&w, ALPHA, CURTAIN).remaining(w.tick);
    assert_eq!(charge::stage(&w, ALPHA, CURTAIN), 54 * (T - rem) / T);
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    assert_eq!(charge::stage(&w, ALPHA, CURTAIN), 54);
    assert_eq!(charge::label(&w, ALPHA, CURTAIN), Label::Ready);
    assert!(charge::offers_targeting(&w, ALPHA, CURTAIN));
}

#[test]
fn claim17_flash_window() {
    // spec: S-0512 claim 17 (−1 unlimited; F > 0: ticks r to r + F − 1 from the ready tick)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "IRONB", ALPHA, 0, 0);
    spawn(&mut w, "TIMEB", ALPHA, 2, 0);
    spawn(&mut w, "AUXB", ALPHA, 4, 0);
    ready_tick(&mut w, ALPHA, TIMED, 200);
    let rt = rec(&w, ALPHA, TIMED).ready_tick;
    assert!(charge::flashing(&w, ALPHA, TIMED));
    let left = 29 - (w.tick - rt);
    run(&mut w, left);
    assert_eq!(w.tick, rt + 29);
    assert!(charge::flashing(&w, ALPHA, TIMED));
    step(&mut w);
    assert!(
        !charge::flashing(&w, ALPHA, TIMED),
        "ticks r to r + F − 1 only"
    );
    assert!(charge::flashing(&w, ALPHA, CURTAIN), "−1: unlimited");
}

#[test]
fn claim18_countdown_rows_in_grant_order() {
    // spec: S-0512 claim 18 and F6 (rows for types with the countdown flag, in grant order; s = ⌊remaining / 15⌋)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "AUXB", BETA, -6, -6);
    spawn(&mut w, "TIMEB", BETA, -4, -4);
    run(&mut w, 3);
    spawn(&mut w, "AUXB", ALPHA, 6, 6);
    spawn(&mut w, "TIMEB", ALPHA, 4, 4);
    spawn(&mut w, "IRONB", ALPHA, 2, 2);
    run(&mut w, 3);
    let rows = charge::countdown_rows(&w);
    assert_eq!(rows.len(), 2, "only the countdown type has rows");
    assert_eq!(rows[0].house, BETA);
    assert_eq!(rows[1].house, ALPHA);
    let rem = rec(&w, BETA, TIMED).remaining(w.tick);
    assert_eq!(rows[0].seconds, rem / 15);
}

// --- Claims 19 and 20 --------------------------------------------------------

#[test]
fn claim19_drain_cycle() {
    // spec: S-0512 claim 19 and F5 (charging → ready without announcement; ready → active for trunc((T − r) ρ); active → ready for trunc(T − r / ρ); active expiry → charging)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "DRAINB", ALPHA, 0, 0);
    grant_tick(&mut w, ALPHA, DRAINER);
    assert_eq!(rec(&w, ALPHA, DRAINER).drain, DrainState::Charging);
    w.effects.clear();
    until(&mut w, 200, |w| {
        rec(w, ALPHA, DRAINER).drain == DrainState::Ready
    });
    assert!(!events(&w).iter().any(|e| e.0 == SwEvent::Ready));
    assert_eq!(rec(&w, ALPHA, DRAINER).ready_tick, 0, "no ready tick");
    fire_now(&mut w, ALPHA, DRAINER, at(3, 3));
    let x = rec(&w, ALPHA, DRAINER);
    assert_eq!(x.drain, DrainState::Active);
    // ρ = 3 (default): active time trunc((90 − 0) × 3) = 270.
    assert_eq!(x.remain, 3 * T);
    run(&mut w, 30);
    let r_now = rec(&w, ALPHA, DRAINER).remaining(w.tick);
    fire(&mut w, ALPHA, DRAINER, at(3, 3));
    step(&mut w);
    let y = rec(&w, ALPHA, DRAINER);
    assert_eq!(y.drain, DrainState::Ready);
    // recharge trunc(90 − r / 3), r the remaining active time in the order's tick
    assert_eq!(y.remain, (3 * T - r_now) / 3);
    fire_now(&mut w, ALPHA, DRAINER, at(3, 3));
    let z = rec(&w, ALPHA, DRAINER);
    assert_eq!(z.drain, DrainState::Active);
    until(&mut w, 400, |w| {
        rec(w, ALPHA, DRAINER).drain == DrainState::Charging
    });
    assert_eq!(rec(&w, ALPHA, DRAINER).remain, T);
}

#[test]
fn claim19_drain_order_in_charging_does_nothing() {
    // spec: S-0512 claim 19 (an order in charging does nothing)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "DRAINB", ALPHA, 0, 0);
    grant_tick(&mut w, ALPHA, DRAINER);
    let before = rec(&w, ALPHA, DRAINER);
    fire_now(&mut w, ALPHA, DRAINER, at(3, 3));
    assert_eq!(rec(&w, ALPHA, DRAINER), before);
}

#[test]
fn claim20_manual_charge() {
    // spec: S-0512 claim 20 (granted with the clock stopped at T; charges only by a set-charge action; stops again at T after firing)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "MANB", ALPHA, 0, 0);
    grant_tick(&mut w, ALPHA, MANUAL);
    let x = rec(&w, ALPHA, MANUAL);
    assert!(!x.running && x.remain == T);
    run(&mut w, T + 5);
    assert!(!rec(&w, ALPHA, MANUAL).ready);
    charge::set_charge(&mut w, ALPHA, MANUAL, 100);
    run(&mut w, 2);
    assert!(rec(&w, ALPHA, MANUAL).ready);
    fire_now(&mut w, ALPHA, MANUAL, at(3, 3));
    let y = rec(&w, ALPHA, MANUAL);
    assert!(!y.ready && !y.running && y.remain == T);
}

// --- Claims 21, 22 and 23 ------------------------------------------------------

#[test]
fn claim21_charge_time_import() {
    // spec: S-0512 claim 21 and F1 (T = trunc(Z(v × 900)) from the single-precision value; absent or 0: the engine default 4500)
    let mut r = ruleset();
    r.sim.superweapons.get_mut("STORM").unwrap().recharge_time = None;
    r.sim.superweapons.get_mut("SOURCE").unwrap().recharge_time =
        Some(glasnost_rules::sim::Decimal("0".into()));
    r.sim.superweapons.get_mut("DROP").unwrap().recharge_time =
        Some(glasnost_rules::sim::Decimal("2.5".into()));
    let w = game(&r);
    let t = |i: u32| w.rules().superweapons.types[i as usize].charge;
    assert_eq!(t(CURTAIN), T);
    assert_eq!(t(STORM), 4500);
    assert_eq!(t(SOURCE), 4500);
    assert_eq!(t(DROP), 2250);
    assert!(!w.rules().superweapons.warnings.is_empty());
}

#[test]
fn claim22_defeat_loses_every_record_at_next_check() {
    // spec: S-0512 claim 22 (a defeated house loses every granted record, one-time and permanent included, at the next availability check; buildings grant it nothing)
    let (mut w, _, _) = curtain_game();
    charge::one_time_grant(&mut w, ALPHA, STORM);
    charge::permanent_grant(&mut w, ALPHA, SOURCE);
    w.houses[0].econ.result.defeated = true;
    assert!(
        rec(&w, ALPHA, CURTAIN).granted,
        "defeat alone revokes nothing"
    );
    charge::availability(&mut w, ALPHA, None);
    for i in [CURTAIN, STORM, SOURCE] {
        assert!(!rec(&w, ALPHA, i).granted);
    }
    charge::grants(&mut w, ALPHA);
    assert!(!rec(&w, ALPHA, CURTAIN).granted);
}

#[test]
fn claim23_actions_import_and_shared_action_fires_first_type() {
    // spec: S-0512 claim 23 (the aim action is separate from the kind, case-insensitive, `None` is no action); S-0513 claim 2 (quirks.superweapon_order_first_type_of_action)
    let r = ruleset();
    let w = game(&r);
    let ty = &w.rules().superweapons.types;
    assert_eq!(ty[STRIKE as usize].action.as_deref(), Some("nuke"));
    assert_eq!(ty[SHARED as usize].action.as_deref(), Some("nuke"));
    assert!(ty[SHARED as usize].kind.is_none());
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_order_first_type_of_action = quirk;
        r.sim.superweapons.get_mut("STRIKE").unwrap().action = Some("NUKE".into());
        let mut w = game(&r);
        spawn(&mut w, "SHAREB", ALPHA, 0, 0);
        ready_tick(&mut w, ALPHA, SHARED, 200);
        fire_now(&mut w, ALPHA, SHARED, at(3, 3));
        // With the quirk the order acts on STRIKE (not granted): refused,
        // and SHARED keeps its charge.
        assert_eq!(rec(&w, ALPHA, SHARED).ready, quirk, "quirk {quirk}");
    }
    let mut r = ruleset();
    r.sim.superweapons.get_mut("SHARED").unwrap().action = Some("None".into());
    let w = game(&r);
    assert!(
        w.rules().superweapons.types[SHARED as usize]
            .action
            .is_none()
    );
}
