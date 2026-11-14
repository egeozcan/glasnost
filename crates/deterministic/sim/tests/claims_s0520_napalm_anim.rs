//! Claim tests for S-0520 claim 17 with its Edge cases entry, sim issues
//! S15 (#283): a napalm crate shows its position-0 animation at the midpoint
//! whether or not the rules name a flame warhead; a missing warhead skips
//! only the hit and the area damage. Synthetic data of `crate_common`; every
//! name and number is invented. Ledger: `docs/m8-claims/sim-issues.md`,
//! *Batch S15*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::World;

fn napalm_anims(flame: bool) -> (usize, i32) {
    let mut w: World = world_with(false, |r| {
        r.registries
            .entry("animations".into())
            .or_default()
            .insert(0, "FIRSTANIM".into());
        if !flame {
            r.sim.general.economy.flame_damage = None;
        }
    });
    let v = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
    let c = at(1, 0);
    crate_at(&mut w, c, WOOD, 5);
    crates::pickup(&mut w, v, c);
    let n = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "FIRSTANIM"))
        .count();
    (n, w.obj(v).unwrap().health)
}

// spec: S-0520 claim 17 (the animation at position 0 of the animation list appears at the midpoint, then the collector is hit with the flame warhead)
#[test]
fn issue_283_napalm_with_a_flame_warhead() {
    let (n, health) = napalm_anims(true);
    assert_eq!(n, 1, "the position-0 animation appears");
    assert!(health < 400, "the collector is hit");
}

// spec: S-0520 claim 17 and Edge cases (a missing flame warhead for napalm: our engine skips the hit and the area damage)
#[test]
fn issue_283_no_flame_warhead_keeps_the_animation() {
    let (n, health) = napalm_anims(false);
    assert_eq!(health, 400, "no hit without the warhead");
    assert_eq!(n, 1, "the position-0 animation still appears");
}
