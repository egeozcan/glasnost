//! S-0214's rows that read combat state (M4): armed, the target in range,
//! healing actors, the charge planter and the idle row, on the synthetic combat ruleset.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn act(w: &World, actor: ObjectId, on: ClickOn) -> Action {
    targeting::action(w, ALPHA, actor, on, Modifiers::default())
}

/// The combat ruleset with the walker's rifle healing.
fn healing_walker() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.get_mut("Rifle").unwrap().damage = Some(-5);
    r
}

// spec: S-0214 claim 6 (d) (a building is not movable: it attacks a target in range of the weapon it would use, not one out of range)
#[test]
fn claim_6d_an_immobile_actor_attacks_only_in_range() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, mid(SIZE, 0, 0), 0);
    let near = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    let far = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 20), 0);
    assert!(!targeting::movable(&w, ALPHA, tower));
    assert_eq!(act(&w, tower, ClickOn::Object(near)), Action::Attack);
    assert_ne!(act(&w, tower, ClickOn::Object(far)), Action::Attack);
}

// spec: S-0214 table GC row GC4 (an immobile actor: the cell centre in range of the weapon it would use against the cell)
#[test]
fn row_gc4_cell_in_range() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, mid(SIZE, 0, 0), 0);
    let ff = Modifiers {
        force_fire: true,
        ..Default::default()
    };
    let on = |c| targeting::action(&w, ALPHA, tower, ClickOn::Cell(c), ff);
    assert_eq!(on(mid(SIZE, 3, 0)), Action::Attack);
    assert_ne!(on(mid(SIZE, 0, 20)), Action::Attack);
}

// spec: S-0214 Terms (armed: a weapon in the slot it currently uses)
#[test]
fn armed_follows_the_weapon_records() {
    let mut r = testkit::combat_ruleset();
    r.sim.vehicles.get_mut("BUGGY").unwrap().primary = None;
    let mut w = game(&r);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let buggy = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, 2, 0), 0);
    assert!(targeting::armed(&w, cart));
    assert!(!targeting::armed(&w, buggy));
}

// spec: S-0214 Terms (healing actor: armed, the weapon it would use by default does negative damage)
// spec: S-0214 table I row I10 (a healing actor over an enemy: idle)
// spec: S-0214 table I row I11 (a healing actor over itself: area guard)
#[test]
fn rows_i10_i11_healing_actor() {
    let r = healing_walker();
    let mut w = game(&r);
    let medic = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let enemy = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    assert_eq!(act(&w, medic, ClickOn::Object(enemy)), Action::Idle);
    assert_eq!(act(&w, medic, ClickOn::Object(medic)), Action::AreaGuard);
    // The same walker with a damaging rifle attacks the enemy.
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let enemy = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    assert_eq!(act(&w, walker, ClickOn::Object(enemy)), Action::Attack);
}

// spec: S-0214 Part 2 (idle: area guard on the actor itself if its first weapon heals, else guard)
#[test]
fn idle_order_area_guards_when_the_first_weapon_heals() {
    for (r, want) in [
        (healing_walker(), Behaviour::AreaGuard),
        (testkit::combat_ruleset(), Behaviour::Guard),
    ] {
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
        let o = targeting::order(
            &w,
            ALPHA,
            id,
            Action::Idle,
            ClickOn::Object(id),
            Modifiers::default(),
            false,
        );
        assert_eq!(
            o,
            Some(Order::Behaviour {
                actor: id,
                behaviour: want,
                attack_move: false,
                target: Some(Target::Object(id)),
                dest: None,
            })
        );
    }
}

// spec: S-0214 table I row I16 (a charge planter over a building: sabotage if it accepts charges, else attack)
#[test]
fn row_i16_charge_planter() {
    for (accepts, want) in [(None, Action::Sabotage), (Some(false), Action::Attack)] {
        let mut r = testkit::combat_ruleset();
        r.sim.infantry.get_mut("WALKER").unwrap().c4 = Some(true);
        r.sim.buildings.get_mut("TOWER").unwrap().can_c4 = accepts;
        let mut w = game(&r);
        let planter = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
        let tower = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 3, 0), 0);
        assert_eq!(
            act(&w, planter, ClickOn::Object(tower)),
            want,
            "{accepts:?}"
        );
    }
}

// spec: S-0214 table I row I16 (the explosive-charges veterancy ability at its current rank makes a charge planter)
#[test]
fn row_i16_charge_ability_by_rank() {
    let mut r = testkit::combat_ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().veteran_abilities = vec!["C4".into()];
    let mut w = game(&r);
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 3, 0), 0);
    assert_eq!(act(&w, walker, ClickOn::Object(tower)), Action::Attack);
    let rz = Rounding::TowardZero;
    w.obj_mut(walker).unwrap().combat.experience = F64::from_i64(1, rz).to_f32(rz);
    assert_eq!(act(&w, walker, ClickOn::Object(tower)), Action::Sabotage);
}

// Table V row k1 (the crusher ability) needs an armed vehicle that is not
// able to fire, that is a disabled one; nothing is disabled in our engine
// yet, so the row is wired but not observable.
