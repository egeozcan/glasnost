//! S-0214 revision 83 table VD rows VD3 and VD5 (deploy if it has deploy
//! readiness, else no-deploy), sim issues S21 (#416): the rows ask the
//! whole of deploy readiness, a vehicle transport's unload exclusions of
//! S-0507 claim 7 among them, not only the tunnel cell. Synthetic data
//! only. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

mod s21_targeting_common;

use glasnost_map::terrain::land;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action};
use s21_targeting_common::*;

// spec: S-0214 table VD row VD3 (it deploys in place: deploy if it has deploy readiness, else no-deploy) and Terms (deploy readiness: for a vehicle transport, S-0507 claim 7's unload exclusions; water is refused for a player house)
#[test]
fn issue_416_an_in_place_deployer_without_deploy_readiness_shows_no_deploy() {
    let mut w = game();
    w.houses[usize::from(ALPHA)].human = true;
    let car = put(&mut w, Kind::Vehicle, "DEPCAR", ALPHA, 0, 0);
    assert!(targeting::deploy_readiness(&w, car), "control");
    assert_eq!(on_itself(&w, car), Action::Deploy, "control");
    let c = w.obj(car).unwrap().pos.cell();
    w.cells.grid.get_mut(c).unwrap().land = land::WATER;
    assert!(!targeting::deploy_readiness(&w, car), "on water");
    assert_eq!(on_itself(&w, car), Action::NoDeploy);
}

// spec: S-0214 table VD row VD3 and Terms (deploy readiness asks not disabled; a vehicle deactivated by the loss of its control centre, S-0405 claim 20, is not disabled)
#[test]
fn issue_416_a_deactivated_in_place_deployer_keeps_deploy() {
    let mut w = game();
    let car = put(&mut w, Kind::Vehicle, "DEPCAR", ALPHA, 0, 0);
    deactivate(&mut w, car);
    assert!(targeting::deploy_readiness(&w, car));
    assert_eq!(on_itself(&w, car), Action::Deploy);
}
