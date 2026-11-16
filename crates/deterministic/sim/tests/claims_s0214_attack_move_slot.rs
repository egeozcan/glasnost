//! Attack-move eligibility reads the first weapon slot, not the slot in
//! use (#357; S-0214 claim 5 and Terms, *armed*). On the synthetic combat
//! ruleset of the testkit (CRUSHER made a switching type with a weapon in
//! its first slot and nothing in its second), no game data.

use glasnost_sim::combat::weapon;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const ALPHA: HouseId = HouseId(0);

fn game(seed: u32) -> World {
    let mut r = testkit::combat_ruleset();
    let v = r.sim.vehicles.get_mut("CRUSHER").expect("the type exists");
    v.primary = Some("Gun".into());
    v.turret_count = Some(2);
    v.weapon_count = Some(2);
    v.combat.weapon_slots = Some(vec!["Gun".into(), String::new()]);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    glasnost_sim::load::new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(seed),
    )
    .expect("the scenario loads")
}

/// The combat ruleset with CRUSHER given no primary weapon field and its
/// weapon through the numbered slot list only (slot 0: Gun).
fn slot_list_game(seed: u32) -> World {
    let mut r = testkit::combat_ruleset();
    let v = r.sim.vehicles.get_mut("CRUSHER").expect("the type exists");
    v.primary = None;
    v.turret_count = Some(1);
    v.weapon_count = Some(1);
    v.combat.weapon_slots = Some(vec!["Gun".into()]);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    glasnost_sim::load::new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(seed),
    )
    .expect("the scenario loads")
}

fn of(w: &World, h: u8, kind: Kind, type_index: u32) -> Vec<ObjectId> {
    testkit::house_objects(w, h)
        .into_iter()
        .filter(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == kind && o.type_index == type_index)
        })
        .collect()
}

fn all_attack_moves(o: &[Order]) -> bool {
    o.iter().all(|o| {
        matches!(
            o,
            Order::Behaviour {
                attack_move: true,
                ..
            }
        )
    })
}

// spec: S-0214 claim 5 (eligible: infantry or a vehicle with a weapon in its first slot and no prevent-attack-move flag) and Terms (armed: a rule that names the first slot means the first slot)
#[test]
fn issue_357_eligibility_reads_the_first_slot_not_the_current_one() {
    let mut w = game(3);
    let carts = of(&w, 0, Kind::Vehicle, 0);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let am = Modifiers {
        force_fire: true,
        force_select: true,
        ..Default::default()
    };
    let mut sel = carts.clone();
    sel.push(crusher);
    let cell = testkit::cell(0, 10);
    assert!(weapon::slot_weapon(&w, crusher, 0).is_some(), "set-up");
    let r = targeting::click(&w, ALPHA, &sel, ClickOn::Cell(cell), am);
    assert_eq!(r.orders.len(), sel.len());
    assert!(all_attack_moves(&r.orders), "control: {:?}", r.orders);
    // The weapon number names the empty second slot; the first slot still holds a weapon.
    weapon::set_weapon_number(&mut w, crusher, 1);
    assert_eq!(weapon::weapon_number(&w, crusher), 1, "set-up");
    assert!(weapon::slot_weapon(&w, crusher, 0).is_some(), "set-up");
    assert!(weapon::slot_weapon(&w, crusher, 1).is_none(), "set-up");
    assert!(
        orders::attack_move_eligible(&w, crusher),
        "the application side calls the vehicle eligible"
    );
    assert!(
        targeting::shows_attack_move(&w, ALPHA, &sel, ClickOn::Cell(cell), am),
        "the cursor shows the attack-move gesture"
    );
    let r = targeting::click(&w, ALPHA, &sel, ClickOn::Cell(cell), am);
    assert_eq!(r.orders.len(), sel.len());
    assert!(
        all_attack_moves(&r.orders),
        "every selected object has a weapon in its first slot, so the gesture is on: {:?}",
        r.orders
    );
}

// spec: S-0214 claim 5 (eligible: a weapon in its first slot, whatever field supplies it; Terms, armed) and S-0213 claim 6 (an eligible actor's attack-move with a destination gives move with the memory of the destination)
#[test]
fn issue_357_the_application_side_reads_the_objects_first_slot() {
    let mut w = slot_list_game(3);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    assert!(
        weapon::slot_weapon(&w, crusher, 0).is_some(),
        "set-up: a weapon in the first slot"
    );
    assert!(
        orders::attack_move_eligible(&w, crusher),
        "eligible by its first slot"
    );
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    let c = testkit::cell(0, 5);
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: crusher,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: true,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(c)),
        }
    ));
    glasnost_sim::tick::step(&mut w);
    assert_eq!(
        w.obj(crusher).unwrap().attack_move,
        Some(glasnost_sim::world::AttackMoveMemory::Dest(
            glasnost_sim::world::Target::Cell(c)
        )),
        "the order applies as an attack-move"
    );
}
