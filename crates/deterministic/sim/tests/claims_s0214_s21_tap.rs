//! S-0214 revision 83 claim 32 with the Import mapping (vehicle types,
//! `DeploysInto`: the tap's types), sim issues S21 (#418): only a vehicle
//! type adds the building type it deploys into. Synthetic data only.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::tap_types;
use glasnost_sim::world::World;
use s21_targeting_common::*;

fn ty(w: &World, kind: Kind, name: &str) -> (Kind, u32) {
    (kind, w.rules().index_of(kind, name).unwrap())
}

// spec: S-0214 claim 32 (the tap's types: each selected object's type, together with the building type each deploys into) and Import mapping (DeploysInto is read for vehicle types)
#[test]
fn issue_418_an_infantry_deploys_into_key_adds_no_building_type() {
    let mut w = game();
    let mcv = put(&mut w, Kind::Vehicle, "MCV", ALPHA, -2, 0);
    let dinf = put(&mut w, Kind::Infantry, "DINF", ALPHA, 2, 0);
    assert_eq!(
        tap_types(&w, &[mcv]),
        vec![ty(&w, Kind::Vehicle, "MCV"), ty(&w, Kind::Building, "YARD")],
        "control"
    );
    assert_eq!(tap_types(&w, &[dinf]), vec![ty(&w, Kind::Infantry, "DINF")]);
}
