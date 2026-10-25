//! The radar's faster forms give the plain forms' answers (S-0410 claims 6
//! and 8; M8 C4 part 4) on synthetic data: the synthetic ruleset with an
//! invented sight tower and gap generator.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;

use super::*;
use crate::testkit;

const SIZE: i32 = 40;

/// TOWER (sight 4) and GAP (gap generator, radius 3), both 1 × 1.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["TOWER", "GAP"].iter().map(|s| s.to_string()).collect(),
    );
    let b = |sight: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        b.econ.sight = Some(sight);
        b
    };
    r.sim.buildings.insert("TOWER".into(), b(4));
    let mut gap = b(0);
    gap.gap_generator = Some(true);
    gap.econ.gap_radius_in_cells = Some(3);
    r.sim.buildings.insert("GAP".into(), gap);
    r
}

fn spawn(w: &mut World, kind: Kind, name: &str, h: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, h, testkit::mid(SIZE, dx, dy), 0)
}

/// Towers of both houses, and carts of both houses crowded on a few cells,
/// so that pixels list several objects of each house.
fn world() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 5);
    spawn(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    spawn(&mut w, Kind::Building, "TOWER", 1, 3, 1);
    for (h, dx, dy) in [
        (0, 1, 1),
        (0, 1, 1),
        (1, 1, 1),
        (1, 2, 0),
        (0, 2, 0),
        (1, 2, 0),
    ] {
        spawn(&mut w, Kind::Vehicle, "CART", h, dx, dy);
    }
    testkit::run(&mut w, 2);
    w
}

/// Every pixel's winner by the composition's pixel lists and by the scan of
/// every arrival, for house `h`.
fn assert_winners_agree(w: &World, arr: &Arrivals, h: HouseId, label: &str) {
    let g = grid(w).expect("usable cells");
    let img = image(&g);
    let by_pixel = arr.by_pixel();
    let mut found = 0;
    for q in 0..img.height {
        for p in 0..img.width {
            let kept = by_pixel
                .get(&(p, q))
                .and_then(|at| winner_among(w, h, (p, q), &g, &img, at.iter().copied()));
            assert_eq!(
                kept,
                arr.winner(w, h, (p, q), &g, &img),
                "{label}: pixel ({p}, {q})"
            );
            found += usize::from(kept.is_some());
        }
    }
    assert!(found > 0, "{label}: some pixel shows an object");
}

// spec: S-0410 Terms and claim 8 (the arrivals with each object's first place in the update order looked up give the scan's arrivals, also with an object twice in the order, an object missing from it and an entry for no live object; every pixel's winner from its own list is the scan's)
#[test]
fn kept_places_and_pixel_lists_match_the_scans() {
    let mut w = world();
    let g = grid(&w).expect("usable cells");
    let img = image(&g);
    for h in [HouseId(0), HouseId(1)] {
        let (mut kept, mut plain) = (Arrivals::default(), Arrivals::default());
        let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
        for round in 0..4 {
            match round {
                // An object twice in the order (its first place counts), one
                // missing from it and an id of no object.
                1 => {
                    let o = &mut w.update.order;
                    let late = *o.last().expect("an update order");
                    o.insert(0, late);
                    o.push(ids[2]);
                    o.retain(|&x| x != ids[4]);
                    o.insert(1, ObjectId(9999));
                }
                // A tick moves the carts on and rebuilds the order.
                2 => {
                    for (i, &id) in ids.iter().enumerate().skip(2) {
                        if let Some(o) = w.obj_mut(id) {
                            o.dest = Some(crate::world::Target::Cell(testkit::mid(
                                SIZE,
                                i as i32 % 3,
                                -(i as i32 % 2),
                            )));
                        }
                    }
                    testkit::run(&mut w, 6);
                }
                3 => testkit::run(&mut w, 6),
                _ => {}
            }
            kept.observe(&w, h, &g, &img);
            plain.observe_by_scan(&w, h, &g, &img);
            let label = format!("house {}, round {round}", h.0);
            assert_eq!(kept.at, plain.at, "{label}");
            assert!(!kept.at.is_empty(), "{label}: objects are listed");
            assert_winners_agree(&w, &kept, h, &label);
        }
    }
}

// spec: S-0410 claim 6 (with no covered cell the friendly-gap test is false at every cell, as the full test answers; with gap fields the two answers agree at every cell, under both settings of quirks.shroud_on_flat_grid)
#[test]
fn friendly_gap_short_cut_matches_the_full_test() {
    let mut w = world();
    let cells = |w: &World| -> Vec<CellXY> {
        let s = 2 * SIZE + 4;
        (0..s)
            .flat_map(|y| (0..s).map(move |x| CellXY::new(x, y)))
            .filter(|&c| w.cells.idx(c).is_some())
            .collect()
    };
    let none = std::collections::BTreeSet::new();
    for c in cells(&w) {
        for flat in [true, false] {
            assert!(!in_friendly_gap(&w, c, &none, flat));
            assert!(!in_friendly_gap_cells(&w, c, &none, flat), "{c:?}");
        }
    }
    spawn(&mut w, Kind::Building, "GAP", 1, 2, 2);
    spawn(&mut w, Kind::Building, "GAP", 0, -3, 0);
    testkit::run(&mut w, 3);
    for h in [HouseId(0), HouseId(1)] {
        let covered = friendly_gap_cells(&w, h);
        assert!(!covered.is_empty(), "house {} has a friendly field", h.0);
        let mut inside = 0;
        for c in cells(&w) {
            for flat in [true, false] {
                let a = in_friendly_gap(&w, c, &covered, flat);
                assert_eq!(a, in_friendly_gap_cells(&w, c, &covered, flat), "{c:?}");
                inside += usize::from(a);
            }
        }
        assert!(inside > 0, "house {}: some cell is in a field", h.0);
    }
}
