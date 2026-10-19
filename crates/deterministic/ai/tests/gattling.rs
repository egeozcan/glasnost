//! The own AI and gattling weapons (S-0609, PLAN §4.7): a gattling unit
//! firing at its target is not re-ordered at another enemy (which would
//! spin it down), while a plain unit in its place is; a house that can
//! build a gattling defence builds it, up to the wanted count. The tests
//! use invented types on the synthetic skirmish.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{CombatFields, Decimal, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{gattling, testkit};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// Gives combat fields the gattling flag, six slots of SPIN and invented
/// spin settings (3 stages, up 5, down 1, thresholds 10, 30, 60).
fn spin_fields(c: &mut CombatFields) {
    c.weapon_slots = Some(vec!["SPIN".into(); 6]);
    c.is_gattling = Some(true);
    c.weapon_stages = Some(3);
    c.rate_up = Some(5);
    c.rate_down = Some(1);
    c.stage_thresholds = Some(vec![10, 30, 60]);
}

/// The fixture's rules with the weapon SPIN (range 8); the gattling
/// vehicle GATG (slots only, no primary weapon) and its plain twin PLAIN
/// (SPIN as primary), PLAIN beyond the sidebar's tech level; the unarmed
/// sturdy vehicle DUMMY; and the gattling defence GDEF (1 × 1). GATG and
/// GDEF are buildable when `buildable`.
fn rules(buildable: bool) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    r.sim.weapons.insert(
        "SPIN".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(10),
            range: Some(d("8")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some(r.sim.weapons["Gun"].warhead.clone().unwrap()),
            ..Default::default()
        },
    );
    let mut g = r.sim.vehicles["GUNR"].clone();
    g.primary = None;
    g.tech_level = Some(99);
    g.turret_count = Some(1);
    g.weapon_count = Some(6);
    let mut plain = g.clone();
    spin_fields(&mut g.combat);
    if buildable {
        g.tech_level = Some(1);
    }
    plain.primary = Some("SPIN".into());
    let mut dummy = r.sim.vehicles["GUNR"].clone();
    dummy.primary = None;
    dummy.strength = Some(30000);
    dummy.tech_level = Some(99);
    for (n, t) in [("GATG", g), ("PLAIN", plain), ("DUMMY", dummy)] {
        r.sim.vehicles.insert(n.into(), t);
        reg(&mut r, "vehicles", n);
    }
    let mut b = fixture::building(1, 1, 300, 0);
    b.turret_count = Some(1);
    spin_fields(&mut b.combat);
    if !buildable {
        b.tech_level = Some(99);
    }
    r.sim.buildings.insert("GDEF".into(), b);
    reg(&mut r, "buildings", "GDEF");
    Arc::new(SimRules::new(&r, "synthetic-gattling"))
}

fn game(rules: Arc<SimRules>) -> Game {
    let mut g = Game::new(
        rules,
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

fn reveal(w: &mut World, c: CellXY) {
    glasnost_sim::econ::shroud::area_reveal(w, HouseId(0), Pos::centre(c, 0), 2);
}

/// A world with a `unit` of house 0 eight cells south of its construction yard (once deployed),
/// attacking a DUMMY of house 1 seven cells further south (outside the
/// base's defence radius), for 40 ticks; then a second DUMMY appears
/// between the unit and the base (an intruder). Returns the world, the
/// unit, the first and the intruding enemy.
fn scene(mut g: Game, unit: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    while count(&g.world, 0, "YARD") == 0 {
        g.step();
    }
    let mut w = g.world.clone();
    let h = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Building)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(h.x, h.y + 8);
    let u = testkit::spawn(&mut w, Kind::Vehicle, unit, 0, at, 0);
    let far = CellXY::new(h.x, h.y + 15);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "DUMMY", 1, far, 0);
    reveal(&mut w, far);
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(u).unwrap();
    o.target = Some(Target::Object(a));
    o.beh.force(Behaviour::Attack, tick, stale);
    testkit::run(&mut w, 40);
    let near = CellXY::new(h.x + 1, h.y + 4);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "DUMMY", 1, near, 0);
    reveal(&mut w, near);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, u, a, b)
}

fn attack_of(orders: &[Order], who: ObjectId) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(t)),
            ..
        } if *actor == who => Some(*t),
        _ => None,
    })
}

// S-0609 claims 5 and 11, PLAN §4.7: a gattling unit firing at its target is spinning up and is not sent at an intruder; the order is counted as a kept spin.
#[test]
fn a_firing_gattling_keeps_its_target() {
    let g = game(rules(false));
    let (w, u, a, _) = scene(g, "GATG");
    assert_eq!(w.obj(u).unwrap().target, Some(Target::Object(a)));
    assert!(
        gattling::spin(&w, u).value > 0,
        "{:?}",
        gattling::spin(&w, u)
    );
    assert!(glasnost_ai::gattling::engaged(&w, u));
    let mut b = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = b.think(&w);
    assert_eq!(attack_of(&orders, u), None, "{orders:?}");
    assert!(b.special_uses().spins_kept >= 1);
}

// PLAN §4.7: the same scene with a plain unit sends it at the intruder, so the rule above is what keeps the gattling on its target.
#[test]
fn a_plain_unit_is_sent_at_the_intruder() {
    let g = game(rules(false));
    let (w, u, _, intruder) = scene(g, "PLAIN");
    assert!(!glasnost_ai::gattling::engaged(&w, u));
    let mut b = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = b.think(&w);
    assert_eq!(attack_of(&orders, u), Some(intruder), "{orders:?}");
    assert_eq!(b.special_uses().spins_kept, 0);
}

/// The number of buildings of type `name` house `h` has on the map.
fn count(w: &World, h: u8, name: &str) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            o.kind == Kind::Building && w.rules().buildings[o.type_index as usize].name == name
        })
        .count()
}

/// The number of living vehicles of type `name` house `h` has built.
fn vehicles(w: &World, h: u8, name: &str) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.health > 0 && o.kind == Kind::Vehicle)
        .filter(|o| w.rules().vehicles[o.type_index as usize].name == name)
        .count()
}

// S-0609 Terms, PLAN §4.7: a house that can build a gattling defence builds it, at most the wanted count, and counts the orders; it builds the gattling vehicle (whose weapons are only its slot list) as an army type.
#[test]
fn builds_gattling_defences() {
    let mut g = game(rules(true));
    let wanted = glasnost_ai::gattling::DEFENCES_WANTED;
    while g.world.tick < 20_000 && g.outcome().is_none() && count(&g.world, 0, "GDEF") < wanted {
        g.step();
    }
    assert_eq!(
        count(&g.world, 0, "GDEF"),
        wanted,
        "by tick {}",
        g.world.tick
    );
    assert!(
        g.special_uses()
            .iter()
            .any(|(_, u)| u.gattling_defences >= 1)
    );
    // The vehicle line builds the harvesters a large field keeps first
    // (M7b AI audit), so the gattling vehicle may come later.
    let gatg = |w: &World| (0..2).map(|h| vehicles(w, h, "GATG")).sum::<usize>();
    let until = g.world.tick + 9000;
    while g.world.tick < until && g.outcome().is_none() && gatg(&g.world) == 0 {
        g.step();
        assert!(count(&g.world, 0, "GDEF") <= wanted);
    }
    let built = gatg(&g.world);
    assert!(built > 0, "no GATG built by tick {}", g.world.tick);
}

// PLAN §4.7: no gattling defence when the house cannot build one.
#[test]
fn no_defence_when_not_buildable() {
    let mut g = game(rules(false));
    for _ in 0..6000 {
        g.step();
    }
    assert_eq!(count(&g.world, 0, "GDEF"), 0);
    assert!(
        g.special_uses()
            .iter()
            .all(|(_, u)| u.gattling_defences == 0)
    );
}
