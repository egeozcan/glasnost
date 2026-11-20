//! #449 (B12): the army's round of types keeps the units it sends alone and
//! loses on use to a share of what it spends on the army. The synthetic
//! skirmish with invented types (test data of our own), all built at the
//! barracks: PLANTER (a bomb-planting warhead, S-0519 claim 1), KAMI (a
//! suicide weapon, S-0519 claim 11), CHARGE (explosive charges, S-0519
//! claim 12), DESIG (an air-strike designator, S-0519 Terms) and ENGR (an
//! armed engineer, S-0509), beside the fixture's rifleman WALKER.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::tech;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// The types the house sends alone and loses on use.
const SPENT: [&str; 5] = ["PLANTER", "KAMI", "CHARGE", "DESIG", "ENGR"];

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn weapon(warhead: &str, suicide: bool) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(40),
        range: Some(d("2")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        suicide: suicide.then_some(true),
        ..Default::default()
    }
}

/// The fixture's rules with the five types above, each a copy of WALKER
/// (buildable at the barracks, cost 200) with its one difference.
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    for (n, f) in [("PLANTH", true), ("KAMIH", false)] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                ivan_bomb: f.then_some(true),
                verses: Some(vec![d("1"); 11]),
                ..Default::default()
            },
        );
    }
    r.sim.weapons.insert("PlantW".into(), weapon("PLANTH", false));
    r.sim.weapons.insert("KamiW".into(), weapon("KAMIH", true));
    for n in SPENT {
        let mut u = r.sim.infantry["WALKER"].clone();
        match n {
            "PLANTER" => u.primary = Some("PlantW".into()),
            "KAMI" => u.primary = Some("KamiW".into()),
            "CHARGE" => u.c4 = Some(true),
            "DESIG" => {
                u.combat.airstrike_team = Some(1);
                u.combat.airstrike_team_type = Some("JET".into());
            }
            _ => u.engineer = Some(true),
        }
        r.registries
            .entry("infantry".into())
            .or_default()
            .push(n.into());
        r.sim.infantry.insert(n.into(), u);
    }
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn infantry(w: &World, n: &str) -> TypeRef {
    let index = w
        .rules()
        .infantry
        .iter()
        .position(|u| u.name == n)
        .expect("a type of the test rules") as u32;
    TypeRef {
        kind: Kind::Infantry,
        index,
    }
}

fn of(w: &World, n: &str) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.on_map && o.health > 0)
        .filter(|o| {
            let r = w.rules();
            let name = match o.kind {
                Kind::Building => r.buildings.get(o.type_index as usize).map(|b| b.name.as_str()),
                Kind::Vehicle => r.vehicles.get(o.type_index as usize).map(|u| u.name.as_str()),
                Kind::Infantry => r.infantry.get(o.type_index as usize).map(|u| u.name.as_str()),
                _ => None,
            };
            name == Some(n)
        })
        .map(|o| o.id)
        .collect()
}

/// House 0 at a think tick with its barracks and a gun vehicle, a rifleman
/// on the map, its infantry line free and credits to spare; the ever-built
/// count of each spent type set to `spent` and of WALKER to `walkers`.
fn scene(spent: i32, walkers: i32) -> World {
    let mut g = Game::new(
        rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    while g.world.tick < 9000
        && g.outcome().is_none()
        && (of(&g.world, "PILL").is_empty() || of(&g.world, "GUNR").is_empty())
    {
        g.step();
    }
    let mut w = g.world.clone();
    let pill = *of(&w, "PILL").first().expect("house 0 has a barracks");
    assert!(!of(&w, "GUNR").is_empty(), "house 0 has a gun vehicle");
    // The infantry line free: what it was building is taken off it.
    for l in w.econ.lines.lines.iter_mut() {
        if l.house == HouseId(0)
            && l.item
                .as_ref()
                .is_some_and(|i| i.t.kind == Kind::Infantry)
        {
            l.item = None;
        }
    }
    let at = w.obj(pill).unwrap().pos.cell();
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        CellXY::new(at.x - 2, at.y + 3),
        0,
    );
    w.houses[0].credits = 20000;
    let walker = infantry(&w, "WALKER");
    let mut built = vec![(walker, walkers)];
    built.extend(SPENT.iter().map(|n| (infantry(&w, n), spent)));
    for (t, n) in built {
        w.houses[0].econ.tech.ever_built.insert(t, n);
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    w
}

fn infantry_builds(w: &World, orders: &[Order]) -> Vec<String> {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::Build { what } if what.kind == Kind::Infantry => w
                .rules()
                .infantry
                .get(what.index as usize)
                .map(|u| u.name.clone()),
            _ => None,
        })
        .collect()
}

// design: own AI (#449, B12): with the house's spend on the types it sends alone (S-0519 claims 1, 11 and 12, S-0519 Terms, S-0509) already past its share of the army's spend (S-0407 claim 10's ever-built counts), the round of army types builds a fighter, not one more of them, though none of them stands on the map and the rifleman does.
#[test]
fn the_round_builds_a_fighter_once_the_spent_share_is_full() {
    let w = scene(4, 1);
    assert_eq!(tech::ever_built(&w, HouseId(0), infantry(&w, "PLANTER")), 4);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    let builds = infantry_builds(&w, &orders);
    assert!(
        builds.iter().any(|n| n == "WALKER"),
        "a rifleman is built: {builds:?}"
    );
    for n in SPENT {
        assert!(
            !builds.iter().any(|b| b == n),
            "no {n} is built past the share: {builds:?}"
        );
    }
}

// design: own AI (#449, B12): while the spend on the types sent alone is small beside the army's, the round still builds them (S-0519's special attacks keep their units): one of them, at the round's ever-built tie-break, before a rifleman that already stands on the map.
#[test]
fn the_round_still_builds_a_spent_type_within_the_share() {
    let w = scene(0, 60);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    let builds = infantry_builds(&w, &orders);
    assert!(
        builds.iter().any(|b| SPENT.contains(&b.as_str())),
        "a spent type is built within the share: {builds:?}"
    );
}
