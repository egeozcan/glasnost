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

// spec: S-0214 claim 5 (the attack-move gesture also turns the actions part 1 of table M treats as attack, here heal, into attack-move on a target; scenario T3)
#[test]
fn claim_5_attack_move_gesture_on_a_heal() {
    let r = healing_walker();
    let mut w = game(&r);
    let medic = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let friend = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 2, 0), 0);
    w.obj_mut(friend).unwrap().health -= 10;
    assert_eq!(act(&w, medic, ClickOn::Object(friend)), Action::Heal);
    let gesture = Modifiers {
        attack_move_mode: true,
        ..Default::default()
    };
    let res = targeting::click(&w, ALPHA, &[medic], ClickOn::Object(friend), gesture);
    assert_eq!(
        res.orders,
        vec![Order::Behaviour {
            actor: medic,
            behaviour: Behaviour::Attack,
            attack_move: true,
            target: Some(Target::Object(friend)),
            dest: None,
        }]
    );
}

// spec: S-0214 Terms (Allied, enemy: judged from the actor's side with the one-way alliance; whether the target's house allies the actor's is not asked)
#[test]
fn terms_allied_is_judged_from_the_actor_side() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    // House 1 allies house 0; house 0 does not ally house 1.
    w.houses[1].allies.push(ALPHA);
    w.houses[1].allies.sort();
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let b = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    // House 0's actor: house 1's cart is an enemy.
    assert_eq!(act(&w, a, ClickOn::Object(b)), Action::Attack);
    // House 1's actor: house 0's cart is allied, so no attack without force-fire.
    let beta = HouseId(1);
    let from_b = targeting::action(&w, beta, b, ClickOn::Object(a), Modifiers::default());
    assert_ne!(from_b, Action::Attack);
}

// spec: S-0214 claim 12 (an addition goes at the front when its type's primary weapon does positive damage, else at the end; a removal keeps the others' order) and S-0213 Terms (selection order)
#[test]
fn claim_12_selection_order() {
    let mut r = healing_walker();
    r.sim.vehicles.get_mut("CART").unwrap().primary = None;
    let mut w = game(&r);
    // No primary weapon, negative primary damage, and two positive ones.
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let first = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 2, 0), 0);
    let medic = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 4, 0), 0);
    let second = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 6, 0), 0);
    assert!(!targeting::positive_primary(&w, cart));
    assert!(targeting::positive_primary(&w, first));
    assert!(!targeting::positive_primary(&w, medic));
    let mut sel = Vec::new();
    for id in [cart, first, medic, second] {
        targeting::add_to_selection(&w, &mut sel, id);
    }
    assert_eq!(sel, vec![second, first, cart, medic]);
    // A toggle-select click removes the second tank, keeping the order.
    let force = Modifiers {
        force_select: true,
        ..Modifiers::default()
    };
    let r = targeting::click(&w, ALPHA, &sel, ClickOn::Object(second), force);
    assert_eq!(r.selection, Some(vec![first, cart, medic]));
    // And adds it again at the front.
    let again = targeting::click(
        &w,
        ALPHA,
        &[first, cart, medic],
        ClickOn::Object(second),
        force,
    );
    assert_eq!(again.selection, Some(vec![second, first, cart, medic]));
}

// spec: S-0214 claim 12 and Import mapping (a primary weapon of 0 damage does not do positive damage: the object goes at the end)
#[test]
fn claim_12_zero_primary_damage_is_not_positive() {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.get_mut("Gun").unwrap().damage = Some(0);
    let mut w = game(&r);
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 2, 0), 0);
    assert!(
        !targeting::positive_primary(&w, tank),
        "0 damage is not positive"
    );
}

// spec: S-0214 claim 19 (a box adds own infantry, vehicles and aircraft not yet selected; never enemy objects or ordinary buildings)
#[test]
fn claim_19_box_eligibility() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 2, 0), 0);
    let enemy = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 4, 0), 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, mid(SIZE, 8, 0), 0);
    assert!(targeting::box_eligible(&w, ALPHA, &[], cart));
    assert!(targeting::box_eligible(&w, ALPHA, &[], walker));
    assert!(
        !targeting::box_eligible(&w, ALPHA, &[cart], cart),
        "already selected"
    );
    assert!(!targeting::box_eligible(&w, ALPHA, &[], enemy));
    assert!(!targeting::box_eligible(&w, ALPHA, &[], tower));
}

// spec: S-0214 table C (attack: the in-range frames only with one selected unit or building whose weapon is in range of the target)
#[test]
fn claim_21_attack_cursor_context() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let b = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 1), 0);
    let near = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 1, 0), 0);
    let far = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 14, 0), 0);
    assert!(targeting::attack_cursor_in_range(
        &w,
        &[a],
        ClickOn::Object(near)
    ));
    assert!(!targeting::attack_cursor_in_range(
        &w,
        &[a],
        ClickOn::Object(far)
    ));
    assert!(!targeting::attack_cursor_in_range(
        &w,
        &[a, b],
        ClickOn::Object(near)
    ));
}

// spec: S-0214 table C and claim 5 (the attack-move frames when the gesture turns the representative's move or attack into an attack-move; not without the gesture)
#[test]
fn claim_21_attack_move_cursor() {
    let r = testkit::combat_ruleset();
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    let empty = ClickOn::Cell(glasnost_map::CellXY::new(SIZE / 2 + 4, SIZE / 2 + 4));
    let gesture = Modifiers {
        force_fire: true,
        force_select: true,
        ..Modifiers::default()
    };
    assert!(targeting::shows_attack_move(
        &w,
        ALPHA,
        &[a],
        empty,
        gesture
    ));
    assert!(!targeting::shows_attack_move(
        &w,
        ALPHA,
        &[a],
        empty,
        Modifiers::default()
    ));
}

// spec: S-0214 F5 (i = x mod n, unsigned remainder; none for n = 0, no value taken)
#[test]
fn f5_order_response_choice() {
    assert_eq!(targeting::order_response(7, 3), Some(1));
    assert_eq!(targeting::order_response(7, 0), None);
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

/// The combat ruleset with AATOWER (TOWER with the weapon Flak, whose
/// projectile is anti-air and not anti-ground), PULSE (TOWER as a pulse
/// cannon) and the flying aircraft BIRD.
fn b7_rules() -> Ruleset {
    use glasnost_rules::sim::{Locomotor, ProjectileType, UnitType};
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .extend(["AATOWER", "PULSE"].map(String::from));
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("BIRD".into());
    let s = &mut r.sim;
    s.projectiles.insert(
        "Flak".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            ..Default::default()
        },
    );
    let mut flak = s.weapons["Cannon"].clone();
    flak.projectile = Some("Flak".into());
    s.weapons.insert("FlakGun".into(), flak);
    let mut aa = s.buildings["TOWER"].clone();
    aa.primary = Some("FlakGun".into());
    s.buildings.insert("AATOWER".into(), aa);
    let mut pulse = s.buildings["TOWER"].clone();
    pulse.emp_cannon = Some(true);
    s.buildings.insert("PULSE".into(), pulse);
    s.aircraft.insert(
        "BIRD".into(),
        UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            ..Default::default()
        },
    );
    r
}

// spec: S-0214 table B row B7 (a building with a weapon in its first slot whose projectile cannot hit ground targets: nothing, whatever the target); Q-1761 question 3 open after S-0306 revision 33 and S-0309 revision 38 (the answer for an airborne target is the interim reading of the row)
#[test]
fn row_b7_anti_air_only_building_does_nothing() {
    let r = b7_rules();
    let mut w = game(&r);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, mid(SIZE, 0, 0), 0);
    let aa = spawn(&mut w, Kind::Building, "AATOWER", 0, mid(SIZE, 0, 4), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 2), 0);
    assert_eq!(act(&w, tower, ClickOn::Object(cart)), Action::Attack);
    assert_eq!(act(&w, aa, ClickOn::Object(cart)), Action::Nothing);
    // An aircraft on the ground and one in flight: nothing either way.
    let bird = spawn(&mut w, Kind::Aircraft, "BIRD", 1, mid(SIZE, 2, 5), 0);
    assert_eq!(act(&w, aa, ClickOn::Object(bird)), Action::Nothing);
    let p = w.obj(bird).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    w.obj_mut(bird).unwrap().pos.z = ground + 400;
    assert!(!glasnost_sim::air::on_ground(&w, bird));
    assert_eq!(act(&w, aa, ClickOn::Object(bird)), Action::Nothing);
}

// spec: S-0214 table B row B7 (a pulse cannon, or a building being sold (S-0407 Terms): nothing)
#[test]
fn row_b7_pulse_cannon_and_building_being_sold() {
    let r = b7_rules();
    let mut w = game(&r);
    let pulse = spawn(&mut w, Kind::Building, "PULSE", 0, mid(SIZE, 0, 0), 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 0, mid(SIZE, 0, 4), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 2), 0);
    assert_eq!(act(&w, pulse, ClickOn::Object(cart)), Action::Nothing);
    assert_eq!(act(&w, tower, ClickOn::Object(cart)), Action::Attack);
    let tick = w.tick;
    w.obj_mut(tower)
        .unwrap()
        .beh
        .force(Behaviour::Selling, tick, true);
    assert!(glasnost_sim::econ::tech::being_sold(w.obj(tower).unwrap()));
    assert_eq!(act(&w, tower, ClickOn::Object(cart)), Action::Nothing);
}

// spec: S-0214 Edge cases (Attack offered over a reason of the moment: with quirks.moment_reason_attack_cursor off, part (e) asks only whether a reason that gives illegal holds, at any rank, passing over every reason of the moment, reloading and out of ammunition among them), S-0304 claim 1 (a versus value of exactly zero: illegal at S-0301 claim 9 rank 9) and S-0301 claim 9 (the own AI's lasting test passes over ranks 5 to 8 in both positions)
// The on position (the default) is tested in claims_s0214_r78.rs.
#[test]
fn claim_6e_switch_off_reloading_neither_gives_nor_takes_the_attack() {
    use glasnost_sim::combat::check::{self, Answer};
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.moment_reason_attack_cursor = false;
    // The gun's warhead does nothing to class light (index 3, the cart's).
    r.sim
        .warheads
        .get_mut("HE")
        .unwrap()
        .verses
        .as_mut()
        .unwrap()[3] = glasnost_rules::sim::Decimal("0".into());
    let mut w = game(&r);
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 0, 0), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    let heavy = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, mid(SIZE, 0, 3), 0);
    let at = |w: &World, t| check::check(w, tank, Some(Target::Object(t)), 0, false);
    let lasting = |w: &World, t| check::check_lasting(w, tank, Some(Target::Object(t)), 0);
    assert_eq!(at(&w, cart), Answer::Illegal);
    assert_eq!(lasting(&w, cart), Answer::Illegal);
    assert_ne!(act(&w, tank, ClickOn::Object(cart)), Action::Attack);
    assert_eq!(act(&w, tank, ClickOn::Object(heavy)), Action::Attack);
    // Reloading: the countdown runs well past this tick.
    let tick = w.tick;
    w.obj_mut(tank).unwrap().combat.rearm_until = tick + 50;
    assert_eq!(at(&w, cart), Answer::Rearming);
    assert_eq!(lasting(&w, cart), Answer::Illegal);
    assert_ne!(lasting(&w, heavy), Answer::Illegal);
    assert_ne!(act(&w, tank, ClickOn::Object(cart)), Action::Attack);
    assert_eq!(act(&w, tank, ClickOn::Object(heavy)), Action::Attack);
    // Out of ammunition, also a reason of the moment.
    w.obj_mut(tank).unwrap().combat.rearm_until = tick;
    w.obj_mut(tank).unwrap().combat.ammo = 0;
    assert_eq!(at(&w, cart), Answer::OutOfAmmo);
    assert_eq!(lasting(&w, cart), Answer::Illegal);
    assert_ne!(act(&w, tank, ClickOn::Object(cart)), Action::Attack);
    assert_eq!(act(&w, tank, ClickOn::Object(heavy)), Action::Attack);
}
