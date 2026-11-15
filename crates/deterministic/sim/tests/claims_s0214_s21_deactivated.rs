//! S-0214 revision 83 Terms (*Disabled*, *Movable*, *Able to fire*, the
//! deploy check and deploy readiness), sim issues S21 (#415, #413): a
//! vehicle deactivated by the loss of its control centre (S-0405 claims 13
//! and 20) is neither movable nor able to fire, but it is not *disabled*,
//! so table R keeps its rank and it passes the deploy check. Synthetic data
//! only. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use s21_targeting_common::*;

// spec: S-0214 Terms (Disabled: paralysed, or a vehicle playing its death sequence; movable and able to fire each ask not disabled and not deactivated, S-0405 claims 13 and 20) and table R (1 disabled; 2 not armed; 5 an armed unit whose default weapon does positive damage) and claim 1 (the highest rank is the representative)
#[test]
fn issue_415_a_deactivated_armed_vehicle_keeps_its_rank() {
    let mut w = game();
    let tank = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, -4, 0);
    let cart = put(&mut w, Kind::Vehicle, "CART", ALPHA, 2, 0);
    let on = ClickOn::Cell(at(4, 0));
    deactivate(&mut w, tank);
    assert!(!targeting::movable(&w, A, tank), "deactivated: not movable");
    assert!(
        !targeting::able_to_fire(&w, A, tank),
        "deactivated: not able to fire"
    );
    let tanks = targeting::action(&w, A, tank, on, Modifiers::default());
    let carts = targeting::action(&w, A, cart, on, Modifiers::default());
    assert_ne!(tanks, carts, "the two actions tell the representative");
    // The cart is nearer, but the armed tank's rank 5 is above the cart's 2.
    assert_eq!(cursor(&w, &[cart, tank], on), tanks);
    assert_eq!(cursor(&w, &[tank, cart], on), tanks);
}

// spec: S-0214 Terms (deploy readiness: a vehicle has it when it is not disabled and not inside a tunnel and it deploys into a building; the deactivated state is not disabled) and table VD row VD2 (deploy if the building could be placed and the vehicle has deploy readiness)
#[test]
fn issue_415_a_deactivated_vehicle_that_deploys_into_a_building_shows_deploy() {
    let mut w = game();
    let mcv = put(&mut w, Kind::Vehicle, "MCV", ALPHA, 0, 0);
    assert_eq!(on_itself(&w, mcv), Action::Deploy, "control");
    deactivate(&mut w, mcv);
    assert!(targeting::deploy_readiness(&w, mcv));
    assert!(targeting::deploy_check(&w, mcv));
    assert_eq!(on_itself(&w, mcv), Action::Deploy);
}
