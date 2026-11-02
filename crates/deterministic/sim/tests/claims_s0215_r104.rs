//! S-0215 revision 104 claim 37 parts c and e (M8 step A3 intake 74): the
//! approach step's outcomes for a gripped unit, citing S-0607 claim 9
//! (revision 38). The step's drop (part d) and its failed search with no
//! nearby cell (S-0307 claim 17) give the unit no destination, so while it
//! has a destination they end its grip; part e's clearing removes the
//! destination directly and is no effective clear, so no jumpjet stop
//! follows it. Part d's drop and part e's clearing are tested in
//! `claims_s0215_r88.rs` and `claims_s0607_r33.rs`; this file adds the
//! failed search. Synthetic data only (the lift fixture of `lift_common`
//! with an invented short-range weapon); no stock value. Ledger:
//! `docs/m8-claims/intake74.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_rules::sim::WeaponType;
use glasnost_sim::jump;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use lift_common::*;

/// LIFTER (house 0) at (0, 0) grips VICT (house 1, computer-controlled) at
/// (6, 0), armed with an invented weapon of range 2 cells, and runs until
/// the victim is in the air (S-0502 claim 30). The victim then hunts with
/// its lifter as its target: out of range, part d keeps the target (hunt,
/// computer-controlled owner), and the grip's destination, the lifter's
/// cell, lies within m × r′ of the lifter, so part e keeps it and, the unit
/// being in the air, the step goes on to part f's search.
fn in_the_air_hunting_its_lifter() -> (World, ObjectId, ObjectId) {
    let mut w = world(|r| {
        r.sim.weapons.insert(
            "Short".into(),
            WeaponType {
                damage: Some(1),
                rof: Some(20),
                range: Some(d("2")),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some("C4".into()),
                ..Default::default()
            },
        );
        r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Short".into());
    });
    w.houses[1].human = false;
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v), "the lifter grips the victim");
    assert!(
        run_until(&mut w, 200, |w| {
            glasnost_sim::combat::weapon::in_air(w, v)
        }),
        "the victim rises into the air"
    );
    assert!(lift::grips(&w, l, v), "still gripped");
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(v).unwrap();
    o.target = Some(Target::Object(l));
    o.beh.force(Behaviour::Hunt, t, stale);
    (w, l, v)
}

/// Claims every cell for a vehicle (`on`), so that no nearby cell
/// qualifies for S-0307 claim 17's search, or releases those claims.
fn block_every_cell(w: &mut World, on: bool) {
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        w.cells.claim_vehicle(c, Layer::Ground, on);
    }
}

// spec: S-0215 claim 37 (c) (revision 104: the step's failed search with no nearby cell, S-0307 claim 17, gives a gripped unit no destination, so while the unit has a destination it ends its grip, S-0607 claim 9) with (d) (hunt, computer-controlled owner: kept), (e) (a destination within m × r′ of the target is kept; in the air the step goes on) and (f) (out of range: the search); S-0607 claim 12 (that row leaves the lifter's target as it was)
#[test]
fn a_failed_search_with_no_nearby_cell_ends_the_grip() {
    let (mut w, l, v) = in_the_air_hunting_its_lifter();
    let dest = w.obj(v).unwrap().dest;
    assert!(dest.is_some(), "the grip's destination");
    block_every_cell(&mut w, true);
    glasnost_sim::approach::step(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, None, "the search dropped the target");
    assert_eq!(o.dest, None, "with no nearby cell: no destination");
    assert!(!o.lift.grip_without_dest, "not part e's direct removal");
    // The cells free again, so that the set-down's jumpjet stop finds one.
    block_every_cell(&mut w, false);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v), "the grip ended");
    assert!(lift::seized(&w, v), "still seized");
    assert_eq!(
        w.obj(l).unwrap().target,
        Some(Target::Object(v)),
        "the lifter's target is not cleared"
    );
}

// spec: S-0215 claim 37 (e) (revision 104: part e's clearing removes the destination directly, not by an assignment of an empty destination, so it is not an effective clear in S-0502 claim 24's sense and no jumpjet stop follows it, S-0607 claim 9): the movement keeps its point and commitment, unlike the failed search's empty destination above
#[test]
fn part_e_clearing_is_no_effective_clear_and_no_stop_follows() {
    let (mut w, l, v) = in_the_air_hunting_its_lifter();
    // A destination farther from the target than m × r′: part e clears it.
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(20, 20)));
    let moving = jump::moving(&w, v);
    glasnost_sim::approach::step(&mut w, v);
    let o = w.obj(v).unwrap();
    assert!(o.lift.grip_without_dest, "part e's direct removal");
    run(&mut w, 1);
    assert!(lift::grips(&w, l, v), "the grip holds");
    assert_eq!(w.obj(v).unwrap().dest, None, "and it has no destination");
    assert_eq!(
        jump::moving(&w, v),
        moving,
        "no stop: the commitment is kept"
    );
}
