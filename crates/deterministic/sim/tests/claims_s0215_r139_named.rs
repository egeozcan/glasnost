//! Claim tests for S-0215 r139 claim 30, sim issues S15 (#232, Q-3100
//! answered): the named area-fire deployer in *unload*. A type with the
//! named area-fire deployer flag of our format (Terms, Import mapping) aims
//! at nothing new and waits its *deploy* sequence's frame count + 1 while
//! `quirks.named_deploy_area_fire` is on; with the switch off the flag is
//! ignored. Synthetic data of `deploy_common`; every name and number is
//! invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

mod deploy_common;

use deploy_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

fn seq(w: &World, i: ObjectId) -> Option<Seq> {
    w.obj(i).unwrap().infantry.as_ref().and_then(|s| s.seq)
}

/// AREA: a deployer infantry type that does not deploy to fire, with
/// undeploy delay `delay`; its secondary is an area-fire copy of the rifle
/// when `area`. `flag` sets the named area-fire deployer flag on it and
/// `named` is `quirks.named_deploy_area_fire`.
fn rules(area: bool, flag: bool, named: bool, delay: i32) -> Ruleset {
    let mut r = ruleset();
    r.globals.quirks.named_deploy_area_fire = named;
    let mut gun = r.sim.weapons["Rifle"].clone();
    gun.area_fire = Some(area);
    r.sim.weapons.insert("AreaRifle".into(), gun);
    let mut t = r.sim.infantry["DEPINF"].clone();
    t.deploy_fire = None;
    t.undeploy_delay = Some(delay);
    t.secondary = Some("AreaRifle".into());
    t.named_area_fire_deployer = Some(flag);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("AREA".into());
    r.sim.infantry.insert("AREA".into(), t);
    r
}

/// The frame count of AREA's *deploy* sequence in `r`.
fn deploy_frames(r: &Ruleset) -> i32 {
    r.sim.infantry["AREA"].sequences["deploy"].count
}

/// Deploys an AREA unit and returns (its target, its cell, the delay of
/// the *unload* processing) once that processing has run.
fn area_unload(r: &Ruleset) -> (Option<Target>, glasnost_map::CellXY, i32) {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 4);
    let i = put(&mut w, "AREA", Kind::Infantry, 0, 0, 0);
    let from = w.tick;
    deploy(&mut w, i);
    for _ in 0..10 {
        step(&mut w);
        let o = w.obj(i).unwrap();
        if seq(&w, i) == Some(Seq::Deploy)
            && let Some((at, d)) = o.beh.last
            && at >= from
        {
            assert_eq!(o.beh.current, Some(Behaviour::Guard), "unload forces guard");
            assert_eq!(o.dest, None, "unload clears the destination");
            return (o.target, o.pos.cell(), d);
        }
    }
    panic!("unload was never processed");
}

// spec: S-0215 r139 claim 30 (an undeployed unit of the type singled out by name, whose deploy weapon has the area-fire flag, aims at nothing new; undeploy delay negative: the frame count of its deploy sequence + 1) and Terms (the named area-fire deployer: the flag of our format)
#[test]
fn issue_232_named_area_fire_deployer_aims_at_nothing_new() {
    let r = rules(true, true, true, -1);
    let (target, _, d) = area_unload(&r);
    assert_eq!(target, None, "no new target");
    assert_eq!(
        d,
        deploy_frames(&r) + 1,
        "its deploy sequence's frame count + 1"
    );
}

// spec: S-0215 r139 claim 30 (the delay table: undeployed with an undeploy delay of 0 or more gives the undeploy delay, whatever the area-fire flag and the type)
#[test]
fn issue_232_named_area_fire_deployer_with_an_undeploy_delay_waits_that_delay() {
    let (target, _, d) = area_unload(&rules(true, true, true, 33));
    assert_eq!(target, None, "no new target");
    assert_eq!(d, 33, "the undeploy delay");
}

// spec: S-0215 r139 Edge cases (quirks.named_deploy_area_fire off: the named area-fire deployer flag is ignored; the type takes its own cell and the delay of any other type, 450 ticks with a negative undeploy delay)
#[test]
fn issue_232_switch_off_ignores_the_named_flag() {
    let (target, cell, d) = area_unload(&rules(true, true, false, -1));
    assert_eq!(
        target,
        Some(Target::Cell(cell)),
        "its own cell is its target"
    );
    assert_eq!(d, 450, "450 ticks");
}

// spec: S-0215 r139 claim 30 (the named rows need the area-fire flag: without it the flagged type takes the last row, 450 ticks, and no new target)
#[test]
fn issue_232_named_flag_without_area_fire_takes_the_last_row() {
    let (target, _, d) = area_unload(&rules(false, true, true, -1));
    assert_eq!(target, None, "no new target");
    assert_eq!(d, 450, "450 ticks");
}

// spec: S-0215 r139 claim 30 (a unit of any other type takes its own cell as its target; 450 ticks)
#[test]
fn issue_232_unflagged_area_fire_deployer_takes_its_own_cell() {
    let (target, cell, d) = area_unload(&rules(true, false, true, -1));
    assert_eq!(
        target,
        Some(Target::Cell(cell)),
        "its own cell is its target"
    );
    assert_eq!(d, 450, "450 ticks");
}
