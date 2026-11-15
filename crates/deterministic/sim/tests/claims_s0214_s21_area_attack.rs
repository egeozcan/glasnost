//! S-0214 revision 83 table I value *v* row v3 (the actor's deploy weapon
//! fires over an area: area attack), sim issues S21 (#412), with row v2
//! (route mode held: no-deploy) ahead of it and table M (area attack:
//! unload). The deploy weapon is S-0215's: the secondary weapon, or the
//! primary for a type with the spray-attack flag. Synthetic data only.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use s21_targeting_common::*;

fn unload_only(orders: &[Order], actor: glasnost_sim::world::ObjectId) -> bool {
    matches!(orders, [Order::Behaviour { actor: a, behaviour: Behaviour::Unload, .. }] if *a == actor)
}

// spec: S-0214 table I value v rows v3 (the actor's deploy weapon fires over an area: area attack) and v4 (deploys in place), Scenario row v3 (area attack, cursor; an unload order) and table M (area attack: unload); S-0215 claim 30 and Terms (the deploy weapon: the secondary, or the primary for a spray-attack type) and S-0214 Import mapping (Weapon AreaFire)
#[test]
fn issue_412_an_area_fire_deploy_weapon_gives_area_attack() {
    let mut w = game();
    let plain = put(&mut w, Kind::Infantry, "WALKER", ALPHA, -4, 0);
    assert_ne!(on_itself(&w, plain), Action::AreaAttack, "control");
    for (name, dx) in [("AREA", 0), ("SPRAY", 2)] {
        let i = put(&mut w, Kind::Infantry, name, ALPHA, dx, 0);
        assert_eq!(on_itself(&w, i), Action::AreaAttack, "{name}");
        let orders = targeting::click(&w, A, &[i], ClickOn::Object(i), Modifiers::default()).orders;
        assert!(unload_only(&orders, i), "{name}: {orders:?}");
    }
    // A spray-attack type's deploy weapon is its primary, not the area-fire secondary.
    let i = put(&mut w, Kind::Infantry, "SPRAY2", ALPHA, 4, 0);
    assert_ne!(on_itself(&w, i), Action::AreaAttack, "SPRAY2");
}

// spec: S-0214 table I value v row v2 (route mode held: no-deploy, ahead of row v3) and Scenario row v2
#[test]
fn issue_412_route_mode_comes_before_area_attack() {
    let mut w = game();
    let i = put(&mut w, Kind::Infantry, "AREA", ALPHA, 0, 0);
    let on = ClickOn::Object(i);
    let a = on_itself(&w, i);
    assert_eq!(a, Action::AreaAttack);
    assert_eq!(targeting::in_route_mode(&w, &[i], on, a), Action::NoDeploy);
}
