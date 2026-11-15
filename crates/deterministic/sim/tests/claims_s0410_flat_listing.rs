//! Claim test for S-0410 claim 6 (sim batch S20, #377): the radar's
//! listing test of another house's object follows the renderer's
//! `quirks.shroud_on_flat_grid` (S-0110 Edge cases) through the `_on`
//! forms of `minimap::listed` and `Arrivals::observe`. Synthetic data.

mod common;

use glasnost_sim::econ::{minimap, shroud};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Pos};

const A: HouseId = HouseId(0);

// spec: S-0410 claim 6 (an object of another house is listed at a pixel when the shroud test of its point passes) and S-0110 Edge cases (quirks.shroud_on_flat_grid off: the point test reads the cell under the point)
#[test]
fn claim_6_listing_follows_the_flat_grid_switch() {
    let mut s = common::Scene::new();
    s.vehicle("TANK", common::unit_type(common::TRACK, 4, 5));
    // A plateau two levels up: the flat-grid form reads the cell one step back on both axes.
    for dy in -8..=8 {
        for dx in -8..=8 {
            s.set_cell(common::cell(dx, dy), 0, 2);
        }
    }
    for dy in -6..=6 {
        for dx in -6..=6 {
            s.place_unit(1, "TANK", common::cell(dx, dy), 0);
        }
    }
    let mut w = s.build();
    shroud::area_reveal(&mut w, A, Pos::centre(common::cell(1, 1), 208), 3);
    let g = minimap::grid(&w).unwrap();
    let img = minimap::image(&g);
    let tanks: Vec<_> = common::objects_of(&w, Kind::Vehicle);
    let mut forms_differ = 0;
    for flat in [true, false] {
        let mut arr = minimap::Arrivals::default();
        arr.observe_on(&w, A, &g, &img, flat);
        for &id in &tanks {
            let listed = minimap::listed_on(&w, A, id, &g, &img, flat);
            let observed: Vec<(i32, i32)> = arr
                .at
                .keys()
                .filter(|&&(o, _)| o == id)
                .map(|&(_, px)| px)
                .collect();
            let mut want = listed.clone();
            want.sort_unstable();
            want.dedup();
            assert_eq!(observed, want, "object {id:?}, flat grid {flat}");
            if !flat && listed != minimap::listed_on(&w, A, id, &g, &img, true) {
                forms_differ += 1;
            }
        }
    }
    assert!(
        forms_differ > 0,
        "some object is listed differently by the two forms"
    );
    // The plain forms are the flat-grid forms.
    for &id in &tanks {
        assert_eq!(
            minimap::listed(&w, A, id, &g, &img),
            minimap::listed_on(&w, A, id, &g, &img, true)
        );
    }
}
