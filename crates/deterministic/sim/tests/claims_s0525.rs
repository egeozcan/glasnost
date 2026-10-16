//! Claim tests for radiation fields (S-0525) on synthetic data: invented
//! types, weapons and warheads, every number our own. The expected values
//! are the spec's own measurement scenarios (RF1 to RF12), whose values are
//! invented there too.

#[path = "rad_common/mod.rs"]
mod rad_common;

use glasnost_sim::world::ObjectId;
use rad_common::*;

// spec: S-0525 claim 1 (a fired shot of a radiation weapon leaves radiation at its detonation cell: a field of spread 2, level 1000, duration N = 2000 and time left 2000)
#[test]
fn a_fired_shot_leaves_a_field_at_its_detonation_cell() {
    let (mut w, s) = base(|_| {});
    detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields.len(), 1);
    let f = &w.radiation.fields[0];
    assert_eq!(f.centre, at(0, 0));
    assert_eq!(
        (f.spread, f.level, f.duration, f.left),
        (2, 1000, 2000, 2000)
    );
    // spec: S-0525 F2 (n_L = trunc(2000 / 60) = 33)
    assert_eq!(f.level_steps, 33);
}

// spec: S-0525 Import mapping (a radiation level of 0 or less leaves no radiation) and RF1 variant e
#[test]
fn a_weapon_of_level_zero_or_less_leaves_nothing() {
    for level in [0, -1] {
        let (mut w, s) = base(|r| r.sim.weapons.get_mut("RadGun").unwrap().rad_level = Some(level));
        detonate(&mut w, s, X);
        assert!(w.radiation.fields.is_empty() && w.radiation.cells.is_empty());
        assert!(radiation_lights(&w).is_empty());
    }
}

fn radiation_lights(w: &glasnost_sim::world::World) -> Vec<glasnost_sim::radiation::LightSource> {
    glasnost_sim::radiation::lights(w)
}

// spec: S-0525 claims 9 and 10 (RF1: losses only on ticks that are multiples of the application delay 15, each 62 = trunc(1000 x 0.0625), the first on the first multiple of 15 after the detonation)
#[test]
fn hits_come_on_the_application_cadence() {
    let (mut w, s) = base(|_| {});
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 20);
    let t0 = detonate(&mut w, s, X);
    assert_eq!(t0, 20);
    let l = losses(&mut w, v, 62);
    assert_eq!(l.first(), Some(&(30, 62)));
    assert!(l.iter().all(|&(t, x)| t % 15 == 0 && x == 62), "{l:?}");
    assert_eq!(l.len(), 4, "ticks 30, 45, 60 and 75: {l:?}");
}

// spec: S-0525 claim 10 and S-0302 claim 6 (RF1 variant b: a site warhead with versus 50 % gives 31)
#[test]
fn the_hit_applies_the_versus_value() {
    let (mut w, s) = base(|r| r.sim.warheads.get_mut("SITE").unwrap().verses = every("0.5"));
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    assert_eq!(losses(&mut w, v, 15).first(), Some(&(30, 31)));
}

// spec: S-0525 claim 10 and S-0302 claim 5 (RF1 variant c: a level factor of 0 with positive exposure still delivers the hit, which loses 1 health through the minimum)
#[test]
fn a_zero_amount_hit_is_still_delivered() {
    let (mut w, s) = base(|r| r.sim.general.rad_level_factor = Some(d("0")));
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    assert_eq!(losses(&mut w, v, 15).first(), Some(&(30, 1)));
}

// spec: S-0525 claim 9 and F6 (RF1 variant d: a level cap of 800 caps the exposure, so 50 = 800 x 0.0625 even at the centre)
#[test]
fn exposure_is_capped_at_the_level_cap() {
    let (mut w, s) = base(|r| r.sim.general.rad_level_max = Some(800));
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    assert_eq!(losses(&mut w, v, 15).first(), Some(&(30, 50)));
}

/// The health lost by each probe between now and `tick`.
fn lost_by(w: &mut glasnost_sim::world::World, ids: &[ObjectId], tick: i32) -> Vec<i32> {
    let before: Vec<i32> = ids.iter().map(|&i| health(w, i)).collect();
    run_to(w, tick);
    ids.iter()
        .zip(&before)
        .map(|(&i, b)| b - health(w, i))
        .collect()
}

// spec: S-0525 claim 3 (RF2: the first losses of seven probes are 62, 37, 12, 27, 6 and none, none; contributions fall linearly to 0 at the reach 640)
#[test]
fn radiation_spreads_over_the_cells_around_the_centre() {
    let (mut w, s) = base(|_| {});
    let cells = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1), (2, 2), (3, 0)];
    let ids: Vec<ObjectId> = cells
        .iter()
        .map(|&(x, y)| probe(&mut w, "PROBE", at(x, y)))
        .collect();
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    assert_eq!(lost_by(&mut w, &ids, 31), vec![62, 37, 12, 27, 6, 0, 0]);
}

// spec: S-0525 claim 3 and F1 (RF2 variant b: a cell one height level (104 leptons) higher is reached with the three-dimensional distance, the table root of 256^2 + 104^2 = 276, contribution 568.75, loss 35)
#[test]
fn height_differences_count_in_the_distance() {
    let (mut w, s) = base(|_| {});
    w.cells.grid.get_mut(at(1, 0)).unwrap().level = 1;
    let v = probe(&mut w, "PROBE", at(1, 0));
    run_to(&mut w, 20);
    detonate(&mut w, s, X);
    assert_eq!(losses(&mut w, v, 15).first(), Some(&(30, 35)));
}

// spec: S-0525 claim 3 (RF2 variant c: a spread of 2.9 truncates to 2 and gives the same pattern; spread 0 reaches only the centre cell)
#[test]
fn the_spread_is_truncated_to_whole_cells() {
    for (spread, want) in [("2.9", vec![62, 37, 12, 0]), ("0", vec![62, 0, 0, 0])] {
        let (mut w, s) =
            base(|r| r.sim.warheads.get_mut("RADW").unwrap().cell_spread = Some(d(spread)));
        let ids: Vec<ObjectId> = [(0, 0), (1, 0), (2, 0), (3, 0)]
            .iter()
            .map(|&(x, y)| probe(&mut w, "PROBE", at(x, y)))
            .collect();
        run_to(&mut w, 20);
        detonate(&mut w, s, X);
        assert_eq!(lost_by(&mut w, &ids, 31), want, "spread {spread}");
    }
}
