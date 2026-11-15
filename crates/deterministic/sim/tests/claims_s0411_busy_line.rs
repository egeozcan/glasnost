//! S-0411 claim 6 and S-0404 claim 7: a building or defence line with an
//! item takes no starting or queuing build order, also when two orders are
//! issued in one tick (issue 312). Synthetic data only.

mod econ_common;

use econ_common::*;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;

// spec: S-0411 claim 6 (no house has a starting or queuing build order take effect for a building or defence while that line has an item) and S-0404 claim 7 (buildings never queue)
#[test]
fn issue_312_two_building_orders_in_one_tick() {
    for same in [false, true] {
        let mut w = game(&ruleset());
        building(&mut w, "CONYARD", 0, 0, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 1_000_000);
        let hut = t(&w, Kind::Building, "HUT");
        let plant = t(&w, Kind::Building, "PLANT");
        let second = if same { hut } else { plant };
        assert!(issue(&mut w, 0, Order::Build { what: hut }));
        let taken = issue(&mut w, 0, Order::Build { what: second });
        step(&mut w);
        // Control: the same order one tick later is refused at issue.
        assert!(!issue(&mut w, 0, Order::Build { what: second }));
        let l = line(&w, 0, hut).unwrap();
        assert_eq!(l.item.as_ref().map(|i| i.t), Some(hut));
        assert!(
            l.queue.is_empty(),
            "same type {same}: second order taken at issue {taken}; building line queue {:?}",
            l.queue
        );
    }
}
