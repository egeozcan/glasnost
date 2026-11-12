//! Sensor coverage looks only at the disc's cells on the map, whatever the
//! radius (#170). Synthetic data only: the `cloak_common` test ruleset.

mod cloak_common;

use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::Duration;

use cloak_common::*;
use glasnost_map::CellXY;
use glasnost_sim::cloak;

/// House 0's sensor counts after a SEEKER of sensors-sight `r` enters play
/// at `at(dx, dy)`, made on its own thread; `None` when that has not
/// returned within ten seconds.
fn coverage(r: i32, dx: i32, dy: i32) -> Option<BTreeMap<CellXY, i32>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut rules = ruleset();
        rules
            .sim
            .vehicles
            .get_mut("SEEKER")
            .unwrap()
            .combat
            .sensors_sight = Some(r);
        let mut w = game(&rules);
        spawn(&mut w, "SEEKER", ALPHA, dx, dy);
        let counts = w
            .cloak
            .sensors
            .iter()
            .filter(|((h, _), _)| *h == ALPHA.0)
            .map(|(&(_, c), &n)| (c, n))
            .collect();
        let _ = tx.send(counts);
    });
    rx.recv_timeout(Duration::from_secs(10)).ok()
}

// spec: S-0516 Terms (sensor disc: −R ≤ dx ≤ R − 1, −R ≤ dy ≤ R − 1, dx² + dy² < R², the map's cells of it) and claim 22 (a mobile sensor's disc is added on entering play)
#[test]
fn a_huge_sensor_radius_covers_the_map_as_a_map_wide_one() {
    let whole = coverage(4 * SIZE, 0, 0).expect("a map-wide radius");
    assert!(!whole.is_empty());
    for r in [100_000, i32::MAX] {
        assert_eq!(coverage(r, 0, 0).as_ref(), Some(&whole), "R = {r}");
    }
}

// spec: S-0516 Terms (sensor disc) and claim 22: near the map's edge only the disc's cells on the map are counted, each once
#[test]
fn a_small_disc_at_the_edge_counts_its_cells_on_the_map() {
    let (dx, dy) = (SIZE / 2 - 2, 0);
    let r = 5;
    let got = coverage(r, dx, dy).expect("a small radius");
    let centre = at(dx, dy);
    let w = game(&ruleset());
    let want: BTreeMap<CellXY, i32> = cloak::sensor_disc(r)
        .into_iter()
        .map(|(x, y)| CellXY::new(centre.x + x, centre.y + y))
        .filter(|&c| w.cells.idx(c).is_some())
        .map(|c| (c, 1))
        .collect();
    assert_eq!(got, want);
}
