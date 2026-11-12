//! The own AI's use of the superweapon kinds other than the missile, the
//! storm and the paradrop (S-0512, S-0513; PLAN §4.7): an iron curtain on a
//! group of its armed vehicles that is fighting, a force shield on buildings
//! with a seen enemy at them, a spy plane at the enemy base, and the
//! teleport's two stages. The tests check the orders a brain gives on the
//! synthetic skirmish with invented superweapon types; the effects are
//! S-0513's and tested there.

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::CellXY;
use glasnost_rules::sim::{BuildingType, Decimal};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

/// The skirmish ruleset with one superweapon type per kind word and the
/// building that gives it (test data of our own): IRON, SHLD, SPYW, CHRN
/// (first stage, linked) and CHWP (second stage), the giving buildings
/// IRONB, SHLDB, SPYB and CHRNB (both teleport stages), none of which a
/// house can build. Every charge time is 0.1 minutes and none depends on
/// power.
fn rules() -> std::sync::Arc<glasnost_sim::rules::SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    let sw = |kind: &str, action: &str| SuperweaponType {
        kind: Some(kind.into()),
        action: Some(action.into()),
        recharge_time: Some(Decimal("0.1".into())),
        is_powered: Some(false),
        ..Default::default()
    };
    let mut chrn = sw("ChronoSphere", "ChronoSphere");
    chrn.pre_click = Some(true);
    let mut chwp = sw("ChronoWarp", "ChronoWarp");
    chwp.post_click = Some(true);
    chwp.pre_dependent = Some("ChronoSphere".into());
    for (n, t) in [
        ("IRON", sw("IronCurtain", "IronCurtain")),
        ("SHLD", sw("ForceShield", "ForceShield")),
        ("SPYW", sw("SpyPlane", "SpyPlane")),
        ("CHRN", chrn),
        ("CHWP", chwp),
    ] {
        r.sim.superweapons.insert(n.into(), t);
        reg(&mut r, "superweapons", n);
    }
    let giver = |a: &str, b: Option<&str>| {
        let mut x = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        x.econ.super_weapon = Some(a.into());
        x.econ.super_weapon2 = b.map(Into::into);
        x
    };
    for (n, b) in [
        ("IRONB", giver("IRON", None)),
        ("SHLDB", giver("SHLD", None)),
        ("SPYB", giver("SPYW", None)),
        ("CHRNB", giver("CHRN", Some("CHWP"))),
    ] {
        r.sim.buildings.insert(n.into(), b);
        reg(&mut r, "buildings", n);
    }
    std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-sw-kinds"))
}

fn free(w: &World, c: CellXY) -> bool {
    w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
}

/// A game on the synthetic map with house 0's yard standing, the giving
/// building `giver` placed for it beside the yard, and its superweapons
/// ready.
fn ready(giver: &str) -> (Game, CellXY) {
    ready_until(giver, None)
}

/// Like [`ready`], but with `until` named the loop ends in the step after
/// which that type's record is ready, before the house's own brain has
/// thought again (the brain fires a ready record at once, and its order is
/// pending in the world: a fixed wait of 400 ticks can end after that).
fn ready_until(giver: &str, until: Option<&str>) -> (Game, CellXY) {
    let mut g = Game::new(
        rules(),
        &fixture::map(),
        "synthetic-sw-kinds",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    while g.world.tick < 3000
        && !g.world.objects.values().any(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && g.world.rules().buildings[o.type_index as usize].name == "YARD"
        })
    {
        g.step();
    }
    let yard = g
        .world
        .objects
        .values()
        .find(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && g.world.rules().buildings[o.type_index as usize].name == "YARD"
        })
        .expect("the yard stands")
        .pos
        .cell();
    let at = (2..12)
        .map(|d| CellXY::new(yard.x - d, yard.y))
        .find(|&c| free(&g.world, c))
        .expect("a free cell for the giving building");
    testkit::spawn(&mut g.world, Kind::Building, giver, 0, at, 0);
    // Granted at the next recheck; ready after the charge time.
    let index = until.map(|n| {
        g.world
            .rules()
            .superweapons
            .types
            .iter()
            .position(|t| t.name == n)
            .expect("the type exists") as u32
    });
    for _ in 0..if index.is_some() { 1000 } else { 400 } {
        g.step();
        if let Some(i) = index
            && glasnost_sim::superweapons::charge::get(&g.world, HouseId(0), i)
                .is_some_and(|r| r.ready)
        {
            return (g, yard);
        }
    }
    (g, yard)
}

/// The cell `dx` east of the yard, `dy` south, as a block centre.
fn spot(yard: CellXY, dx: i32, dy: i32) -> CellXY {
    CellXY::new(yard.x + dx, yard.y + dy)
}

/// Three armed vehicles of house 0 in the block of `c`.
fn group(w: &mut World, c: CellXY) {
    for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
        testkit::spawn(
            w,
            Kind::Vehicle,
            "GUNR",
            0,
            CellXY::new(c.x + dx, c.y + dy),
            0,
        );
    }
}

fn fires(orders: &[Order], sw: &str, w: &World) -> Vec<CellXY> {
    let i = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == sw)
        .expect("the type exists") as u32;
    orders
        .iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect()
}

fn think(w: &World) -> Vec<Order> {
    let mut brain = Brain::with_plan(
        w,
        &fixture::map(),
        HouseId(0),
        glasnost_ai::brain::default_plan(),
    );
    brain.think(w)
}

// An iron curtain goes on a group of the house's armed vehicles with a
// seen enemy near; not on a quiet group, a group with its own infantry in
// the block (the curtain kills every reached infantry unit, S-0513 claim
// 23) or one with an enemy vehicle in the block (it would be protected
// too, claim 25). Enemy infantry in the block does not hold it back: it
// dies (claim 23; M7b AI round 2).
#[test]
fn curtains_a_fighting_group_of_vehicles() {
    let (g, yard) = ready("IRONB");
    let c = spot(yard, 10, 8);
    let walker = |w: &mut World, house, at: CellXY| {
        testkit::spawn(w, Kind::Infantry, "WALKER", house, at, 0)
    };

    let mut quiet = g.world.clone();
    group(&mut quiet, c);
    glasnost_sim::tick::step(&mut quiet);
    assert!(
        fires(&think(&quiet), "IRON", &quiet).is_empty(),
        "no fight, no curtain"
    );

    let mut fight = g.world.clone();
    group(&mut fight, c);
    walker(&mut fight, 1, spot(c, 6, 0));
    glasnost_sim::tick::step(&mut fight);
    assert_eq!(
        fires(&think(&fight), "IRON", &fight),
        vec![c],
        "the group is curtained"
    );

    let mut with_infantry = g.world.clone();
    group(&mut with_infantry, c);
    walker(&mut with_infantry, 1, spot(c, 6, 0));
    walker(&mut with_infantry, 0, spot(c, 1, 1));
    glasnost_sim::tick::step(&mut with_infantry);
    assert!(
        fires(&think(&with_infantry), "IRON", &with_infantry).is_empty(),
        "its own infantry would die"
    );

    let mut with_enemy_infantry = g.world.clone();
    group(&mut with_enemy_infantry, c);
    walker(&mut with_enemy_infantry, 1, spot(c, 1, 1));
    glasnost_sim::tick::step(&mut with_enemy_infantry);
    assert_eq!(
        fires(&think(&with_enemy_infantry), "IRON", &with_enemy_infantry),
        vec![c],
        "an enemy infantry unit in the block dies, so the curtain goes on"
    );

    let mut with_enemy_vehicle = g.world.clone();
    group(&mut with_enemy_vehicle, c);
    testkit::spawn(
        &mut with_enemy_vehicle,
        Kind::Vehicle,
        "GUNR",
        1,
        spot(c, 1, 1),
        0,
    );
    glasnost_sim::tick::step(&mut with_enemy_vehicle);
    assert!(
        fires(&think(&with_enemy_vehicle), "IRON", &with_enemy_vehicle).is_empty(),
        "an enemy vehicle in the block would be protected too"
    );
}

// Two armed vehicles about to fight are enough for the curtain (M7b AI
// round 2: three in one block was rare, and the curtain stood ready).
#[test]
fn curtains_a_fighting_pair_of_vehicles() {
    let (g, yard) = ready("IRONB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    for at in [c, spot(c, 1, 0)] {
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at, 0);
    }
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(c, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(
        fires(&think(&w), "IRON", &w),
        vec![c],
        "the pair is curtained"
    );

    let mut one = g.world.clone();
    testkit::spawn(&mut one, Kind::Vehicle, "GUNR", 0, c, 0);
    testkit::spawn(&mut one, Kind::Infantry, "WALKER", 1, spot(c, 6, 0), 0);
    glasnost_sim::tick::step(&mut one);
    assert!(
        fires(&think(&one), "IRON", &one).is_empty(),
        "one vehicle is not worth the curtain"
    );
}

// A building of the house that a seen enemy is attacking is curtained on
// its reference cell (S-0513 claim 25: buildings are protected; M7b AI
// round 2), and not while no enemy attacks it.
#[test]
fn curtains_a_building_under_attack() {
    let (g, yard) = ready("IRONB");
    let b = spot(yard, 12, 12);
    let mut w = g.world.clone();
    let plant = testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    let e = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let mut quiet = w.clone();
    if let Some(o) = quiet.obj_mut(e) {
        o.target = None;
    }
    assert!(
        fires(&think(&quiet), "IRON", &quiet).is_empty(),
        "no attack on the building, no curtain"
    );
    if let Some(o) = w.obj_mut(e) {
        o.target = Some(glasnost_sim::world::Target::Object(plant));
    }
    let r = w.reference_point(plant).cell();
    assert_eq!(
        fires(&think(&w), "IRON", &w),
        vec![r],
        "the attacked building is curtained"
    );
}

// A force shield goes on buildings with a seen enemy at them.
#[test]
fn shields_buildings_with_an_enemy_at_them() {
    let (g, yard) = ready("SHLDB");
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    assert!(
        fires(&think(&w), "SHLD", &w).is_empty(),
        "no enemy, no shield"
    );
    let b = spot(yard, 12, 12);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, spot(b, 1, 1), 0);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 6, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SHLD", &w);
    assert_eq!(cells.len(), 1, "one shield order: {cells:?}");
    let c = cells[0];
    assert!(
        (c.x - b.x).abs() <= 1 && (c.y - b.y).abs() <= 1,
        "the shield is on the pair of buildings: {c:?} for {b:?}"
    );
}

// S-0513 claim 31 and F3 (M7b AI audit): the shield takes the buildings of the house within its radius of the target cell, so two buildings with a gap between them are shielded together.
#[test]
fn shields_buildings_with_a_gap_between_them() {
    let (g, yard) = ready("SHLDB");
    let mut w = g.world.clone();
    let b = spot(yard, 12, 12);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, b, 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, spot(b, 5, 0), 0);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, spot(b, 8, 0), 0);
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SHLD", &w);
    assert_eq!(cells.len(), 1, "one shield order: {cells:?}");
}

// A ready spy plane goes at the enemy base.
#[test]
fn sends_the_spy_plane_at_the_enemy() {
    let (g, _) = ready_until("SPYB", Some("SPYW"));
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    let cells = fires(&think(&w), "SPYW", &w);
    assert_eq!(cells.len(), 1, "one spy plane order: {cells:?}");
    let enemy = glasnost_ai::skirmish::start_cells(&fixture::map());
    assert!(
        enemy
            .iter()
            .any(|&s| (s.x - cells[0].x).abs() <= 2 && (s.y - cells[0].y).abs() <= 2),
        "at an enemy start: {cells:?} of {enemy:?}"
    );
}

// A pair of armed vehicles is enough for a teleport source (a person sends
// two tanks too); the iron curtain wants three and a fight.
#[test]
fn teleports_a_pair_of_vehicles() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    for (dx, dy) in [(0, 0), (1, 0)] {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "GUNR",
            0,
            CellXY::new(c.x + dx, c.y + dy),
            0,
        );
    }
    glasnost_sim::tick::step(&mut w);
    assert_eq!(
        fires(&think(&w), "CHRN", &w),
        vec![c],
        "the first stage on the pair"
    );
}

// The teleport: the first stage on a group of armed vehicles, the second
// stage (only once a source is held) on a free block next to the enemy base.
#[test]
fn teleports_a_group_to_the_enemy_base() {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, 10, 8);
    let mut w = g.world.clone();
    glasnost_sim::tick::step(&mut w);
    assert!(
        fires(&think(&w), "CHRN", &w).is_empty() && fires(&think(&w), "CHWP", &w).is_empty(),
        "no group, no teleport"
    );
    group(&mut w, c);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert_eq!(
        fires(&orders, "CHRN", &w),
        vec![c],
        "the first stage on the group"
    );
    assert!(
        fires(&orders, "CHWP", &w).is_empty(),
        "no second stage without a source"
    );
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut w);
    }
    let orders = think(&w);
    assert!(
        fires(&orders, "CHRN", &w).is_empty(),
        "the source is held: no new pick"
    );
    let dest = fires(&orders, "CHWP", &w);
    assert_eq!(dest.len(), 1, "the second stage: {dest:?}");
    let starts = glasnost_ai::skirmish::start_cells(&fixture::map());
    assert!(
        starts.iter().any(|&s| {
            let d = (s.x - dest[0].x).abs().max((s.y - dest[0].y).abs());
            (4..=9).contains(&d)
        }),
        "next to an enemy base: {dest:?} of {starts:?}"
    );
}

/// The teleport ready with a source picked on a group at `c` (the first
/// stage's order issued and 200 ticks run).
fn picked(c_at: (i32, i32)) -> (World, CellXY) {
    let (g, yard) = ready("CHRNB");
    let c = spot(yard, c_at.0, c_at.1);
    let mut w = g.world.clone();
    group(&mut w, c);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert_eq!(fires(&orders, "CHRN", &w), vec![c], "the first stage");
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut w);
    }
    (w, c)
}

// design: own AI (#2) and S-0512 claim 10 (the second stage uses the first stage's last picked source and does not consume it; with the checked order, only an unconsumed source selection is accepted): once the second stage has been fired, the brain does not fire it again at the next tick while no new source is picked.
#[test]
fn fires_the_second_stage_once_per_pick() {
    let (mut w, _) = picked((10, 8));
    let orders = think(&w);
    assert_eq!(fires(&orders, "CHWP", &w).len(), 1, "the second stage");
    // The second stage consumes the source selection (S-0512 claim 10); the
    // first stage's record keeps the source cell it last picked.
    for i in 0..w.rules().superweapons.types.len() as u32 {
        if let Some(r) = glasnost_sim::superweapons::record_mut(&mut w, HouseId(0), i) {
            r.source_fresh = false;
        }
    }
    for _ in 0..3 {
        glasnost_sim::tick::step(&mut w);
        let again = think(&w);
        assert!(
            fires(&again, "CHWP", &w).is_empty(),
            "no second stage without a new pick at tick {}",
            w.tick
        );
    }
}

// design: own AI (#2) and S-0512 claim 10 (the player may pick another source): a picked source whose block no longer holds the group (it drove off) is not teleported; the first stage is fired again on the group where it stands now.
#[test]
fn picks_again_when_the_group_has_left_the_source() {
    let (mut w, c) = picked((10, 8));
    let moved = CellXY::new(c.x + 6, c.y + 2);
    let ids: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Vehicle)
        .filter(|o| (o.pos.cell().x - c.x).abs() <= 1 && (o.pos.cell().y - c.y).abs() <= 1)
        .map(|o| o.id)
        .collect();
    assert_eq!(ids.len(), 3, "the group stands at the source");
    for id in ids {
        glasnost_sim::objects::remove(&mut w, id);
    }
    group(&mut w, moved);
    glasnost_sim::tick::step(&mut w);
    let orders = think(&w);
    assert!(
        fires(&orders, "CHWP", &w).is_empty(),
        "no teleport from a source the group has left"
    );
    assert_eq!(fires(&orders, "CHRN", &w), vec![moved], "a new pick");
}
