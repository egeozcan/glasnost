//! Claim tests for the light of radiation fields and for data the original
//! cannot run (S-0525 claims 13 to 16, scenarios RF11 and RF12).

#[path = "rad_common/mod.rs"]
mod rad_common;

use glasnost_map::Theater;
use glasnost_sim::radiation::{self, LightSource};
use glasnost_sim::testkit;
use rad_common::*;

fn light(w: &glasnost_sim::world::World) -> LightSource {
    radiation::lights(w)[0]
}

// spec: S-0525 claims 13 and 14 and F7 (RF11 part a: with colour (0, 0, 0) and tint factor 0 the field light has intensity 500 = trunc(1000 x 0.5), no tint, radius 640 at the centre cell; after one light step it is 485 = 500 - trunc(500 / 33))
#[test]
fn the_light_has_intensity_radius_and_steps() {
    let (mut w, s) = base(|r| {
        r.sim.general.rad_color = Some([0, 0, 0]);
        r.sim.general.rad_tint_factor = Some(d("0"));
    });
    let t0 = detonate(&mut w, s, X);
    let l = light(&w);
    assert_eq!(
        (l.centre, l.radius, l.intensity, l.tint),
        (at(0, 0), 640, 500, [0; 3])
    );
    run_to(&mut w, t0 + 61);
    assert_eq!(light(&w).intensity, 485);
    run_to(&mut w, t0 + 121);
    assert_eq!(light(&w).intensity, 470);
    // The light is gone with the field.
    run_to(&mut w, t0 + 2001);
    assert!(radiation::lights(&w).is_empty());
}

// spec: S-0525 claim 14 and F7 (RF11 part b: the tint of the colour (0, 255, 0) is (0, 1000, 0), and (0, 970, 0) after one light step = trunc(1000 x 1940 / 2000))
#[test]
fn the_tint_follows_the_time_left() {
    let (mut w, s) = base(|_| {});
    let t0 = detonate(&mut w, s, X);
    assert_eq!(light(&w).tint, [0, 1000, 0]);
    run_to(&mut w, t0 + 61);
    assert_eq!(light(&w).tint, [0, 970, 0]);
}

// spec: S-0525 claim 15 and F7 (RF11 part c: on a snow-theater map the red and blue channels are -128 before the conversion: tint (-501, 1000, -501))
#[test]
fn snow_maps_subtract_red_and_blue() {
    let (mut w, s) = base(|_| {});
    w.theater = Theater::Snow;
    detonate(&mut w, s, X);
    assert_eq!(light(&w).tint, [-501, 1000, -501]);
}

// spec: S-0525 claim 14 (RF11 part d: with the light factor 0 the intensity stays 0 while the green fades to 970 at the first light step, the corrected drawing defect)
#[test]
fn a_green_only_step_is_a_light_change() {
    let (mut w, s) = base(|r| r.sim.general.rad_light_factor = Some(d("0")));
    let t0 = detonate(&mut w, s, X);
    run_to(&mut w, t0 + 61);
    let l = light(&w);
    assert_eq!((l.intensity, l.tint), (0, [0, 970, 0]));
}

// spec: S-0525 F7 (the intensity and each tint channel are capped at 2000 above, with no lower cap)
#[test]
fn the_light_is_capped_above() {
    let (mut w, s) = base(|r| {
        r.sim.general.rad_light_factor = Some(d("10"));
        r.sim.general.rad_tint_factor = Some(d("5"));
    });
    detonate(&mut w, s, X);
    assert_eq!(light(&w).intensity, 2000);
    assert_eq!(light(&w).tint, [0, 2000, 0]);
}

// spec: S-0525 claim 4 (the light is set afresh from the new level at a top-up, with the old radius)
#[test]
fn a_top_up_resets_the_light() {
    let (mut w, s) = base(|_| {});
    let t0 = detonate(&mut w, s, X);
    run_to(&mut w, t0 + 300);
    detonate(&mut w, s, X);
    let l = light(&w);
    assert_eq!(l.intensity, 925); // trunc(1850 x 0.5)
    assert_eq!(l.radius, 640);
}

// spec: S-0525 claim 13 and Edge cases (a field of negative spread reaches no cell and its light reaches no cell; the field still exists)
#[test]
fn a_negative_spread_reaches_nothing() {
    let (mut w, s) =
        base(|r| r.sim.warheads.get_mut("RADW").unwrap().cell_spread = Some(d("-1.5")));
    let v = probe(&mut w, "PROBE", at(0, 0));
    detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields.len(), 1);
    assert_eq!(light(&w).radius, 0);
    assert!(w.radiation.cells.is_empty());
    assert!(losses(&mut w, v, 100).is_empty());
}

// spec: S-0525 claim 16 (an application delay of 0: fields damage nothing, tick 0 included)
#[test]
fn an_application_delay_of_zero_damages_nothing() {
    let (mut w, _s) = base(|r| r.sim.general.rad_application_delay = Some(0));
    let v = probe(&mut w, "PROBE", at(0, 0));
    let rg = weapon(&w, "RadGun");
    radiation::detonation(&mut w, rg, at(0, 0));
    assert!(losses(&mut w, v, 100).is_empty());
}

// spec: S-0525 claim 16 (a negative application delay behaves like its absolute value: RF12 part e)
#[test]
fn a_negative_application_delay_acts_as_its_magnitude() {
    let hits = |a: i32| {
        let (mut w, s) = base(|r| r.sim.general.rad_application_delay = Some(a));
        let v = probe(&mut w, "PROBE", at(0, 0));
        run_to(&mut w, 20);
        detonate(&mut w, s, X);
        losses(&mut w, v, 60)
    };
    assert_eq!(hits(-15), hits(15));
    assert!(!hits(15).is_empty());
}

// spec: S-0525 claim 16 (a level delay of 0: no level step, so a decay step takes nothing and the radiation stays; our engine's safe rule)
#[test]
fn a_level_delay_of_zero_never_decays() {
    let (mut w, s) = base(|r| r.sim.general.rad_level_delay = Some(0));
    let t0 = detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields[0].level_steps, 0);
    run_to(&mut w, t0 + 600);
    assert_eq!(radiation::exposure(&w, at(0, 0)), 1000);
}

// spec: S-0525 claim 16 (a light delay of 0, or a duration shorter than a positive light delay: no light steps, so the intensity stays as made until the field ends while the tint still scales with the time left)
#[test]
fn without_light_steps_the_intensity_stays() {
    let (mut w, s) = base(|r| r.sim.general.rad_light_delay = Some(0));
    let t0 = detonate(&mut w, s, X);
    run_to(&mut w, t0 + 101);
    let l = light(&w);
    assert_eq!(l.intensity, 500);
    assert_eq!(l.tint[1], 1000 * 1900 / 2000);
    // Level 10 gives 20 ticks, shorter than the light delay 60.
    let (mut w, s) = base(|r| r.sim.weapons.get_mut("RadGun").unwrap().rad_level = Some(10));
    detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields[0].duration, 20);
    run_to(&mut w, 15);
    assert_eq!(light(&w).intensity, 5);
}

// spec: S-0525 claim 16 (a duration multiple of 0: the field ends at its first field update, after adding its contributions, which then stay in the cells)
#[test]
fn a_duration_multiple_of_zero_ends_the_field_at_once() {
    let (mut w, s) = base(|r| r.sim.general.rad_duration_multiple = Some(0));
    detonate(&mut w, s, X);
    testkit::run(&mut w, 2);
    assert!(w.radiation.fields.is_empty());
    assert_eq!(radiation::exposure(&w, at(0, 0)), 1000);
}

// spec: S-0525 Edge cases (a level cap of 0 or less: exposure is never positive, so fields never damage)
#[test]
fn a_level_cap_of_zero_never_damages() {
    let (mut w, s) = base(|r| r.sim.general.rad_level_max = Some(0));
    let v = probe(&mut w, "PROBE", at(0, 0));
    detonate(&mut w, s, X);
    assert!(losses(&mut w, v, 100).is_empty());
}

// spec: S-0525 Edge cases (radiation-site warhead absent: our engine's fields make no hits)
#[test]
fn without_a_site_warhead_fields_make_no_hits() {
    let (mut w, s) = base(|r| r.sim.general.rad_site_warhead = None);
    let v = probe(&mut w, "PROBE", at(0, 0));
    detonate(&mut w, s, X);
    assert!(losses(&mut w, v, 100).is_empty());
}

// spec: S-0525 Determinism: the state hash of a world with live fields survives a save and load
#[test]
fn fields_survive_a_save_and_load() {
    let (mut w, s) = base(|_| {});
    detonate(&mut w, s, X);
    testkit::run(&mut w, 100);
    let bytes = glasnost_sim::save::save(&w);
    let h = glasnost_sim::save::state_hash(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert_eq!(w2.radiation, w.radiation);
}
