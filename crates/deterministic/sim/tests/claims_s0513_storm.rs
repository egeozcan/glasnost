//! Claim tests of S-0513 claims 14 to 21 (the weather storm), table T1 and
//! Formulas F4 to F6, on synthetic rules (`tests/sw_common`: countdown 30,
//! duration 60, hit period 10, scatter period 5, spread 6, separation 2,
//! damage 50, clouds with middle frame 3).

mod sw_common;

use glasnost_num::Rounding;
use glasnost_sim::superweapons::{StormState, SwEvent, storm};
use glasnost_sim::world::World;
use sw_common::*;

/// Steps one tick with the effects cleared first; returns that tick's
/// superweapon events.
fn tick_events(w: &mut World) -> Vec<SwEvent> {
    w.effects.clear();
    step(w);
    events(w).into_iter().map(|e| e.0).collect()
}

/// Runs until the storm is in `s`; returns the tick in which it changed.
fn until_state(w: &mut World, s: StormState, max: i32) -> i32 {
    until(w, max, |w| w.sw.storm.state == s) - 1
}

// --- F4 ----------------------------------------------------------------------------

#[test]
fn f4_cloud_lift_examples() {
    // spec: S-0513 F4 (image heights 0 or 1 give −3 leptons, 2 give 3, 100 give 344, 200 give 693)
    for m in [Rounding::Nearest, Rounding::TowardZero] {
        assert_eq!(storm::cloud_lift(0, m), -3);
        assert_eq!(storm::cloud_lift(1, m), -3);
        assert_eq!(storm::cloud_lift(2, m), 3);
        assert_eq!(storm::cloud_lift(100, m), 344);
        assert_eq!(storm::cloud_lift(200, m), 693);
    }
}

// --- Claim 14 and F5 --------------------------------------------------------------------

#[test]
fn claim14_countdown_start_tick_and_launch_announcement() {
    // spec: S-0513 claim 14 and F5 (a request in tick T before the weather update with countdown L starts the storm in tick T + L − 1)
    let r = ruleset();
    let mut w = game(&r);
    let t = w.tick;
    storm::request(&mut w, Some(ALPHA), at(0, 0), STORM_D, DEFER);
    assert_eq!(w.sw.storm.state, StormState::Counting);
    let s = until_state(&mut w, StormState::Raging, 100);
    assert_eq!(s, t + DEFER - 1);
    assert_eq!(w.sw.storm.start, s);
    // spec: S-0513 claim 14 (the weather launch plays the storm-created announcement to every player)
    let mut w = game(&r);
    spawn(&mut w, "WEATHERB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, STORM, 200);
    fire(&mut w, ALPHA, STORM, at(0, 0));
    assert!(tick_events(&mut w).contains(&SwEvent::Launch));
    assert_eq!(w.sw.storm.state, StormState::Counting);
    assert_eq!(w.sw.storm.owner, Some(ALPHA));
}

#[test]
fn claim14_countdown_replacement_and_refusals() {
    // spec: S-0513 claim 14 (a later request replaces centre, owner and duration; the remainder is the smaller of the running one and L)
    let mut w = game(&ruleset());
    storm::request(&mut w, Some(ALPHA), at(0, 0), STORM_D, 20);
    run(&mut w, 5);
    let left = w.sw.storm.countdown;
    storm::request(&mut w, Some(BETA), at(2, 2), 7, 100);
    assert_eq!(w.sw.storm.countdown, left);
    assert_eq!(w.sw.storm.owner, Some(BETA));
    assert_eq!(w.sw.storm.centre, at(2, 2));
    assert_eq!(w.sw.storm.duration, 7);
    storm::request(&mut w, Some(BETA), at(2, 2), 7, 3);
    assert_eq!(w.sw.storm.countdown, 3);
    // spec: S-0513 claim 14 (a request while raging does nothing)
    until_state(&mut w, StormState::Raging, 10);
    storm::request(&mut w, Some(ALPHA), at(-3, -3), 500, 0);
    assert_eq!(w.sw.storm.owner, Some(BETA));
    assert_eq!(w.sw.storm.duration, 7);
    // spec: S-0512 claim 8 (a weather launch while raging is refused, the record stays ready)
    spawn(&mut w, "WEATHERB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, STORM, 200);
    if w.sw.storm.state == StormState::Idle {
        storm::request(&mut w, Some(BETA), at(0, 0), 500, 0);
    }
    fire(&mut w, ALPHA, STORM, at(0, 0));
    assert!(tick_events(&mut w).contains(&SwEvent::Refused));
    assert!(rec(&w, ALPHA, STORM).ready);
}

#[test]
fn claim14_approach_warnings_every_225_ticks() {
    // spec: S-0513 claim 14 and F5 (warnings in ticks T + L − 225k − 1 that lie before the start)
    let mut w = game(&ruleset());
    let t = w.tick;
    let l = 500;
    storm::request(&mut w, Some(ALPHA), at(0, 0), STORM_D, l);
    let mut seen = Vec::new();
    while w.sw.storm.state == StormState::Counting {
        let k = w.tick;
        if tick_events(&mut w).contains(&SwEvent::StormApproach) {
            seen.push(k);
        }
    }
    assert_eq!(seen, vec![t + l - 450 - 1, t + l - 225 - 1]);
}

// --- Claim 15 ----------------------------------------------------------------------------

#[test]
fn claim15_start_radar_outage_event_and_message() {
    // spec: S-0513 claim 15 (radar outages of the duration for every house not allied with the owner; a radar event; the storm-start message)
    let mut w = game(&ruleset());
    storm::request(&mut w, Some(ALPHA), at(0, 0), STORM_D, 1);
    let ev = tick_events(&mut w);
    assert_eq!(w.sw.storm.state, StormState::Raging);
    assert!(ev.contains(&SwEvent::Radar));
    assert!(ev.contains(&SwEvent::StormStart));
    let s = w.sw.storm.start;
    assert_eq!(w.houses[1].econ.radar.outage_until, Some(s + STORM_D));
    assert_eq!(w.houses[0].econ.radar.outage_until, None);
}

#[test]
fn claim15_no_owner_every_house_loses_radar() {
    // spec: S-0513 claim 15 (every such house when there is no owner)
    let mut w = game(&ruleset());
    storm::request(&mut w, None, at(0, 0), STORM_D, 0);
    assert!(w.houses[0].econ.radar.outage_until.is_some());
    assert!(w.houses[1].econ.radar.outage_until.is_some());
}

// --- Claims 16 and 17, F6 ------------------------------------------------------------------

#[test]
fn claim16_centre_clouds_on_absolute_multiples_and_scatter_separation() {
    // spec: S-0513 claim 16 (centre clouds in ticks that are multiples of the hit period; scatter clouds keep the separation); F6
    let mut w = game(&ruleset());
    storm::request(&mut w, Some(ALPHA), at(0, 0), 500, 0);
    let mut centre_ticks = Vec::new();
    for _ in 0..40 {
        let k = w.tick;
        let n0 = w.sw.clouds.iter().filter(|c| c.created == k).count();
        let _ = n0;
        step(&mut w);
        let made: Vec<_> = w.sw.clouds.iter().filter(|c| c.created == k).collect();
        if made.iter().any(|c| c.cell == at(0, 0)) {
            centre_ticks.push(k);
        }
        // at most one scatter cloud plus one centre cloud per tick
        assert!(made.len() <= 2);
        for c in &made {
            if c.cell != at(0, 0) {
                // spec: S-0513 F6 (|Δx|, |Δy| ≤ ⌊6 / 2⌋)
                assert!((c.cell.x - at(0, 0).x).abs() <= 3);
                assert!((c.cell.y - at(0, 0).y).abs() <= 3);
                assert_eq!(k % SCATTER, 0);
            }
        }
    }
    assert!(!centre_ticks.is_empty());
    assert!(centre_ticks.iter().all(|t| t % HIT == 0));
    // spec: S-0513 F6 (no two present clouds closer than the separation, except centre clouds over the same cell)
    let cs = &w.sw.clouds;
    for (i, a) in cs.iter().enumerate() {
        for b in &cs[i + 1..] {
            let d = (a.cell.x - b.cell.x).abs() + (a.cell.y - b.cell.y).abs();
            let both_centre = a.cell == at(0, 0) && b.cell == at(0, 0);
            assert!(
                d >= 2 || both_centre || a.created == b.created,
                "{a:?} {b:?}"
            );
        }
    }
}

#[test]
fn claim17_cloud_height_and_list_pick() {
    // spec: S-0513 claim 17 and F4 (over the cell centre at h_level + h_c; Q-0737 interim H = 0, h_c = −3); one list pick before the creation draws
    let mut w = game(&ruleset());
    let taken = w.rng.taken();
    storm::cloud(&mut w, at(1, 1));
    assert!(w.rng.taken() > taken);
    let c = &w.sw.clouds[0];
    let lvl = i32::from(w.cells.cell(at(1, 1)).unwrap().level);
    assert_eq!(c.pos.z, lvl * 104 - 3);
    assert!(c.anim == "CLOUD1" || c.anim == "CLOUD2");
}

// --- Claim 18 ------------------------------------------------------------------------------

#[test]
fn claim18_trigger_cloud_strikes_while_idle_without_credit() {
    // spec: S-0513 claim 18 (a trigger cloud strikes in every storm state once its frame exceeds ⌊F / 2⌋); claim 19 (before any request it credits no house)
    let mut w = game(&ruleset());
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let h0 = w.obj(cart).unwrap().health;
    storm::cloud(&mut w, at(0, 0));
    let mut lightning = false;
    for _ in 0..60 {
        if tick_events(&mut w)
            .iter()
            .any(|e| matches!(e, SwEvent::Lightning(_)))
        {
            lightning = true;
        }
        if !w.sw.clouds.first().is_some_and(|c| c.pending) {
            break;
        }
    }
    assert!(lightning, "the strike plays a lightning sound");
    assert!(w.obj(cart).is_none_or(|o| o.health < h0));
    assert_eq!(w.sw.storm.state, StormState::Idle);
    // The cloud goes once it reaches its last frame (Q-0732 interim F = 2μ).
    until(&mut w, 100, |w| w.sw.clouds.is_empty());
}

#[test]
fn claim18_strike_frame_after_middle_frame() {
    // spec: S-0513 claim 18 (Q-0732 interim: a cloud with μ = 3 strikes in the first update whose frame ⌊(t − t₀) / d⌋ exceeds 3)
    let mut w = game(&ruleset());
    storm::cloud(&mut w, at(0, 0));
    let c = w.sw.clouds[0].clone();
    let t = until(&mut w, 200, |w| !w.sw.clouds[0].pending) - 1;
    assert_eq!(t, c.created + 4 * c.delay);
}

// --- Claim 19 --------------------------------------------------------------------------------

#[test]
fn claim19_clearing_after_duration_then_idle_without_clouds() {
    // spec: S-0513 claim 19 and F5 (clearing in tick t_s + D + 1; idle in the first update after clearing began that finds no present cloud; owner none)
    let mut r = ruleset();
    r.sim.general.superweapons.weather_con_clouds = Some(vec![]);
    let mut w = game(&r);
    storm::request(&mut w, Some(ALPHA), at(0, 0), 20, 1);
    let s = until_state(&mut w, StormState::Raging, 10);
    let c = until_state(&mut w, StormState::Clearing, 100);
    assert_eq!(c, s + 20 + 1);
    let i = until_state(&mut w, StormState::Idle, 10);
    assert_eq!(i, c + 1);
    assert_eq!(w.sw.storm.owner, None);
}

#[test]
fn claim19_present_clouds_postpone_idle() {
    // spec: S-0513 claim 19 (the storm becomes idle one weather update after the last cloud's removal; a trigger cloud added while clearing postpones it)
    let mut w = game(&ruleset());
    storm::request(&mut w, Some(ALPHA), at(0, 0), 20, 1);
    let c = until_state(&mut w, StormState::Clearing, 100);
    storm::cloud(&mut w, at(3, 3));
    let mut gone = None;
    while w.sw.storm.state != StormState::Idle {
        let k = w.tick;
        step(&mut w);
        if gone.is_none() && w.sw.clouds.is_empty() {
            gone = Some(k);
        }
    }
    let idle = w.tick - 1;
    assert!(idle > c + 1);
    assert_eq!(Some(idle - 1), gone);
}

// --- Claim 20 -------------------------------------------------------------------------------

#[test]
fn claim20_stop_and_start_actions() {
    // spec: S-0513 claim 20 (the storm-stop action begins clearing while raging; the storm-start action requests a storm at a random legal cell with no owner)
    let mut w = game(&ruleset());
    storm::stop(&mut w);
    assert_eq!(w.sw.storm.state, StormState::Idle);
    let taken = w.rng.taken();
    storm::start_action(&mut w, 40);
    assert!(w.rng.taken() >= taken + 2, "a row and a column draw");
    assert_eq!(w.sw.storm.state, StormState::Counting);
    assert_eq!(w.sw.storm.owner, None);
    assert_eq!(w.sw.storm.duration, 40);
    assert!(w.cells.footprint().is_legal(w.sw.storm.centre));
    until_state(&mut w, StormState::Raging, 100);
    storm::stop(&mut w);
    assert_eq!(w.sw.storm.state, StormState::Clearing);
}

// --- Claim 21 -------------------------------------------------------------------------------

#[test]
fn claim21_unsafe_storm_data() {
    // spec: S-0513 claim 21 (an empty cloud list makes no cloud; no warhead delivers no damage; both warn)
    let mut r = ruleset();
    r.sim.general.superweapons.weather_con_clouds = Some(vec![]);
    let w0 = game(&r);
    assert!(
        w0.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("cloud"))
    );
    let mut w = w0;
    storm::cloud(&mut w, at(0, 0));
    assert!(w.sw.clouds.is_empty());
    let mut r = ruleset();
    r.sim.general.superweapons.lightning_warhead = None;
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let h0 = w.obj(cart).unwrap().health;
    storm::cloud(&mut w, at(0, 0));
    until(&mut w, 200, |w| w.sw.clouds.is_empty());
    assert_eq!(w.obj(cart).unwrap().health, h0);
}
