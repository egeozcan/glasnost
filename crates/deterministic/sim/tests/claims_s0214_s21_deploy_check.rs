//! S-0214 revision 83 table V row V5 and table VC row VC6 (*q* or *c*′ is
//! attack and the vehicle fails the deploy check: nothing), sim issues S21
//! (#413). Only a disabled vehicle (paralysed, or playing its death
//! sequence) or one inside a tunnel with the deploy-to-fire flag fails the
//! check; a vehicle deactivated by the loss of its control centre passes
//! it, so a force-fire on a cell in range stays *attack*. Synthetic data
//! only. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use s21_targeting_common::*;

// spec: S-0214 Terms (deploy check: failed only by a disabled vehicle, or one inside a tunnel with the deploy-to-fire flag) and table VC rows GC4 (a weapon in the current slot, force-fire, the cell centre in range: attack), VC6 (attack and a failed deploy check: nothing) and VC7 (otherwise c′)
#[test]
fn issue_413_a_deactivated_vehicle_force_firing_on_a_cell_in_range_attacks() {
    let mut w = game();
    let tank = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    deactivate(&mut w, tank);
    assert!(!targeting::movable(&w, A, tank));
    assert!(targeting::deploy_check(&w, tank), "deactivated is not disabled");
    let fire = Modifiers {
        force_fire: true,
        ..Default::default()
    };
    let on = ClickOn::Cell(at(3, 0));
    assert_eq!(targeting::action(&w, A, tank, on, fire), Action::Attack);
}
