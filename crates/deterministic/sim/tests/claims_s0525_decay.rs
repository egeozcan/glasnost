//! Claim tests for the decay, end, adding and residue of radiation fields
//! (S-0525 claims 2 and 4 to 7, scenarios RF3 to RF5) on synthetic data.

#[path = "rad_common/mod.rs"]
mod rad_common;

use glasnost_sim::radiation;
use glasnost_sim::testkit;
use rad_common::*;

fn with_level_delay(delay: i32) -> impl FnOnce(&mut glasnost_rules::schema::Ruleset) {
    move |r| r.sim.general.rad_level_delay = Some(delay)
}

// spec: S-0525 claims 5, 6 and 11 (RF3: the loss is 62 until the first decay step 60 ticks after the shot, then falls each 60 ticks; a hit tick that is also a step tick already shows the lower loss; no loss from the 33rd step at 1980 ticks on; the field's light is gone 2000 ticks after the shot)
#[test]
fn the_radiation_falls_in_steps_and_the_field_ends() {
    let (mut w, s) = base(|_| {});
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 30);
    let t0 = detonate(&mut w, s, X);
    assert_eq!(t0, 30);
    let mut l = losses(&mut w, v, 1);
    l.extend(losses(&mut w, v, 2100));
    let at_tick = |t: i32| l.iter().find(|&&(x, _)| x == t).map(|&(_, a)| a);
    // The shot is at tick 30: hit ticks 45, 60, 75 show 62; the first step falls at tick 90, also a hit tick.
    assert_eq!(
        [45, 60, 75, 90, 105].map(at_tick),
        [Some(62), Some(62), Some(62), Some(60), Some(60)]
    );
    assert!(
        l.windows(2).all(|p| p[1].1 <= p[0].1),
        "the loss never rises"
    );
    // The 33rd step comes at tick 30 + 1980: nothing is left to damage from then on.
    assert!(l.iter().all(|&(t, _)| t < 30 + 1980), "{:?}", l.last());
    assert!(l.iter().any(|&(t, _)| t >= 30 + 1900));
    // The field ends in the update of tick 30 + 2000.
    assert!(w.radiation.fields.is_empty());
}

// spec: S-0525 claim 6 (a field ends in the field update in which its time left reaches 0: N updates after it was made; a field made in the turn pass first updates in the next tick)
#[test]
fn a_field_ends_after_its_duration_in_updates() {
    let (mut w, s) = base(|_| {});
    let t0 = detonate(&mut w, s, X);
    run_to(&mut w, t0 + 2000);
    assert_eq!(
        w.radiation.fields.len(),
        1,
        "still there after tick t0 + 1999"
    );
    assert_eq!(w.radiation.fields[0].left, 1);
    testkit::run(&mut w, 1);
    assert!(
        w.radiation.fields.is_empty(),
        "gone after tick t0 + 2000's update"
    );
}

// spec: S-0525 claim 5 (the cell radiation falls in steps of c / n_L every D_L ticks: exposure 969 after one step, 0 after the 33rd)
#[test]
fn each_step_takes_one_part_in_the_level_steps() {
    let (mut w, s) = base(|_| {});
    let t0 = detonate(&mut w, s, X);
    assert_eq!(radiation::exposure(&w, at(0, 0)), 1000);
    run_to(&mut w, t0 + 61);
    assert_eq!(radiation::exposure(&w, at(0, 0)), 969);
    run_to(&mut w, t0 + 1981);
    assert_eq!(radiation::exposure(&w, at(0, 0)), 0);
}

// spec: S-0525 claim 7 and F4 (the field level is trunc(L x T / N): 850 when 300 of 2000 ticks have passed)
#[test]
fn the_field_level_falls_by_one_part_in_the_duration_every_tick() {
    let (mut w, s) = base(|_| {});
    let t0 = detonate(&mut w, s, X);
    assert_eq!(radiation::centre_field_level(&w, at(0, 0)), Some(1000));
    run_to(&mut w, t0 + 301);
    assert_eq!(radiation::centre_field_level(&w, at(0, 0)), Some(850));
    // No field is centred on a neighbour, whatever it is exposed to.
    assert_eq!(radiation::centre_field_level(&w, at(1, 0)), None);
    assert!(radiation::exposure(&w, at(1, 0)) > 0);
}

// spec: S-0525 claim 7 (a field's level is 0 when its duration is below 1)
#[test]
fn a_field_of_no_duration_has_level_zero() {
    let mut f = {
        let (mut w, s) = base(|_| {});
        detonate(&mut w, s, X);
        w.radiation.fields[0].clone()
    };
    f.duration = 0;
    assert_eq!(radiation::field_level(&f), 0);
}

// spec: S-0525 claims 2 and 4 (RF4 part a: shooting the centre cell of a live field adds to it: level 850 + 1000 = 1850, so the loss is 115 and falls to 57 at the first restarted level step 1500 ticks after the second shot; the light ends 3700 ticks after the second shot)
#[test]
fn a_second_shot_on_the_centre_adds_to_the_field() {
    let (mut w, s) = base(with_level_delay(1500));
    let v = probe(&mut w, "PROBE", at(0, 0));
    let t0 = detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields.len(), 1);
    run_to(&mut w, t0 + 300);
    let t1 = detonate(&mut w, s, X);
    assert_eq!(t1, t0 + 300);
    assert_eq!(w.radiation.fields.len(), 1, "one field, topped up");
    let f = &w.radiation.fields[0];
    assert_eq!((f.level, f.duration, f.left), (1850, 3700, 3700));
    let l = losses(&mut w, v, 40);
    assert_eq!(l.first().map(|x| x.1), Some(115));
    run_to(&mut w, t1 + 1530);
    let l = losses(&mut w, v, 30);
    assert_eq!(l.last().map(|x| x.1), Some(57), "{l:?}");
    run_to(&mut w, t1 + 3700);
    assert!(!w.radiation.fields.is_empty(), "alive after tick t1 + 3699");
    testkit::run(&mut w, 1);
    assert!(
        w.radiation.fields.is_empty(),
        "the light ends 3700 ticks after the second shot"
    );
}

// spec: S-0525 claim 4 and F5 (RF4 part b: a probe on the neighbour Y of two overlapping fields loses 99 before a top-up of the first field (exposure 1599) and 94 after it with the overdraw (exposure 1509), 131 with exact removal (exposure 2109))
#[test]
fn adding_removes_old_radiation_of_the_cells_of_its_square() {
    for (overdraw, want_after) in [(true, 94), (false, 131)] {
        let (mut w, s) = base(|r| {
            r.sim.general.rad_level_delay = Some(1500);
            r.globals.quirks.radiation_add_overdraw = overdraw;
        });
        let vy = probe(&mut w, "PROBE", at(1, 0));
        let t0 = detonate(&mut w, s, X);
        run_to(&mut w, t0 + 10);
        detonate(&mut w, s, (1, 0));
        assert_eq!(w.radiation.fields.len(), 2);
        let before = losses(&mut w, vy, 40);
        assert_eq!(before.first().map(|x| x.1), Some(99), "{before:?}");
        run_to(&mut w, t0 + 300);
        detonate(&mut w, s, X);
        let after = losses(&mut w, vy, 20);
        assert_eq!(
            after.first().map(|x| x.1),
            Some(want_after),
            "overdraw {overdraw}: {after:?}"
        );
    }
}

// spec: S-0525 claim 4 (RF4 part c: the spread of the top-up shot does not matter: a second shot of spread 0 leaves the reach of the first, so a probe two cells away still loses 23 just after the top-up)
#[test]
fn a_top_up_keeps_the_spread_of_the_field() {
    let (mut w, s) = base(|r| {
        r.sim.general.rad_level_delay = Some(1500);
        r.sim.warheads.insert(
            "RADW0".into(),
            glasnost_rules::sim::WarheadType {
                cell_spread: Some(d("0")),
                verses: every("0"),
                ..Default::default()
            },
        );
        let mut g = r.sim.weapons["RadGun"].clone();
        g.warhead = Some("RADW0".into());
        r.sim.weapons.insert("RadGun0".into(), g);
    });
    let v = probe(&mut w, "PROBE", at(2, 0));
    let t0 = detonate(&mut w, s, X);
    run_to(&mut w, t0 + 300);
    detonate_slot(&mut w, s, 1, X);
    assert_eq!(w.radiation.fields[0].spread, 2);
    let l = losses(&mut w, v, 20);
    assert_eq!(l.first().map(|x| x.1), Some(23), "{l:?}");
    let light = radiation::lights(&w)[0];
    assert_eq!(light.radius, 640, "the light keeps its old radius");
}

// spec: S-0525 claim 2 (a detonation in a cell that another field reaches, but that is not its centre, makes a new field; the two overlap and their contributions add up)
#[test]
fn a_detonation_in_a_reached_cell_makes_a_second_field() {
    let (mut w, s) = base(|_| {});
    detonate(&mut w, s, X);
    detonate(&mut w, s, (1, 0));
    assert_eq!(w.radiation.fields.len(), 2);
    // At Y: its own 1000 plus the first field's 600 (just below, in doubles).
    assert_eq!(radiation::exposure(&w, at(1, 0)), 1599);
    // At X: its own 1000 plus the second field's 600.
    assert_eq!(radiation::exposure(&w, at(0, 0)), 1599);
}

// spec: S-0525 claim 6 and Edge cases (RF5 part a: with no level step the field's radiation never decays or ends: losses of 62 every 15 ticks also after the light is gone at 2000 ticks; a new shot makes a new field and the loss becomes 125)
#[test]
fn an_ended_field_leaves_its_radiation_behind() {
    let (mut w, s) = base(with_level_delay(3000));
    let v = probe(&mut w, "PROBE", at(0, 0));
    let t0 = detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields[0].level_steps, 0);
    run_to(&mut w, t0 + 2100);
    assert!(w.radiation.fields.is_empty() && radiation::lights(&w).is_empty());
    let l = losses(&mut w, v, 300);
    assert!(l.iter().all(|x| x.1 == 62) && l.len() == 20, "{l:?}");
    detonate(&mut w, s, X);
    assert_eq!(w.radiation.fields.len(), 1, "a new field is made");
    let l = losses(&mut w, v, 30);
    assert_eq!(l.first().map(|x| x.1), Some(125), "{l:?}");
}

// spec: S-0525 claim 6 (RF5 part b: a field made before row 13 of a tick updates in that tick, so it ends one decay step short; its remaining centre exposure is about 50 and keeps hitting for 3 after its light is gone)
#[test]
fn a_field_made_before_row_13_ends_one_step_short() {
    let (mut w, _s) = base(|r| {
        r.sim.general.rad_level_delay = Some(100);
        r.sim.general.rad_light_delay = Some(100);
        r.sim.general.rad_application_delay = Some(1);
    });
    let v = probe(&mut w, "PROBE", at(0, 0));
    run_to(&mut w, 40);
    let rg = weapon(&w, "RadGun");
    radiation::detonation(&mut w, rg, at(0, 0));
    run_to(&mut w, 40 + 1999);
    assert_eq!(w.radiation.fields.len(), 1);
    testkit::run(&mut w, 1);
    assert!(w.radiation.fields.is_empty(), "ended in tick 40 + 1999");
    let e = radiation::exposure(&w, at(0, 0));
    assert!((49..=50).contains(&e), "{e}");
    let l = losses(&mut w, v, 5);
    assert_eq!(l.len(), 5);
    assert!(l.iter().all(|x| x.1 == 3));
}

// spec: S-0525 claim 6 and Edge cases (quirks.radiation_residue off: when a field ends each cell loses what remains of its contribution, so a cell holds radiation only from live fields)
#[test]
fn without_residue_the_end_takes_the_rest() {
    for no_steps in [false, true] {
        let (mut w, s) = base(|r| {
            r.globals.quirks.radiation_residue = false;
            if no_steps {
                r.sim.general.rad_level_delay = Some(3000);
            }
        });
        detonate(&mut w, s, X);
        run_to(&mut w, 2100);
        assert!(w.radiation.fields.is_empty());
        assert_eq!(radiation::exposure(&w, at(0, 0)), 0, "no steps: {no_steps}");
        assert_eq!(radiation::exposure(&w, at(1, 0)), 0);
    }
}
