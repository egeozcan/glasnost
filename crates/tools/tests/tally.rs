//! The AI lane's counters of transport loads, hovering transports and
//! superweapon launches by kind (M6c step 2b and 2c), on the synthetic
//! skirmish (no game data).

use glasnost_ai::{fixture, game::Game};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::charge;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};
use glasnost_tools::skirmish_lane::Tally;

fn of(w: &World, h: u8, name: &str) -> Vec<glasnost_sim::world::ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            let r = w.rules();
            match o.kind {
                Kind::Building => r.buildings[o.type_index as usize].name == name,
                Kind::Vehicle => r.vehicles[o.type_index as usize].name == name,
                _ => false,
            }
        })
        .map(|o| o.id)
        .collect()
}

// Riflemen carried over the strait by a hovering transport are counted as
// loads onto a hovering transport, and their unloading as unloads
// (S-0507, S-0503).
#[test]
fn counts_the_loads_onto_a_hovering_transport() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map_strait(),
        "synthetic-strait",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the strait skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let free = |w: &World, c: glasnost_map::CellXY| {
        w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
    };
    let cells: Vec<glasnost_map::CellXY> = (0..12)
        .map(|d| glasnost_map::CellXY::new(yard.x, yard.y + 5 + d))
        .filter(|&c| free(&g.world, c))
        .take(4)
        .collect();
    assert_eq!(cells.len(), 4);
    testkit::spawn(&mut g.world, Kind::Vehicle, "LIFT", 0, cells[0], 0);
    for &c in &cells[1..] {
        testkit::spawn(&mut g.world, Kind::Infantry, "WALKER", 0, c, 0);
    }
    let limit = g.world.tick + 6000;
    while g.world.tick < limit && tally.unloads == 0 {
        g.step();
        tally.observe(&g.world);
    }
    assert!(tally.loads > 0, "riflemen boarded: {tally:?}");
    assert_eq!(
        tally.hover_loads, tally.loads,
        "onto the hovering transport"
    );
    assert!(tally.unloads > 0, "they were let out: {tally:?}");
}

// A launch is counted when the ready mark falls, by the kind of the type
// (S-0512, S-0513).
#[test]
fn counts_a_launch_by_kind() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let at = (2..12)
        .map(|d| glasnost_map::CellXY::new(yard.x - d, yard.y))
        .find(|&c| g.world.cells.usable(c) && g.world.objects.values().all(|o| o.pos.cell() != c))
        .expect("a free cell for the silo");
    testkit::spawn(&mut g.world, Kind::Building, "SILO", 0, at, 0);
    let limit = g.world.tick + 3000;
    while g.world.tick < limit && !charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready) {
        g.step();
        tally.observe(&g.world);
    }
    assert!(charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready));
    assert_eq!(tally.launches, 0);
    // The fire order a person gives, at the enemy's start.
    let enemy = fixture::at(fixture::START_DX, 0);
    assert!(g.issue(HouseId(0), Order::FireSuperweapon { sw: 0, cell: enemy }));
    for _ in 0..3 {
        g.step();
        tally.observe(&g.world);
    }
    assert_eq!(tally.launches, 1, "{tally:?}");
    assert_eq!(tally.launches_by_kind.get("Missile"), Some(&1));
}

// Units that go berserk are counted once each, when they go berserk, and
// the house-ticks with a hostile-intent line are counted (S-0610 claims 4
// and 10).
#[test]
fn counts_berserk_units_and_intent_lines() {
    let mut r = fixture::ruleset();
    let mut eye = fixture::building(1, 1, 150, 0);
    eye.econ.psychic_detection_radius = Some(5);
    eye.tech_level = Some(99);
    r.sim.buildings.insert("EYE".into(), eye);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("EYE".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    tally.observe(&g.world);
    assert_eq!((tally.berserked, tally.intent_ticks), (0, 0));
    let unit = g
        .world
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(1)) && o.kind != Kind::Building && o.on_map)
        .map(|o| o.id)
        .expect("a unit of house 1");
    glasnost_sim::psychic::enter(&mut g.world, unit, 10_000);
    let near = fixture::at(4, 4);
    let eye = testkit::spawn(&mut g.world, Kind::Building, "EYE", 0, near, 0);
    g.world.obj_mut(unit).unwrap().dest = Some(glasnost_sim::world::Target::Object(eye));
    for _ in 0..3 {
        tally.observe(&g.world);
    }
    assert_eq!(tally.berserked, 1, "{tally:?}");
    assert_eq!(tally.intent_ticks, 3, "{tally:?}");
}

// A takeover is counted once, when the control link forms; the most
// objects a house holds at once and its controllers are kept per house
// (S-0601 claims 1, 5 and 6).
#[test]
fn counts_takeovers_and_controllers() {
    use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
    let mut r = fixture::ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            mind_control: Some(true),
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RAY".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(40),
            range: Some(Decimal("2".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut c = r.sim.vehicles["GUNR"].clone();
    c.primary = Some("RAY".into());
    c.secondary = None;
    c.tech_level = Some(99);
    r.sim.vehicles.insert("CTRL".into(), c);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("CTRL".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let ctrl = testkit::spawn(&mut g.world, Kind::Vehicle, "CTRL", 0, fixture::at(4, 4), 0);
    tally.observe(&g.world);
    assert_eq!(tally.taken, 0);
    assert_eq!(tally.controllers[0].len(), 1, "{tally:?}");
    let unit = g
        .world
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(1)) && o.kind != Kind::Building && o.on_map)
        .map(|o| o.id)
        .expect("a unit of house 1");
    assert!(glasnost_sim::mind::take(&mut g.world, ctrl, unit));
    for _ in 0..3 {
        tally.observe(&g.world);
    }
    assert_eq!(tally.taken, 1, "{tally:?}");
    assert_eq!(tally.controlled_max[0], 1, "{tally:?}");
    assert_eq!(tally.controlled_max[1], 0, "{tally:?}");
}

// A house's gattling types are kept per house, and a tick counts once per
// house however many of its gattlings stand at the top stage (S-0609
// Terms and claim 2).
#[test]
fn counts_gattling_types_and_top_stage_ticks() {
    let mut r = fixture::ruleset();
    let mut c = r.sim.vehicles["GUNR"].clone();
    c.primary = None;
    c.tech_level = Some(99);
    c.turret_count = Some(1);
    c.weapon_count = Some(6);
    c.combat.weapon_slots = Some(vec!["Gun".into(); 6]);
    c.combat.is_gattling = Some(true);
    c.combat.weapon_stages = Some(3);
    c.combat.rate_up = Some(5);
    c.combat.rate_down = Some(1);
    c.combat.stage_thresholds = Some(vec![10, 30, 60]);
    r.sim.vehicles.insert("GATG".into(), c);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("GATG".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let a = testkit::spawn(&mut g.world, Kind::Vehicle, "GATG", 0, fixture::at(4, 4), 0);
    let b = testkit::spawn(&mut g.world, Kind::Vehicle, "GATG", 0, fixture::at(5, 4), 0);
    tally.observe(&g.world);
    assert_eq!(tally.gattling_types[0].len(), 1, "{tally:?}");
    assert_eq!(tally.gattling_types[1].len(), 0, "{tally:?}");
    assert_eq!(tally.top_ticks, [0, 0]);
    for id in [a, b] {
        let o = g.world.obj_mut(id).unwrap();
        o.combat.spin = 60;
        o.combat.gattling_stage = 2;
    }
    tally.observe(&g.world);
    tally.observe(&g.world);
    assert_eq!(tally.top_ticks, [2, 0], "{tally:?}");
}
