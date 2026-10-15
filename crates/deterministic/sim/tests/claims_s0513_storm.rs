//! Claim tests of S-0513 claims 14 to 21 (the weather storm), table T1 and
//! Formulas F4 to F6, on synthetic rules (`tests/sw_common`: countdown 30,
//! duration 60, hit period 10, scatter period 5, spread 6, separation 2,
//! damage 50, clouds of 7 image frames with a stage delay of 1).

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
    // spec: S-0513 claim 17 and F4 (over the cell centre at h_level + h_c; the fixture bolt image header has canvas height 0, so h_c = −3); one list pick before the creation draws
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
    // The cloud goes once a weather update finds its stage at F - 1 (claim 46).
    until(&mut w, 100, |w| w.sw.clouds.is_empty());
}

/// The debris animations a strike at the cell (0, 0) makes with a vehicle at
/// in-cell offset `cart_at` and an infantry unit at `walker_at` (both in
/// cell (0, 0) of the middle), the vehicle one hit from death and the
/// infantry unit sturdy: the debris pieces' positions.
fn strike_debris(cart_at: (i32, i32), walker_at: (i32, i32)) -> Vec<glasnost_sim::world::Pos> {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().strength = Some(40);
    r.sim.infantry.get_mut("WALKER").unwrap().strength = Some(5000);
    r.sim.general.metallic_debris = Some(vec!["DEB1".into(), "DEB2".into()]);
    for n in ["DEB1", "DEB2"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    let base = at(0, 0);
    for (id, (x, y)) in [(cart, cart_at), (walker, walker_at)] {
        let o = w.obj_mut(id).unwrap();
        o.pos.x = base.x * 256 + x;
        o.pos.y = base.y * 256 + y;
    }
    storm::cloud(&mut w, base);
    for _ in 0..100 {
        w.effects.clear();
        step(&mut w);
        if events(&w)
            .iter()
            .any(|e| matches!(e.0, SwEvent::Lightning(_)))
        {
            assert!(
                w.obj(cart).is_none_or(|o| o.health <= 0),
                "the vehicle died"
            );
            return w
                .effects
                .iter()
                .filter_map(|e| match e {
                    glasnost_sim::combat::effects::Effect::Anim { name, pos, .. }
                        if name.starts_with("DEB") =>
                    {
                        Some(*pos)
                    }
                    _ => None,
                })
                .collect();
        }
    }
    panic!("no strike");
}

#[test]
fn claim18_debris_follows_the_top_corner_witness_not_the_centre_nearest_unit() {
    // spec: S-0513 claim 18 (Q-0740: the witness is the ground-layer candidate with the smallest trunc(r(x² + y²)) of its in-cell position, the top corner as reference; debris only when the pre-damage witness is not infantry and the witness changed)
    // Vehicle near the top corner, infantry near the centre: the vehicle is the witness, it dies, debris.
    let pieces = strike_debris((20, 20), (130, 130));
    assert!((2..=4).contains(&pieces.len()), "{}", pieces.len());
    // Debris is created at the strike point: the cell centre at its height level, no 20-lepton lift.
    let c = at(0, 0);
    let level = 0; // flat test map
    for p in &pieces {
        assert_eq!((p.x, p.y), (c.x * 256 + 128, c.y * 256 + 128));
        assert_eq!(p.z, level);
    }
    // Reversed: the infantry unit is nearer the top corner, so it is the witness and no debris is left.
    let pieces = strike_debris((130, 130), (20, 20));
    assert!(pieces.is_empty());
}

/// Whether a strike at the cell offset `strike` makes storm debris when an
/// `AUXB` building of foundation `found` (top-left cell (0, 0), one hit from
/// death) and a sturdy infantry unit at in-cell offset `walker_at` of the
/// cell offset `walker_cell` share the struck cell. Debris follows only a
/// pre-damage witness that is not infantry, so no debris means the infantry
/// unit won the witness test.
fn building_strike_debris(
    found: [i32; 2],
    strike: (i32, i32),
    walker_cell: (i32, i32),
    walker_at: (i32, i32),
) -> bool {
    let mut r = ruleset();
    {
        let b = r.sim.buildings.get_mut("AUXB").unwrap();
        b.strength = Some(40);
        b.foundation = Some(found);
    }
    r.sim.infantry.get_mut("WALKER").unwrap().strength = Some(5000);
    r.sim.general.metallic_debris = Some(vec!["DEB1".into(), "DEB2".into()]);
    for n in ["DEB1", "DEB2"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    let mut w = game(&r);
    // The infantry unit is placed first, so it also precedes the building in the listing.
    let walker = spawn(&mut w, "WALKER", BETA, walker_cell.0, walker_cell.1);
    let bld = spawn(&mut w, "AUXB", BETA, 0, 0);
    let base = at(walker_cell.0, walker_cell.1);
    let o = w.obj_mut(walker).unwrap();
    o.pos.x = base.x * 256 + walker_at.0;
    o.pos.y = base.y * 256 + walker_at.1;
    let target = at(strike.0, strike.1);
    storm::cloud(&mut w, target);
    for _ in 0..100 {
        w.effects.clear();
        step(&mut w);
        if events(&w)
            .iter()
            .any(|e| matches!(e.0, SwEvent::Lightning(_)))
        {
            assert!(w.obj(bld).is_none_or(|o| o.health <= 0), "the building died");
            return w.effects.iter().any(|e| {
                matches!(e, glasnost_sim::combat::effects::Effect::Anim { name, .. }
                    if name.starts_with("DEB"))
            });
        }
    }
    panic!("no strike");
}

#[test]
fn claim18_a_building_witness_is_measured_at_its_foundation_centre() {
    // spec: S-0513 claim 18 (Q-0800: the building's reference point is its foundation centre, (256a + 128W, 256b + 128H); only its remainders modulo 256 enter, so a 1 x 1 building is at distance 181, a 2 x 2 building at 0, odd-by-even and even-by-odd at 128); S-0204 claim 4
    // 1 x 1: distance 181. Infantry below it win (no debris); above it the building is the witness (debris).
    assert!(!building_strike_debris([1, 1], (0, 0), (0, 0), (100, 100)));
    assert!(building_strike_debris([1, 1], (0, 0), (0, 0), (200, 200)));
    // 2 x 2: distance 0 against every positive-distance candidate.
    assert!(building_strike_debris([2, 2], (0, 0), (0, 0), (1, 1)));
    // Odd width, even height (1 x 2): remainders (128, 0), distance 128.
    assert!(!building_strike_debris([1, 2], (0, 0), (0, 0), (120, 0)));
    assert!(building_strike_debris([1, 2], (0, 0), (0, 0), (150, 0)));
    // Even width, odd height (2 x 1): remainders (0, 128), distance 128.
    assert!(!building_strike_debris([2, 1], (0, 0), (0, 0), (0, 120)));
    assert!(building_strike_debris([2, 1], (0, 0), (0, 0), (0, 150)));
}

#[test]
fn claim18_the_building_threshold_is_the_same_in_every_foundation_cell() {
    // spec: S-0513 claim 18 (Q-0800: the building is listed in every foundation cell and the same remainder test applies in each; a missing cell of an irregular foundation does not list it); S-0209 claim 13 (per-foundation-cell listing)
    // A 2 x 2 building struck on its non-anchor cell (1, 1): it still beats a positive-distance infantry unit.
    assert!(building_strike_debris([2, 2], (1, 1), (1, 1), (1, 1)));
    assert!(building_strike_debris([2, 2], (1, 0), (1, 0), (255, 255)));
    // A 3 x 3 building (both odd, 181) struck on a non-anchor cell: infantry at 141 wins, at 282 the building does.
    assert!(!building_strike_debris([3, 3], (2, 2), (2, 2), (100, 100)));
    assert!(building_strike_debris([3, 3], (2, 2), (2, 2), (200, 200)));
}

/// The synthetic rules with both clouds of `frames` image frames and rate
/// `rate` (None: no rate field).
fn cloud_rules(frames: i32, rate: Option<i32>) -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    for n in ["CLOUD1", "CLOUD2"] {
        let a = r.sim.animations.get_mut(n).unwrap();
        a.frame_count = Some(frames);
        a.middle_frame = Some(frames / 2);
        a.rate = rate;
    }
    r
}

#[test]
fn claim46_forward_cloud_strike_and_presence_ticks() {
    // spec: S-0513 claim 46 (an ordinary forward one-pass cloud created before the turn pass: strike in tick c + (⌊F/2⌋ + 1)d + 1 for F ≥ 3, ceasing to be present in tick c + (F − 1)d + 1; Q-0732)
    for frames in [3, 5, 8] {
        for d in [1, 2, 4] {
            let r = cloud_rules(frames, Some(900 / d));
            let mut w = game(&r);
            storm::cloud(&mut w, at(0, 0));
            let c = w.sw.clouds[0].clone();
            assert_eq!(c.delay, d, "F {frames} d {d}");
            let (mut strike, mut gone) = (None, None);
            for _ in 0..200 {
                let t = w.tick;
                if tick_events(&mut w)
                    .iter()
                    .any(|e| matches!(e, SwEvent::Lightning(_)))
                {
                    strike = Some(t);
                }
                if w.sw.clouds.is_empty() {
                    gone = Some(t);
                    break;
                }
            }
            assert_eq!(
                strike,
                Some(c.created + (frames / 2 + 1) * d + 1),
                "F {frames} d {d}"
            );
            assert_eq!(
                gone,
                Some(c.created + (frames - 1) * d + 1),
                "F {frames} d {d}"
            );
        }
    }
}

#[test]
fn claim46_short_clouds_never_strike() {
    // spec: S-0513 claim 46 and claim 18 (with F of 1 or 2 the ordinary cloud is removed before a striking stage survives to a weather update)
    for frames in [1, 2] {
        let r = cloud_rules(frames, Some(450));
        let mut w = game(&r);
        let cart = spawn(&mut w, "CART", BETA, 0, 0);
        let h0 = w.obj(cart).unwrap().health;
        storm::cloud(&mut w, at(0, 0));
        while !w.sw.clouds.is_empty() {
            assert!(w.sw.clouds[0].pending, "F {frames}: never struck");
            step(&mut w);
        }
        assert_eq!(w.obj(cart).unwrap().health, h0);
    }
}

#[test]
fn claim46_zero_delay_freezes_the_stage() {
    // spec: S-0513 claim 46 and S-0109 claim 2 (a zero stage delay freezes the stage; it is not replaced by 1 tick, so the cloud neither strikes nor goes)
    let r = cloud_rules(5, Some(0));
    let mut w = game(&r);
    storm::cloud(&mut w, at(0, 0));
    run(&mut w, 60);
    assert_eq!(w.sw.clouds.len(), 1);
    assert!(w.sw.clouds[0].pending);
    assert_eq!(w.sw.clouds[0].delay, 0);
}

#[test]
fn claim46_random_rate_uses_the_drawn_delay() {
    // spec: S-0513 claim 46 and S-0109 claim 3 (a random rate pair with differing delays: the stage delay is the creation's ranged draw)
    let mut r = cloud_rules(5, Some(900));
    for n in ["CLOUD1", "CLOUD2"] {
        r.sim.animations.get_mut(n).unwrap().random_rate = Some([450, 225]);
    }
    let mut w = game(&r);
    storm::cloud(&mut w, at(0, 0));
    let c = w.sw.clouds[0].clone();
    assert!((2..=4).contains(&c.delay), "drawn delay {}", c.delay);
    let strike = until(&mut w, 200, |w| !w.sw.clouds[0].pending) - 1;
    assert_eq!(strike, c.created + (5 / 2 + 1) * c.delay + 1);
}

#[test]
fn claim46_frame_count_is_the_image_not_twice_the_middle_field() {
    // spec: S-0513 claim 46 (the threshold uses the frame count F of the current image: odd counts and a middle field from another sprite do not give F = 2μ)
    let mut r = cloud_rules(5, Some(900));
    for n in ["CLOUD1", "CLOUD2"] {
        r.sim.animations.get_mut(n).unwrap().middle_frame = Some(10);
    }
    let mut w = game(&r);
    storm::cloud(&mut w, at(0, 0));
    let c = w.sw.clouds[0].clone();
    let strike = until(&mut w, 200, |w| !w.sw.clouds[0].pending) - 1;
    assert_eq!(strike, c.created + 3 + 1);
}

#[test]
fn claim46_normalised_type_takes_the_speed_normalised_delay() {
    // spec: S-0513 claim 46 and S-0109 claim 4 (a normalised type's delay is N(d, s) for the game speed s when it is chosen)
    let mut r = cloud_rules(5, Some(225));
    for n in ["CLOUD1", "CLOUD2"] {
        r.sim.animations.get_mut(n).unwrap().normalized = Some(true);
    }
    let mut w = game(&r);
    storm::cloud(&mut w, at(0, 0));
    let expected = glasnost_sim::infantry::normalised(4, w.game_speed);
    assert_eq!(w.sw.clouds[0].delay, expected);
}

#[test]
fn claim21_f4_height_follows_the_header_canvas_height() {
    // spec: S-0513 F4 (the cloud's height term comes from the first bolt image's header canvas height H)
    let mut r = ruleset();
    r.sim.animations.get_mut("BOLT").unwrap().canvas_height = Some(100);
    let mut w = game(&r);
    storm::cloud(&mut w, at(1, 1));
    let lvl = i32::from(w.cells.cell(at(1, 1)).unwrap().level);
    assert_eq!(w.sw.clouds[0].pos.z, lvl * 104 + 344);
}

#[test]
fn claim21_missing_images_make_no_cloud() {
    // spec: S-0513 claim 21 and F4 (a selected cloud type with no image, or a nonempty bolt list whose first type has no image: no cloud; the selection draw was taken, no creation draw follows)
    for broken in ["CLOUD1", "BOLT"] {
        let mut r = ruleset();
        if broken == "BOLT" {
            r.sim.animations.get_mut("BOLT").unwrap().canvas_height = None;
        } else {
            for n in ["CLOUD1", "CLOUD2"] {
                r.sim.animations.get_mut(n).unwrap().frame_count = None;
            }
        }
        let mut w = game(&r);
        let taken = w.rng.taken();
        storm::cloud(&mut w, at(0, 0));
        assert!(w.sw.clouds.is_empty(), "{broken}");
        assert_eq!(
            w.rng.taken(),
            taken + 1,
            "{broken}: only the selection draw"
        );
    }
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
