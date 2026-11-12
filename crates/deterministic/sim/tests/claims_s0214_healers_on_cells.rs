//! S-0214 tables IC and VC, rows IC4 and VC1 (#210): a healing actor whose
//! cell result is *attack* gets *area guard* (infantry) or *idle*
//! (vehicles), and table M turns these into area-guard orders. Synthetic
//! data: the testkit's combat ruleset with a healing `Rifle`, carried by
//! WALKER and BUGGY.

use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, Target};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

fn healing() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.get_mut("Rifle").unwrap().damage = Some(-5);
    r.sim.vehicles.get_mut("BUGGY").unwrap().primary = Some("Rifle".into());
    r
}

fn forced() -> Modifiers {
    Modifiers {
        force_fire: true,
        ..Default::default()
    }
}

// spec: S-0214 table IC row IC4 (a healing actor whose c′ is attack: area guard) and table M part 2 (area guard on a cell); scenario T10 (line IC4, IC6, IC7, VC1)
#[test]
fn row_ic4_a_healing_infantry_unit_force_firing_on_a_cell() {
    let r = healing();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let medic = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let cell = mid(SIZE, 3, 0);
    let a = targeting::action(&w, ALPHA, medic, ClickOn::Cell(cell), forced());
    let res = targeting::click(&w, ALPHA, &[medic], ClickOn::Cell(cell), forced());
    assert_eq!(a, Action::AreaGuard);
    assert_eq!(
        res.orders,
        vec![Order::Behaviour {
            actor: medic,
            behaviour: Behaviour::AreaGuard,
            attack_move: false,
            target: Some(Target::Cell(cell)),
            dest: None,
        }]
    );
}

// spec: S-0214 table VC row VC1 (a healing actor whose c′ is attack: idle) and table M part 2 (idle: area guard with the actor as target when its first weapon heals); scenario T10 (line IC4, IC6, IC7, VC1)
#[test]
fn row_vc1_a_healing_vehicle_force_firing_on_a_cell() {
    let r = healing();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let van = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, 0, 0), 0);
    let cell = mid(SIZE, 3, 0);
    assert!(targeting::armed(&w, van));
    let a = targeting::action(&w, ALPHA, van, ClickOn::Cell(cell), forced());
    let res = targeting::click(&w, ALPHA, &[van], ClickOn::Cell(cell), forced());
    assert_eq!(a, Action::Idle);
    assert_eq!(
        res.orders,
        vec![Order::Behaviour {
            actor: van,
            behaviour: Behaviour::AreaGuard,
            attack_move: false,
            target: Some(Target::Object(van)),
            dest: None,
        }]
    );
}

// spec: S-0214 tables IC and VC (rows IC4 and VC1 ask for a healing actor: an ordinary weapon keeps attack)
#[test]
fn an_ordinary_weapon_keeps_attack_on_a_cell() {
    let r = testkit::combat_ruleset();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let rifle = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 2), 0);
    let cell = mid(SIZE, 3, 0);
    for id in [rifle, cart] {
        let a = targeting::action(&w, ALPHA, id, ClickOn::Cell(cell), forced());
        assert_eq!(a, Action::Attack);
    }
}
