//! S-0311 r14 claim 16 (#452) with S-0520 claim 22: a veteran crate raises
//! a building only when its type is made trainable by its data. Synthetic
//! data only: the `crate_common` test ruleset, whose building BASE gives no
//! trainable key.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_num::soft::F32;
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;

/// Picks up one veteran crate (argument 1) next to a building BASE whose
/// trainable key is `key`, and returns the building's experience
/// afterwards, with that of a vehicle beside it as the control.
fn pickup(key: Option<bool>) -> (F32, F32) {
    let mut w = world_with(false, |r| {
        r.sim.general.economy.powerups.as_mut().unwrap()[14].argument = Some(d("1"));
        r.sim.buildings.get_mut("BASE").unwrap().combat.trainable = key;
    });
    let v = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
    let b = spawn(&mut w, Kind::Building, "BASE", 0, at(2, 0));
    let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 0, at(1, 1));
    let c = at(1, 0);
    crate_at(&mut w, c, WOOD, 14);
    crates::pickup(&mut w, v, c);
    (
        w.obj(b).unwrap().combat.experience,
        w.obj(near).unwrap().combat.experience,
    )
}

// spec: S-0311 r14 claim 16 (a building of an untrainable type, one whose data gives no trainable key or clears it, is not raised by the veteran crate; one made trainable is) and S-0520 claim 22 (one step from rookie: experience 1); the vehicle beside it, trainable by default, is the control; #452
#[test]
fn claim_16_issue_452_veteran_crate_raises_only_a_trainable_building() {
    let one = F32(0x3F80_0000); // 1.0
    for (key, raised) in [(None, false), (Some(false), false), (Some(true), true)] {
        let (building, control) = pickup(key);
        assert_eq!(control, one, "key {key:?}: the control is raised");
        assert_eq!(
            building,
            if raised { one } else { F32::ZERO },
            "key {key:?}"
        );
    }
}
