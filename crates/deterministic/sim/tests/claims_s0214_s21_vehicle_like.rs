//! S-0214 revision 83 Terms (*vehicle-like building*, *movable*) with
//! S-0415's Import mapping (`UndeploysInto`: a name that matches no vehicle
//! type makes the building not a packing building), sim issues S21 (#417).
//! Synthetic data only. Ledger: `docs/m8-claims/sim-issues.md`,
//! *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting;
use s21_targeting_common::*;

// spec: S-0214 Terms (a vehicle-like building: its type undeploys into a vehicle and has a 1 × 1 foundation; a building is movable when it is rally-capable or its type undeploys into a vehicle) and claim 19 (an ordinary band box takes vehicle-like buildings, not ordinary ones); S-0415 Import mapping (UndeploysInto: a name that matches no vehicle type does not make a packing building)
#[test]
fn issue_417_a_building_that_undeploys_into_no_vehicle_type_is_not_vehicle_like() {
    let mut w = game();
    let pack = put(&mut w, Kind::Building, "PACK", ALPHA, -3, 0);
    let hut = put(&mut w, Kind::Building, "HUT", ALPHA, 3, 0);
    // Control: the name names a vehicle type.
    assert!(targeting::counts_as_vehicle(&w, pack));
    assert!(targeting::movable(&w, A, pack));
    assert!(targeting::box_eligible(&w, A, &[], pack));
    assert!(glasnost_sim::structures::engineer::vehicle_like(&w, pack));
    assert!(!targeting::counts_as_vehicle(&w, hut));
    assert!(!targeting::movable(&w, A, hut));
    assert!(!targeting::box_eligible(&w, A, &[], hut));
    assert!(!glasnost_sim::structures::engineer::vehicle_like(&w, hut));
}
