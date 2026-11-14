//! The own AI and radiation fields (S-0525, PLAN §4.7): an idle unit that
//! radiation hurts and that stands on an irradiated cell it sees is sent to
//! the nearest clean cell; immune units and units on clean cells are left
//! alone; the AI reads only cells its house has explored.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::radiation;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with the radiation settings, the radiation weapon
/// RADG and the immune rifleman IMMUNE.
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("IMMUNE".into());
    let mut imm = r.sim.infantry["WALKER"].clone();
    imm.strength = Some(100_000);
    imm.immune_to_radiation = Some(true);
    r.sim.infantry.insert("IMMUNE".into(), imm);
    r.sim.infantry.get_mut("WALKER").unwrap().strength = Some(100_000);
    let g = &mut r.sim.general;
    g.rad_duration_multiple = Some(2);
    g.rad_application_delay = Some(15);
    g.rad_level_max = Some(10_000);
    g.rad_level_delay = Some(60);
    g.rad_light_delay = Some(60);
    g.rad_level_factor = Some(d("0.0625"));
    g.rad_light_factor = Some(d("0.5"));
    g.rad_tint_factor = Some(d("1.0"));
    g.rad_color = Some([0, 255, 0]);
    g.rad_site_warhead = Some("RADSITE".into());
    r.sim.warheads.insert(
        "RADSITE".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "RADW".into(),
        WarheadType {
            cell_spread: Some(d("2")),
            verses: Some(vec![d("0"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RADG".into(),
        WeaponType {
            damage: Some(0),
            rof: Some(10),
            range: Some(d("6")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("RADW".into()),
            rad_level: Some(1000),
            ..Default::default()
        },
    );
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game() -> Game {
    let mut g = Game::new(
        rules(),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn scene(g: &Game, name: &str, field: bool) -> (World, ObjectId, CellXY) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, name, 0, at, 0);
    // The house sees the cells around its unit.
    glasnost_sim::econ::shroud::area_reveal(
        &mut w,
        HouseId(0),
        glasnost_sim::world::Pos::centre(at, 0),
        12,
    );
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    if field {
        let rg = w.rules().combat.weapon_index["RADG"];
        radiation::detonation(&mut w, rg, at);
    }
    (w, unit, at)
}

fn moves(orders: &[Order], who: ObjectId) -> Option<CellXY> {
    orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == who => Some(*c),
        _ => None,
    })
}

// S-0525, PLAN §4.7: an idle rifleman standing in a field it sees is sent to the nearest cell that holds no radiation; the counter counts it and it gets out.
#[test]
fn leaves_a_field() {
    let g = game();
    let (mut w, unit, at) = scene(&g, "WALKER", true);
    assert!(radiation::exposure(&w, at) > 0);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let dest = moves(&orders, unit).expect("it is sent out of the field");
    assert_eq!(radiation::exposure(&w, dest), 0);
    assert_eq!(brain.special_uses().radiation, 1);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..200 {
        testkit::run(&mut w, 1);
    }
    let end = w.obj(unit).unwrap().pos.cell();
    assert_eq!(radiation::exposure(&w, end), 0, "it stands on a clean cell");
}

// S-0525, PLAN §4.7: a unit on a clean cell, or no field at all, gives no order; nor does a unit that radiation does not hurt.
#[test]
fn stays_when_it_is_safe() {
    let g = game();
    let (w, unit, _) = scene(&g, "WALKER", false);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(moves(&brain.think(&w), unit), None);
    let (w, unit, _) = scene(&g, "IMMUNE", true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(moves(&brain.think(&w), unit), None);
    assert_eq!(brain.special_uses().radiation, 0);
}

// S-0525 claim 8 (design: own AI, #48): a unit in the air above a field (at least 208 leptons above its surface, S-0525 Terms) is not hit, so it is not sent out of the field
#[test]
fn a_unit_in_the_air_above_a_field_stays() {
    let g = game();
    let (mut w, unit, at) = scene(&g, "WALKER", true);
    assert!(radiation::exposure(&w, at) > 0);
    w.obj_mut(unit).unwrap().pos.z += 400;
    assert!(glasnost_sim::combat::weapon::airborne_by_height(&w, unit));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(moves(&brain.think(&w), unit), None);
    assert_eq!(brain.special_uses().radiation, 0);
}

// S-0525, PLAN §4.7: it reads only what its house has explored: a unit on an unexplored irradiated cell is not sent anywhere.
#[test]
fn reads_only_explored_cells() {
    let g = game();
    let (mut w, unit, at) = scene(&g, "WALKER", true);
    // A far corner the house has not explored.
    let far = CellXY::new(at.x + 30, at.y);
    if !glasnost_sim::econ::shroud::explored(&w, HouseId(0), far) {
        w.obj_mut(unit).unwrap().pos = glasnost_sim::world::Pos::centre(far, 0);
        let rg = w.rules().combat.weapon_index["RADG"];
        radiation::detonation(&mut w, rg, far);
        let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
        assert_eq!(moves(&brain.think(&w), unit), None);
    }
}
