//! Claim test for S-0520 claim 4 with S-0524 claim 9, sim issues S15
//! (#284): the expiry of a crate place whose crate another rule already
//! removed leaves the cell's overlay alone. Whether such a place is freed
//! earlier is Q-3118. Synthetic data of `crate_common`; every name and number
//! is invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{crates, mp_triggers};

fn buildings_at(w: &World, c: glasnost_map::CellXY) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.pos.cell() == c)
        .map(|o| o.id)
        .collect()
}

// spec: S-0520 Terms (a crate is a cell holding a crate overlay) and claim 4 (each expiry removes one crate); S-0524 claim 9 (the foundation overlays of a missing starting building are removed)
#[test]
fn issue_284_an_expiring_place_whose_crate_is_gone_leaves_the_cell_alone() {
    let mut w = world_with(true, |_| {});
    let c = at(2, 2);
    // A starting building of house 0 on c, recorded, then lost.
    let b = spawn(&mut w, Kind::Building, "BASE", 0, c);
    mp_triggers::record_starting_buildings(&mut w);
    glasnost_sim::objects::remove(&mut w, b);
    // A crate takes the free cell, with a crate place.
    assert!(crates::place_trigger(&mut w, c, 0));
    let p = *w.crates.places.values().next().unwrap();
    assert_eq!(p.cell, c);
    assert!(crates::is_crate(&w, c));
    // Action 130 for house 0: the foundation overlay (the crate) is removed.
    mp_triggers::restore_starting_buildings(&mut w, 9013, Some(HouseId(0)));
    assert!(!crates::is_crate(&w, c), "set-up: the crate overlay is gone");
    assert_eq!(w.crates.places.len(), 1, "set-up: its place is still occupied");
    assert_eq!(*w.crates.places.values().next().unwrap(), p);
    // The replacement building (if one was placed) is lost again and another
    // overlay, not a crate, comes to the cell; it stands for a wall or a
    // resource overlay.
    for id in buildings_at(&w, c) {
        glasnost_sim::objects::remove(&mut w, id);
    }
    crate_at(&mut w, c, PLAIN, 0);
    assert!(!crates::is_crate(&w, c));
    run_to(&mut w, p.start + p.life + 1);
    assert_eq!(
        w.cells.cell(c).map(|x| x.overlay),
        Some(PLAIN),
        "the expiry of the stale place removed an overlay that is not a crate"
    );
}
