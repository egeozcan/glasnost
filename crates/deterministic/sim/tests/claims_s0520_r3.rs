//! Claim tests for S-0520 revision 3 (answers Q-0987): the stored contents of
//! random crates (claim 8, scenario CR20) and the direct placement of an
//! awarded unit with its fallback search (claim 13, scenario CR9). Synthetic
//! data only. The layered power-up table is tested in the importer
//! (`crates/convert/src/sim_crate_tests.rs`).

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};

fn armed(w: &mut World) -> ObjectId {
    spawn(w, Kind::Vehicle, "ARMED", 0, at(0, 0))
}

fn goodie(w: &World) -> Option<ObjectId> {
    let ti = w.rules().index_of(Kind::Vehicle, "GOODIE");
    w.objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && Some(o.type_index) == ti)
        .map(|o| o.id)
}

// spec: S-0520 claim 8 and Terms, scenario CR20 (revision 3): trigger crates store the number slot as S-0524 claim 4 says: 19 stores 19, exactly 20 stores the random value 255, 21 stores 21, 255 stores 255 and 276 stores 20; every stored value outside 0 to 18 picks at random at pickup
#[test]
fn trigger_crates_store_exact_contents() {
    for (slot, want) in [(19, 19u8), (20, 255), (21, 21), (255, 255), (276, 20)] {
        let mut w = world_with(false, |_| {});
        let c = at(2, 2);
        assert!(crates::place_trigger(&mut w, c, slot), "slot {slot}");
        assert_eq!(w.cells.cell(c).unwrap().overlay_data, want, "slot {slot}");
    }
}

// spec: S-0520 claim 8 and Terms (revision 3): new random crates (a death crate included) store contents 255, not 19 or 20
#[test]
fn random_crates_store_255() {
    let mut w = world_with(false, |_| {});
    let c = at(3, 3);
    assert!(crates::place_death(&mut w, c, false));
    let place = *w.crates.places.values().next().expect("a crate place");
    assert_eq!(w.cells.cell(place.cell).unwrap().overlay_data, 255);
    // The money variant of a building's crate beneath stores money (0), which is not random.
    let mut w = world_with(false, |_| {});
    assert!(crates::place_death(&mut w, c, true));
    let place = *w.crates.places.values().next().expect("a crate place");
    assert_eq!(w.cells.cell(place.cell).unwrap().overlay_data, 0);
}

// spec: S-0520 claim 13 (revision 3, answers Q-0987): the awarded unit is first offered a direct placement at the crate cell centre, which accepts only a clear entry outcome; with the cell empty it stands in the crate cell
#[test]
fn an_empty_crate_cell_takes_the_unit_directly() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let c = at(1, 0);
    crate_at(&mut w, c, WOOD, 1);
    crates::pickup(&mut w, v, c);
    let g = goodie(&w).expect("the unit");
    assert_eq!(w.obj(g).unwrap().pos.cell(), c);
}

// spec: S-0520 claim 13 (revision 3, scenario CR9): a stationary vehicle of the collecting house in the crate cell refuses the direct placement, traversable land notwithstanding; the nearby search then finds another cell, which is offered the same direct test once
#[test]
fn a_stationary_friend_refuses_the_direct_placement() {
    let mut w = world_with(false, |_| {});
    let v = armed(&mut w);
    let c = at(1, 0);
    spawn(&mut w, Kind::Vehicle, "UNARMED", 0, c);
    crate_at(&mut w, c, WOOD, 1);
    crates::pickup(&mut w, v, c);
    let g = goodie(&w).expect("placed by the fallback search");
    assert_ne!(w.obj(g).unwrap().pos.cell(), c);
}

// spec: S-0520 claim 24 and Terms (revision 3, scenario CR18): the resource stage-picture count is the flat run's length 12, so a crate raises an existing stage only up to 11, and a stage above 11 is left unchanged, not lowered
#[test]
fn the_stage_cap_is_one_below_the_picture_count() {
    for (from, to) in [(9u8, 10u8), (10, 11), (11, 11), (12, 12), (14, 14)] {
        let mut w = world_with(false, |_| {});
        let c = at(3, 3);
        {
            let cell = w.cells.grid.get_mut(c).unwrap();
            cell.overlay = ORE;
            cell.overlay_data = from;
        }
        glasnost_sim::econ::resource::crate_stage(&mut w, c, 0);
        assert_eq!(
            w.cells.cell(c).unwrap().overlay_data,
            to,
            "stage {from} against the count 12"
        );
    }
}
